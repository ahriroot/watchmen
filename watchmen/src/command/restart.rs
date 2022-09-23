use std::{collections::HashMap, error::Error};

use crate::{
    const_exit_code::ExitCode,
    entity::{self, Options},
    socket,
};

const RESTART_HELP: &str = r#"Usage: watchmen restart [OPTION...] ...
  -h, --help     display this help of 'restart' command

  -i, --id       restart a task with the specified id
  -n, --name     restart a task with the specified name

Report bugs to ahriknow@ahriknow.com.""#;

pub async fn run(args: &[String]) -> Result<ExitCode, Box<dyn Error>> {
    let len = args.len();
    if len < 1 {
        println!("{}", RESTART_HELP);
        return Ok(ExitCode::SUCCESS);
    }
    let code = match args[0].as_str() {
        "-h" | "--help" => {
            println!("{}", RESTART_HELP);
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
                } else if args[0] == "-i" || args[0] == "--id" {
                    let id = args[1].parse::<u128>();
                    match id {
                        Ok(i) => {
                            options.insert(
                                "id".to_string(),
                                Options {
                                    key: "id".to_string(),
                                    value: entity::Opt::U128(i),
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
                name: "restart".to_string(),
                command: entity::Command {
                    name: "restart".to_string(),
                    options: options,
                    args: args,
                },
            };
            let res = socket::request(&req).await?;
            println!("restart command: {:?}", res);
            ExitCode::SUCCESS
        }
    };
    Ok(code)
}
