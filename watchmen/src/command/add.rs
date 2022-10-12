use std::{collections::HashMap, error::Error};

use crate::{
    entity::{self, Opt},
    socket,
};

const ADD_HELP: &str = r#"Usage: watchmen add [OPTION...] ...
    -h, --help          display this help of 'add' command

    -n, --name          task name, default is a random string

    -o, --origin        recurring task start time
                        format: YYYYMMDD.HHMMSS | YYYYMMDD | MMDD | MMDD.HHMMSS | HHMMSS
                        example: 20201231.235959 | 20201231 | 1231 | 1231.235959 | 235959

    -i, --interval      recurring task interval
                        format: 1d2h3m4s5 | 3m4s5 | 4s5 | 5 ...

    -t, --timing        scheduled Tasks
                        format: split by ',' YYYYMMDD.HHMMSS | YYYYMMDD | MMDD | MMDD.HHMMSS | HHMMSS
                        example: 20210101.000000,20210102.000000,20210103

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
                    let mut input = args[1].clone();

                    if !input.contains(".") {
                        input = match input.len() {
                            4 => {
                                // current year
                                let year = chrono::Local::now().format("%Y").to_string();
                                format!("{}{}.000000", year, input)
                            }
                            8 => format!("{}.000000", input),
                            6 => {
                                let date = chrono::Local::now().format("%Y%m%d").to_string();
                                format!("{}.{}", date, input)
                            }
                            _ => {
                                println!("Invalid timestamp: {}", input);
                                return Ok(entity::Response::f(format!(
                                    "Arg '{}' must be a number",
                                    args[0]
                                )));
                            }
                        };
                    }
                    if input.len() == 11 {
                        let year = chrono::Local::now().format("%Y").to_string();
                        input = format!("{}{}", year, input);
                    }
                    let timestamp = chrono::NaiveDateTime::parse_from_str(&input, "%Y%m%d.%H%M%S")
                        .unwrap()
                        .timestamp_millis() as u128;
                    options.insert("pid".to_string(), Opt::U128(timestamp));
                } else if args[0] == "-i" || args[0] == "--interval" {
                    let mut ms: u128 = 0;
                    let mut num: u128 = 0;

                    for c in args[1].chars() {
                        if c.is_numeric() {
                            num = num * 10 + c.to_digit(10).unwrap() as u128;
                        } else {
                            match c {
                                'd' => {
                                    ms += num * 24 * 60 * 60 * 1000;
                                }
                                'h' => {
                                    ms += num * 60 * 60 * 1000;
                                }
                                'm' => {
                                    ms += num * 60 * 1000;
                                }
                                's' => {
                                    ms += num * 1000;
                                }
                                _ => {
                                    return Ok(entity::Response::f(format!(
                                        "Arg '{}' value invalid unit",
                                        args[0]
                                    )));
                                }
                            }
                            num = 0;
                        }
                    }
                    if num > 999 {
                        return Ok(entity::Response::f(format!(
                            "Arg '{}' value 'ms' must less 1000",
                            args[0]
                        )));
                    }
                    ms += num;
                    options.insert("interval".to_string(), Opt::U128(ms));
                } else if args[0] == "-t" || args[0] == "--timing" {
                    let inputs = args[1].clone();
                    let mut timestamps: Vec<u128> = Vec::new();
                    for ipt in inputs.split(",") {
                        let mut input = ipt.to_string();
                        if !input.contains(".") {
                            input = match input.len() {
                                4 => {
                                    // current year
                                    let year = chrono::Local::now().format("%Y").to_string();
                                    format!("{}{}.000000", year, input)
                                }
                                8 => format!("{}.000000", input),
                                6 => {
                                    let date = chrono::Local::now().format("%Y%m%d").to_string();
                                    format!("{}.{}", date, input)
                                }
                                _ => {
                                    println!("Invalid timestamp: {}", input);
                                    return Ok(entity::Response::f(format!(
                                        "Arg '{}' must be a number",
                                        args[0]
                                    )));
                                }
                            };
                        }
                        if input.len() == 11 {
                            let year = chrono::Local::now().format("%Y").to_string();
                            input = format!("{}{}", year, input);
                        }
                        let timestamp =
                            chrono::NaiveDateTime::parse_from_str(&input, "%Y%m%d.%H%M%S")
                                .unwrap()
                                .timestamp_millis() as u128;
                        timestamps.push(timestamp);
                    }
                    options.insert("timing".to_string(), Opt::U128s(timestamps));
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
