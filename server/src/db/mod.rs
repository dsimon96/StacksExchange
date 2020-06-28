pub mod models;
pub mod schema;

use anyhow::Result;
use diesel::{pg::PgConnection, r2d2};
use serde::Deserialize;

pub type Pool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;

#[derive(Clone, Deserialize)]
/// Configuration params for database. See https://docs.rs/postgres/0.17.3/postgres/config/struct.Config.html
pub struct DatabaseSettings {
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub dbname: String,
    pub application_name: Option<String>,
    pub connect_timeout_sec: u64,
    pub pool_timeout_ms: u64,
    pub read_timeout_ms: u64,
}

impl DatabaseSettings {
    /// Obtain a postgres url (https://www.postgresql.org/docs/current/libpq-connect.html#LIBPQ-CONNSTRING)
    /// with the given configuration parameters
    fn to_postgres_url(&self) -> String {
        let mut url = format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.dbname
        );

        let mut params = Vec::new();

        if let Some(app_name) = &self.application_name {
            params.push(format!("application_name={}", app_name));
        }

        if self.connect_timeout_sec > 0 {
            params.push(format!("connect_timeout={}", self.connect_timeout_sec));
        }

        if !params.is_empty() {
            url += "?";
            url += &params.join("&");
        }

        url
    }
}

/// Create a database connection pool with the given configuration parameters
pub fn make_pool(cfg: &DatabaseSettings) -> Result<Pool> {
    let manager = r2d2::ConnectionManager::<PgConnection>::new(cfg.to_postgres_url());

    Ok(r2d2::Pool::new(manager)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_postgres_url() {
        let basic = DatabaseSettings {
            user: String::from("user"),
            password: String::from("hunter2"),
            host: String::from("host"),
            port: 1337,
            dbname: String::from("mydb"),
            application_name: None,
            connect_timeout_sec: 0,
            pool_timeout_ms: 0,
            read_timeout_ms: 0,
        };
        let with_app_name = DatabaseSettings {
            application_name: Some(String::from("myapp")),
            ..basic.clone()
        };
        let with_timeout = DatabaseSettings {
            connect_timeout_sec: 42,
            ..basic.clone()
        };
        let with_both = DatabaseSettings {
            connect_timeout_sec: 42,
            ..with_app_name.clone()
        };

        assert_eq!(
            "postgres://user:hunter2@host:1337/mydb",
            basic.to_postgres_url()
        );
        assert_eq!(
            "postgres://user:hunter2@host:1337/mydb?application_name=myapp",
            with_app_name.to_postgres_url()
        );
        assert_eq!(
            "postgres://user:hunter2@host:1337/mydb?connect_timeout=42",
            with_timeout.to_postgres_url()
        );
        assert_eq!(
            "postgres://user:hunter2@host:1337/mydb?application_name=myapp&connect_timeout=42",
            with_both.to_postgres_url()
        );
    }
}
