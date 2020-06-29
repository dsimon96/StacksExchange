mod app;
mod graphql;

use actix_web::{middleware, web, App, HttpServer};
use anyhow::Result;
use config::Config;
use std::convert::TryFrom;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use structopt::StructOpt;

#[cfg(feature = "autoreload")]
use listenfd::ListenFd;

#[derive(Debug, StructOpt)]
#[structopt(name = "stacks_exchange")]
struct Opt {
    #[structopt(parse(from_os_str), short, long)]
    conf: Option<PathBuf>,
}

fn init_cfg() -> Result<Config> {
    let opt = Opt::from_args();

    // always apply default config to ensure all settings defined
    let mut cfg = Config::new();
    cfg.merge(config::File::with_name("conf/default.toml"))?;

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

    // validate server config values before doing anything else
    let listen_addr = cfg.get_str("listen_addr").unwrap();
    let listen_port = u16::try_from(cfg.get_int("listen_port").unwrap())?;
    let server_name = cfg.get_str("server_name").unwrap();
    let addr = SocketAddr::from((listen_addr.parse::<IpAddr>()?, listen_port));

    let context = web::Data::new(graphql::Context::new());

    let mut server = HttpServer::new(move || {
        let app = App::new()
            .data(graphql::make_schema())
            .app_data(context.clone())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(
                web::resource("graphql")
                    .name("graphql")
                    .route(web::post().to(app::graphql))
                    .route(web::get().to(app::graphql)),
            );

        if cfg!(feature = "graphiql") {
            app.service(web::resource("graphiql").route(web::get().to(app::graphiql)))
                .service(web::resource("playground").route(web::get().to(app::playground)))
        } else {
            app
        }
    })
    .server_hostname(server_name);

    #[cfg(feature = "autoreload")]
    {
        let mut listenfd = ListenFd::from_env();
        server = if let Some(l) = listenfd.take_tcp_listener(0)? {
            server.listen(l)?
        } else {
            server.bind(addr)?
        }
    }

    #[cfg(not(feature = "autoreload"))]
    {
        server = server.bind(addr)?;
    }

    Ok(server.run().await?)
}
