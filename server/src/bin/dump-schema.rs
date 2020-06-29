use anyhow::Result;
use juniper::{introspect, IntrospectionFormat};
use stacks_exchange::{
    db::make_pool,
    graphql::{make_schema, Context},
    settings::Settings,
};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "dump_schema")]
struct Opt {
    #[structopt(parse(from_os_str), short, long)]
    conf: Option<PathBuf>,
}

fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let opt = Opt::from_args();
    let settings = Settings::init(opt.conf)?;
    pretty_env_logger::init();

    let pool = make_pool(&settings.db)?;

    let (res, _errors) = introspect(
        &make_schema(),
        &Context::new(settings, pool),
        IntrospectionFormat::default(),
    )
    .unwrap();

    let json = serde_json::to_string_pretty(&res)?;

    println!("{}", json);
    Ok(())
}
