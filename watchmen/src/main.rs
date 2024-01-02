use colored::Colorize;
use std::{env, error::Error};

use watchmen::common::{arg::TaskArgs, config::Config, handle::Response};
use watchmen::{args, commands::handle_exec, utils::print_result};

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

    if let Some(commands) = clargs.commands {
        if let Some(engine) = clargs.engine {
            config.watchmen.engine = engine;
        }
        let res = handle_exec(commands, config).await;
        if let Err(e) = res {
            print_result(vec![Response::failed(e.to_string())]).await;
        }
    }
    return Ok(());
}
