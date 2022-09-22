use std::error::Error;

use crate::{const_exit_code::ExitCode, entity, socket};

const LIST_HELP: &str = r#"Usage: watchmen list [OPTION...] [SECTION] PAGE...
  -h, --help     display this help of 'start' command

Report bugs to ahriknow@ahriknow.com.""#;

pub async fn run(args: &[String]) -> Result<ExitCode, Box<dyn Error>> {
    let len = args.len();
    if len == 1 && (args[0] == "-h" || args[0] == "--help") {
        println!("{}", LIST_HELP);
        return Ok(ExitCode::SUCCESS);
    }
    let req = entity::Request {
        name: "list".to_string(),
        command: entity::Command {
            name: "list".to_string(),
            args: args.to_vec(),
        },
    };
    let res = socket::request(&req).await?;
    println!("start command: {:?}", res);
    Ok(ExitCode::SUCCESS)
}
