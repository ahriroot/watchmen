use std::error::Error;

use crate::{const_exit_code::ExitCode, entity, socket};

const START_HELP: &str = r#"Usage: watchmen start [OPTION...] [SECTION] PAGE...
  -h, --help     display this help of 'start' command

Report bugs to ahriknow@ahriknow.com.""#;

pub async fn run(args: &[String]) -> Result<ExitCode, Box<dyn Error>> {
    let len = args.len();
    if len < 1 {
        println!("{}", START_HELP);
        return Ok(ExitCode::SUCCESS);
    }
    let code = match args[0].as_str() {
        "-h" | "--help" => {
            println!("{}", START_HELP);
            ExitCode::SUCCESS
        }
        _ => {
            let req = entity::Request {
                name: "start".to_string(),
                command: entity::Command {
                    name: "start".to_string(),
                    args: args.to_vec(),
                },
            };
            let res = socket::request(&req).await?;
            println!("start command: {:?}", res);
            ExitCode::SUCCESS
        }
    };
    Ok(code)
}
