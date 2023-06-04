use colored::Colorize;
use common::{arg::DaemonArgs, config::Config};
use std::error::Error;

use daemon::engine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let clargs = DaemonArgs::new();
    if clargs.version {
        println!(
            "{} {}",
            "Watchmen rust".green(),
            env!("CARGO_PKG_VERSION").green()
        );
        return Ok(());
    }

    let config: Config = Config::init(clargs.config.clone())?;
    engine::start(config).await;
    // println!("{:#?}", config);
    return Ok(());
}
