use colored::Colorize;
use std::{env, error::Error};

use common::{arg::TaskArgs, config::Config};
use watchmen::{args, commands::handle_exec};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let clargs = TaskArgs::new();
    if clargs.version {
        println!(
            "{} {}",
            "Watchmen rust".green(),
            env!("CARGO_PKG_VERSION").green()
        );
        return Ok(());
    }

    if let Some(path) = &clargs.generate {
        return args::generate(path);
    }

    let mut config: Config = Config::init(clargs.config.clone())?;

    if clargs.daemon {
        return args::daemon(config);
    }

    if let Some(commands) = clargs.commands {
        if let Some(engine) = clargs.engine {
            config.watchmen.engine = engine;
        }
        return handle_exec(commands, config).await;
    }
    return Ok(());
}
