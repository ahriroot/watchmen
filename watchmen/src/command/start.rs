use std::{collections::HashMap, error::Error};

use crate::{
    entity::{self, Opt},
    socket,
};

const START_HELP: &str = r#"Usage: watchmen start [OPTION...] ...
  -h, --help     display this help of 'start' command

  -i, --id       start a task with the specified id
  -n, --name     start a task with the specified name

Report bugs to ahriknow@ahriknow.com.""#;

pub async fn run(args: &[String]) -> Result<entity::Response, Box<dyn Error>> {
    let len = args.len();
    if len < 1 {
        return Ok(entity::Response::ok(START_HELP));
    }
    let response = match args[0].as_str() {
        "-h" | "--help" => entity::Response::ok(START_HELP),
        _ => {
            let mut options: HashMap<String, Opt> = HashMap::new();

            let mut args: Vec<String> = args.to_vec();
            while args.len() > 1 {
                if args[0] == "-n" || args[0] == "--name" {
                    options.insert("name".to_string(), Opt::Str(args[1].clone()));
                } else if args[0] == "-i" || args[0] == "--id" {
                    let id = args[1].parse::<u128>();
                    match id {
                        Ok(i) => {
                            options.insert("id".to_string(), Opt::U128(i));
                        }
                        Err(_) => {
                            return Ok(entity::Response::err(format!(
                                "Arg '{}' must be a number",
                                args[0]
                            )));
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
                    options: options,
                    args: args,
                },
            };
            let res = socket::request(&req).await?;
            res
        }
    };
    Ok(response)
}
