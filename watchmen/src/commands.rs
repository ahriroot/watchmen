// pub mod add;
// pub mod exit;
// pub mod list;
// pub mod pause;
// pub mod restart;
// pub mod resume;
pub mod add;
pub mod list;
pub mod remove;
pub mod run;
pub mod start;
pub mod stop;

use common::arg::Commands;
use common::config::Config;
use std::error::Error;

pub async fn handle_exec(commands: Commands, config: Config) -> Result<(), Box<dyn Error>> {
    match commands {
        Commands::Run(args) => self::run::run(args, config).await?,
        Commands::Add(args) => self::add::add(args, config).await?,
        Commands::Start(args) => self::start::start(args, config).await?,
        Commands::Stop(args) => self::stop::stop(args, config).await?,
        Commands::Remove(args) => self::remove::remove(args, config).await?,
        Commands::List(args) => self::list::list(args, config).await?,
    }
    Ok(())
}
