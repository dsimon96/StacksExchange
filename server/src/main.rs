mod app;
mod db;
mod graphql;
mod settings;

#[macro_use]
extern crate diesel;

use actix_web::{middleware, web, App, HttpServer};
use anyhow::Result;
use dotenv;
use settings::Settings;
use std::{
    convert::TryFrom,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};
use structopt::StructOpt;

#[cfg(feature = "autoreload")]
use listenfd::ListenFd;

#[derive(Debug, StructOpt)]
#[structopt(name = "stacks_exchange")]
struct Opt {
    #[structopt(parse(from_os_str), short, long)]
    conf: Option<PathBuf>,
}

#[actix_rt::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let opt = Opt::from_args();
    let settings = Settings::init(opt.conf)?;
    pretty_env_logger::init();

    // validate server config values before doing anything else
    let addr = SocketAddr::from((
        settings.server.listen_addr.parse::<IpAddr>()?,
        u16::try_from(settings.server.listen_port)?,
    ));
    let pool = db::make_pool(&settings.db)?;
    let server_name = settings.server.name.clone();

    let mut server = HttpServer::new(move || {
        let app = App::new()
            .data(graphql::make_schema())
            .data(pool.clone())
            .data(settings.clone())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(
                web::resource("/graphql")
                    .name("graphql")
                    .route(web::post().to(app::graphql))
                    .route(web::get().to(app::graphql)),
            );

        if cfg!(feature = "graphiql") {
            app.service(web::resource("/graphiql").route(web::get().to(app::graphiql)))
                .service(web::resource("/playground").route(web::get().to(app::playground)))
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
