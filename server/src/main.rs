mod app;
mod auth;
mod db;
mod googlesignin;
mod graphql;
mod settings;

#[macro_use]
extern crate diesel;

use actix_files::Files;
use actix_web::{middleware, web, App, HttpServer};
use anyhow::Result;
use settings::Settings;
use std::{
    env,
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
        env::var("PORT")
            .ok()
            .map(|s| s.parse::<u16>().unwrap())
            .unwrap_or(settings.server.listen_port),
    ));
    let pool =
        db::make_pool(&env::var("DATABASE_URL").unwrap_or_else(|_| settings.db.to_string()))?;
    let server_name = settings.server.name.clone();

    let mut server = HttpServer::new(move || {
        let app = App::new()
            .data(graphql::make_schema(settings.clone(), pool.clone()))
            .data(settings.clone())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(
                web::resource("/oauth")
                    .name("oauth")
                    .route(web::post().to(auth::oauth_handler)),
            )
            .service(
                web::resource("/graphql")
                    .name("graphql")
                    .route(web::post().to(app::graphql))
                    .route(web::get().to(app::graphql)),
            );

        let app = if cfg!(feature = "graphiql") {
            app.service(web::resource("/graphiql").route(web::get().to(app::graphiql)))
                .service(web::resource("/playground").route(web::get().to(app::playground)))
        } else {
            app
        };

        app.service(Files::new("/", "./static").index_file("index.html"))
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
