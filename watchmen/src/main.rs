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
    let exec_result = command::exec(args).await;
    match exec_result {
        Ok(exit_code) => exit(exit_code as i32),
        Err(err) => {
            eprintln!("{}", err);
            exit(ExitCode::ERROR as i32);
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use chrono::{Local, TimeZone};

    #[test]
    fn format_date() {
        // timestamp to date
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let date = Local.timestamp_millis(timestamp as i64);

        let s = date.format("%Y-%m-%d %H:%M:%S").to_string();
        println!("{}", s);
    }

    #[test]
    fn test() {
        let mut args = vec![
            "-o".to_string(),
            "123".to_string(),
            "-i".to_string(),
            "456".to_string(),
            "python".to_string(),
            "1.py".to_string(),
        ];

        let mut index = 0;
        while args.len() > 1 {
            index += 1;
            println!("{} {:?}", index, args);
            if args[0] == "-n" || args[0] == "--name" {
            } else if args[0] == "-o" || args[0] == "--origin" {
            } else if args[0] == "-i" || args[0] == "--interval" {
            } else {
                break;
            }
            args.remove(0);
            args.remove(0);
        }

        println!("-----------");
        println!("{:?}", args);
    }
}
