use std::{collections::HashMap, error::Error};

use crate::{
    entity::{self, Opt},
    socket,
};

const PAUSE_HELP: &str = r#"Usage: watchmen pause [OPTION...] ...
    -h, --help     display this help of 'pause' command

    -i, --id       task id
    -n, --name     task name
    -p, --pid      task pid

Report bugs to ahriknow@ahriknow.com
Issues: https://git.ahriknow.com/ahriknow/watchmen/issues"#;

pub async fn run(args: &[String], home_path: String) -> Result<entity::Response, Box<dyn Error>> {
    let len = args.len();
    if len < 1 {
        return Ok(entity::Response::ok(PAUSE_HELP));
    }
    let response = match args[0].as_str() {
        "-h" | "--help" => entity::Response::ok(PAUSE_HELP),
        _ => {
            let mut options: HashMap<String, Opt> = HashMap::new();

            let mut args: Vec<String> = args.to_vec();
            while args.len() > 1 {
                if args[0] == "-n" || args[0] == "--name" {
                    options.insert("name".to_string(), Opt::Str(args[1].clone()));
                } else if args[0] == "-p" || args[0] == "--pid" {
                    let pid = args[1].parse::<u32>();
                    match pid {
                        Ok(p) => {
                            options.insert("pid".to_string(), Opt::U32(p));
                        }
                        Err(_) => {
                            return Ok(entity::Response::f(format!(
                                "Arg '{}' must be a number",
                                args[0]
                            )));
                        }
                    }
                } else if args[0] == "-i" || args[0] == "--id" {
                    let id = args[1].parse::<u128>();
                    match id {
                        Ok(i) => {
                            options.insert("id".to_string(), Opt::U128(i));
                        }
                        Err(_) => {
                            return Ok(entity::Response::f(format!(
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
                name: "pause".to_string(),
                command: entity::Command {
                    name: "pause".to_string(),
                    options: options,
                    args: args,
                },
            };
            let res = socket::request(&req, home_path).await?;
            res
        }
    };
    Ok(response)
}