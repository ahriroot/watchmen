use std::{collections::HashMap, error::Error};

use crate::{
    entity::{self, Opt},
    socket,
};

const ADD_HELP: &str = r#"Usage: watchmen add [OPTION...] ...
    -h, --help          display this help of 'add' command

    -n, --name          task name, default is a random string
    -o, --origin        recurring task start time
    -i, --interval      recurring task interval

Report bugs to ahriknow@ahriknow.com.""#;

pub async fn run(args: &[String]) -> Result<entity::Response, Box<dyn Error>> {
    let len = args.len();
    if len < 1 {
        return Ok(entity::Response::ok(ADD_HELP));
    }
    let response = match args[0].as_str() {
        "-h" | "--help" => entity::Response::ok(ADD_HELP),
        _ => {
            let mut options: HashMap<String, Opt> = HashMap::new();

            let mut args: Vec<String> = args.to_vec();
            while args.len() > 1 {
                if args[0] == "-n" || args[0] == "--name" {
                    if args[1].starts_with("-") {
                        return Ok(entity::Response::err(format!(
                            "Value of '{}' connot start with '-'",
                            args[0]
                        )));
                    }
                    options.insert("name".to_string(), Opt::Str(args[1].clone()));
                } else if args[0] == "-o" || args[0] == "--origin" {
                    let origin = args[1].parse::<u128>();
                    match origin {
                        Ok(o) => {
                            options.insert("pid".to_string(), Opt::U128(o));
                        }
                        Err(_) => {
                            return Ok(entity::Response::f(format!(
                                "Arg '{}' must be a number",
                                args[0]
                            )));
                        }
                    }
                } else if args[0] == "-i" || args[0] == "--interval" {
                    let interval = args[1].parse::<u128>();
                    match interval {
                        Ok(i) => {
                            options.insert("interval".to_string(), Opt::U128(i));
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
                name: "add".to_string(),
                command: entity::Command {
                    name: "add".to_string(),
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
