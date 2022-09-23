pub mod exit;
pub mod list;
pub mod restart;
pub mod run;
pub mod start;
pub mod stop;

use colored::Colorize;
use std::error::Error;
use std::fs::remove_file;
use std::io::Write;
use std::process::Stdio;
use tokio::process::Command;

use crate::command;
use crate::const_exit_code::ExitCode;

const HTLP: &str = r#"Usage: watchmen [OPTION|SUBCOMMAND] ...
  -h, --help        display this help and exit
  -v, --version     display version information and exit
  -i, --info        display information about watchmen and exit

  -d, --daemon      startup watchmen daemon
  -t, --terminated  terminated watchmen daemon

  run
    create a task and run it
    run `watchmen run -h` for more information

  drop | exit | rm
    drop a task and stop if it is running
    run `watchmen [this] -h` for more information

  start
    start a task if it is exists
    run `watchmen start -h` for more information

  restart
    restart a task if it is exists
    run `watchmen restart -h` for more information

  stop
    stop a task if it is running
    run `watchmen stop -h` for more information

  list
    list all tasks
    run `watchmen list -h` for more information

Report bugs to ahriknow@ahriknow.com
Issues: https://git.ahriknow.com/ahriknow/watchmen/issues"#;
const INFO: &str = r#"watchmen 0.1.0
Homepage: https://watchmen.ahriknow.com/"#;
const VERSION: &str = "watchmen 0.1.0";

pub async fn exec(args: Vec<String>) -> Result<ExitCode, Box<dyn Error>> {
    let len = args.len();
    if len < 2 {
        println!("{}", HTLP);
        return Ok(ExitCode::SUCCESS);
    }
    let exit_code = match args[1].as_str() {
        "-h" | "--help" => {
            println!("{}", HTLP);
            ExitCode::SUCCESS
        }
        "-i" | "--info" => {
            println!("{}", INFO);
            ExitCode::SUCCESS
        }
        "-v" | "--version" => {
            println!("{}", VERSION);
            ExitCode::SUCCESS
        }
        "run" => command::run::run(&args[2..]).await?,
        "exit" | "rm" | "drop" => command::exit::run(&args[2..]).await?,
        "start" => command::start::run(&args[2..]).await?,
        "restart" => command::restart::run(&args[2..]).await?,
        "stop" => command::stop::run(&args[2..]).await?,
        "list" => command::list::run(&args[2..]).await?,
        "-d" | "--daemon" => start_daemon(&args[2..]).await?,
        "-t" | "--terminated" => terminated_daemon(&args[2..]).await?,
        _ => {
            let err: String = format!("watchmen: invalid command '{}'", args[1]);
            eprintln!("{}", err.red());
            ExitCode::ERROR
        }
    };

    Ok(exit_code)
}

async fn start_daemon(_args: &[String]) -> Result<ExitCode, Box<dyn Error>> {
    let watchmen_path_str =
        std::env::var("WATCHMEN_PATH").unwrap_or_else(|_| "/tmp/watchmen".to_string());

    let watchmen_path = std::path::Path::new(&watchmen_path_str);

    if !watchmen_path.exists() {
        std::fs::create_dir_all(watchmen_path)?;
    }

    let stdout_path = watchmen_path.join("stdout/").clone();
    if !stdout_path.exists() {
        std::fs::create_dir(stdout_path.clone()).unwrap();
    }

    let daemon_stdout = watchmen_path.join("daemon.log").clone();
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(daemon_stdout.clone())?;
    let stdout = Stdio::from(file);

    let sock_path = watchmen_path.join("watchmen.sock");

    let path = std::env::current_dir()?.join("daemon");
    // 子进程
    let mut child = Command::new(path)
        .arg(sock_path)
        .arg(watchmen_path)
        .arg(stdout_path)
        .stdout(stdout)
        .spawn()?;

    let result = child.id();

    match result {
        Some(pid) => {
            // 保存 pid
            let path = std::path::Path::new(&watchmen_path_str);
            let path = path.join("watchmen.pid");
            // 创建文件
            let mut file = std::fs::File::create(path.clone())?;
            // 写入 pid
            file.write_all(pid.to_string().as_bytes())?;

            println!("Start daemon pid: {}", pid);
            tokio::spawn(async move {
                child.wait().await.unwrap();
                remove_file(path).unwrap_or_default();
            });
            Ok(ExitCode::SUCCESS)
        }
        None => {
            eprintln!("watchmen: failed to start daemon");
            Ok(ExitCode::ERROR)
        }
    }
}
extern "C" {
    pub fn kill(pid: i32, sig: i32) -> i32;
}
async fn terminated_daemon(args: &[String]) -> Result<ExitCode, Box<dyn Error>> {
    let mut signal = 15;
    if args.len() == 1 {
        match args[0].as_str() {
            "-f" | "--force" | "-9" => {
                signal = 9;
            }
            _ => {
                let err: String = format!("Invalid signal: '{}'", args[0]);
                return Err(err.into());
            }
        }
    }

    let watchmen_path =
        std::env::var("WATCHMEN_PATH").unwrap_or_else(|_| "/tmp/watchmen".to_string());
    let path = std::path::Path::new(&watchmen_path);
    let path = path.join("watchmen.pid");
    if !path.exists() {
        eprintln!("Pid file not exists: {}", path.display());
        return Ok(ExitCode::ERROR);
    }
    let pid = std::fs::read_to_string(path.clone())?;
    let pid = pid.parse::<i32>()?;
    let res = unsafe { kill(pid, signal) };
    if res == 0 {
        // 删除文件
        remove_file(path).unwrap_or_default();
        println!("Terminated daemon pid: {}", pid);
    } else if res == -1 {
        eprintln!("Daemon process not exists: {}", pid);
        return Ok(ExitCode::ERROR);
    } else {
        eprintln!("Terminated daemon error code: {}", res);
        return Ok(ExitCode::ERROR);
    }
    Ok(ExitCode::SUCCESS)
}
