use crate::db::DatabaseSettings;
use anyhow::Result;
use config::{Config, Environment, File};
use serde::Deserialize;
use std::path::PathBuf;

/// Config parameters pertaining to server / network
#[derive(Clone, Deserialize)]
pub struct ServerSettings {
    pub listen_addr: String,
    pub listen_port: u16,
    pub name: String,
}

/// Container for all config parameters
#[derive(Clone, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub db: DatabaseSettings,
}

impl Settings {
    /// Obtain configuration from default path, override path, and environment
    pub fn init(override_path: Option<PathBuf>) -> Result<Self> {
        // always apply default config to ensure all settings defined
        let mut cfg = Config::new();
        cfg.merge(File::with_name("conf/default.toml"))?;

        // apply a conf override file if one is provided
        if let Some(path) = override_path {
            cfg.merge(File::from(path))?;
        }

        // apply any conf overrides provided in env (e.g. DEF_addr="...")
        cfg.merge(Environment::with_prefix("DEF").separator("_"))?;

        Ok(cfg.try_into()?)
    }
}
