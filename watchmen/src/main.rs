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
    use std::{env, fs};

    #[test]
    fn test() {
        let watchmen_path =
            env::var("WATCHMEN_PATH").unwrap_or_else(|_| "/tmp/watchmen".to_string());

        let sock_path = std::path::Path::new(&watchmen_path);

        if !sock_path.exists() {
            println!("{}", sock_path.display());
            fs::create_dir_all(sock_path).unwrap();
        }

        let stdout_path = sock_path.join("stdout/").clone();
        if !stdout_path.exists() {
            fs::create_dir(stdout_path).unwrap();
        }
    }
}
