use colored::Colorize;
use std::{env, error::Error, fs, process::exit};

use watchmen::command;
use watchmen::const_exit_code::ExitCode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let watchmen_path = env::var("WATCHMEN_PATH").unwrap_or_else(|_| "/tmp/watchmen".to_string());

    let sock_path = std::path::Path::new(&watchmen_path);

    if !sock_path.exists() {
        fs::create_dir_all(sock_path)?;
    }

    let stdout_path = sock_path.join("stdout/").clone();
    if !stdout_path.exists() {
        fs::create_dir(stdout_path).unwrap();
    }

    // 命令行参数 / command line arguments
    let args: Vec<String> = std::env::args().collect();
    // 执行命令 / execute command
    let response = command::exec(args).await;
    match response {
        Ok(res) => {
            let mut code = -1;
            if res.code == 10 {
                code = 0;
                println!("{}", res.msg.green());
            } else if res.code >= 50000 {
                eprintln!("{}", res.msg.blue());
            } else if res.code >= 40000 {
                eprintln!("{}", res.msg.red());
            } else if res.code >= 20000 {
                eprintln!("{}", res.msg.yellow());
            } else if res.code >= 10000 {
                code = 0;
                println!("{}", res.msg.green());
            } else {
                eprintln!("{}", res.msg);
            }
            exit(code);
        }
        Err(err) => {
            eprintln!("{}", err);
            exit(ExitCode::ERROR as i32);
        }
    }
}

#[cfg(test)]
pub mod tests {

    use std::time::{SystemTime, UNIX_EPOCH};

    use chrono::prelude::*;
    #[test]
    fn test() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        println!("{}", now);
        let now: DateTime<Local> = Local::now();
        println!("{}", now.to_string());
        println!("{}", now.format("%Y-%m-%d %H:%M:%S"));
    }
}
