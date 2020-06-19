mod app;
mod graphql;

use actix_web::{middleware, App, HttpServer};
use anyhow::Result;
use config::Config;
use std::path::PathBuf;
use std::sync::Arc;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "stacks_exchange")]
struct Opt {
    #[structopt(parse(from_os_str), short, long)]
    conf: Option<PathBuf>
}

fn init_cfg() -> Result<Config> {
    let opt = Opt::from_args();

    // always apply default config to ensure all settings defined
    let mut cfg = Config::new();
    cfg.merge(config::File::with_name("conf/default.toml")).unwrap();

    // apply a conf override file if one is provided
    if let Some(path) = opt.conf {
        cfg.merge(config::File::from(path))?;
    }

    // apply any conf overrides provided in env (e.g. DEF_addr="...")
    cfg.merge(config::Environment::with_prefix("DEF"))?;

    pretty_env_logger::init();
    Ok(cfg)
}

#[actix_rt::main]
async fn main() -> Result<()> {
    let cfg = init_cfg()?;

    let addr = cfg.get_str("addr").unwrap();
    let data = Arc::new(app::AppData::new(cfg));

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
