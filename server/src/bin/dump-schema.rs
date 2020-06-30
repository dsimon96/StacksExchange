use anyhow::Result;
use stacks_exchange::settings::Settings;
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
    let _settings = Settings::init(opt.conf)?;
    pretty_env_logger::init();

    unimplemented!();
}
