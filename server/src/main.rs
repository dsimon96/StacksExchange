#[macro_use]
extern crate lazy_static;
mod app;
mod graphql;

use actix_web::{middleware, App, HttpServer};
use anyhow::Result;
use config::Config;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use structopt::StructOpt;

lazy_static! {
    /// Globally-accessible config
    static ref CONFIG: RwLock<Config> = RwLock::new(Config::default());
}

#[derive(Debug, StructOpt)]
#[structopt(name = "stacks_exchange")]
struct Opt {
    #[structopt(parse(from_os_str), short, long)]
    conf: Option<PathBuf>
}

fn init_cfg() -> Result<()> {
    let opt = Opt::from_args();
    let mut cfg = CONFIG
        .write()
        .unwrap();

    // always apply default config to ensure all settings defined
    cfg
        .merge(config::File::with_name("conf/default.toml"))
        .unwrap();

    // apply a conf override file if one is provided
    if let Some(path) = opt.conf {
        cfg.merge(config::File::from(path))?;
    }

    // apply any conf overrides provided in env (e.g. DEF_addr="...")
    cfg.merge(config::Environment::with_prefix("DEF"))?;

    pretty_env_logger::init();
    Ok(())
}

#[actix_rt::main]
async fn main() -> Result<()> {
    init_cfg()?;

    let addr = CONFIG.read().unwrap().get_str("addr").unwrap();
    let data = Arc::new(app::AppData::new());

    Ok(HttpServer::new(move || {
        let app = App::new()
            .data(data.clone())
            .wrap(middleware::Logger::default())
            .service(app::graphql);

        if cfg!(feature = "graphiql") {
            app.service(app::graphiql)
        } else {
            app
        }
    })
    .bind(addr)?
    .run()
    .await?)
}
