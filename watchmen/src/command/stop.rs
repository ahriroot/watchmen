use std::{collections::HashMap, error::Error};

use crate::{
    const_exit_code::ExitCode,
    entity::{self, Options},
    socket,
};

const STOP_HELP: &str = r#"Usage: watchmen stop [OPTION...] [SECTION] PAGE...
  -h, --help     display this help of 'start' command

  -n, --name     stop a task with the specified name
  -p, --pid      stop a task with the specified pid

Report bugs to ahriknow@ahriknow.com.""#;

pub async fn run(args: &[String]) -> Result<ExitCode, Box<dyn Error>> {
    let len = args.len();
    if len < 1 {
        println!("{}", STOP_HELP);
        return Ok(ExitCode::SUCCESS);
    }
    let code = match args[0].as_str() {
        "-h" | "--help" => {
            println!("{}", STOP_HELP);
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
                    let pid = args[1].parse::<u32>();
                    match pid {
                        Ok(p) => {
                            options.insert(
                                "pid".to_string(),
                                Options {
                                    key: "pid".to_string(),
                                    value: entity::Opt::U32(p),
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
                name: "stop".to_string(),
                command: entity::Command {
                    name: "stop".to_string(),
                    options: options,
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
