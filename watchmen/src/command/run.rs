use colored::Colorize;
use std::error::Error;

use crate::{const_exit_code::ExitCode, entity, socket};

const RUN_HELP: &str = r#"Usage: watchmen run [OPTION...] [SECTION] PAGE...
  -h, --help     display this help of 'run' command

  -n, --name     create and start a task with the specified name
  -o, --origin   create and start a task with the specified origin
  -i, --interval create and start a task with the specified interval

Report bugs to ahriknow@ahriknow.com.""#;

pub async fn run(args: &[String]) -> Result<ExitCode, Box<dyn Error>> {
    let len = args.len();
    if len < 1 {
        println!("{}", RUN_HELP);
        return Ok(ExitCode::SUCCESS);
    }
    let code = match args[0].as_str() {
        "-h" | "--help" => {
            println!("{}", RUN_HELP);
            ExitCode::SUCCESS
        }
        _ => {
            let req = entity::Request {
                name: "run".to_string(),
                command: entity::Command {
                    name: "run".to_string(),
                    args: args.to_vec(),
                },
            };
            let res = socket::request(&req).await?;
            if res.code >= 50000 {
                println!("{}", res.msg.blue());
                return Ok(ExitCode::ERROR);
            } else if res.code >= 40000 {
                println!("{}", res.msg.red());
                return Ok(ExitCode::ERROR);
            } else if res.code >= 20000 {
                println!("{}", res.msg.yellow());
                return Ok(ExitCode::ERROR);
            } else if res.code >= 10000 {
                println!("{}", res.msg.green());
                return Ok(ExitCode::SUCCESS);
            } else {
                println!("{}", res.msg);
                return Ok(ExitCode::ERROR);
            }
        }
    };
    Ok(code)
}
