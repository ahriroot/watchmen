use colored::Colorize;
use std::{error::Error, collections::HashMap};

use crate::{const_exit_code::ExitCode, entity::{self, Options}, socket};

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
            let mut options: HashMap<String, Options> = HashMap::new();

            let mut args: Vec<String> = args.to_vec();
            while args.len() > 1 {
                if args[0] == "-n" || args[0] == "--name" {
                    options.insert(
                        "name".to_string(),
                        Options {
                            key: "name".to_string(),
                            value: entity::Opt::Str(args[1].clone()),
                        },
                    );
                } else if args[0] == "-p" || args[0] == "--pid" {
                    let pid = args[1].parse::<u128>();
                    match pid {
                        Ok(p) => {
                            options.insert(
                                "pid".to_string(),
                                Options {
                                    key: "pid".to_string(),
                                    value: entity::Opt::Usize(p),
                                },
                            );
                        }
                        Err(_) => {
                            eprintln!("Arg '{}' must be a number", args[0]);
                            return Ok(ExitCode::ERROR);
                        }
                    }
                } else {
                    break;
                }
                args.remove(0);
                args.remove(0);
            }
            let req = entity::Request {
                name: "run".to_string(),
                command: entity::Command {
                    name: "run".to_string(),
                    options: options,
                    args: args,
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
