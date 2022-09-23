use std::{error::Error, collections::HashMap};

use crate::{const_exit_code::ExitCode, entity::{self, Options}, socket};

const START_HELP: &str = r#"Usage: watchmen start [OPTION...] [SECTION] PAGE...
  -h, --help     display this help of 'start' command

  -n, --name     start a task with the specified name
  -p, --pid      start a task with the specified pid

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
                name: "start".to_string(),
                command: entity::Command {
                    name: "start".to_string(),
                    options:options,
                    args: args,
                },
            };
            let res = socket::request(&req).await?;
            println!("start command: {:?}", res);
            ExitCode::SUCCESS
        }
    };
    Ok(code)
}
