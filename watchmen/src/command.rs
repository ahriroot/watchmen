pub mod add;
pub mod exit;
pub mod list;
pub mod pause;
pub mod restart;
pub mod resume;
pub mod run;
pub mod start;
pub mod stop;

use std::fs::remove_file;
use std::io::Write;
use std::process::Stdio;
use std::{collections::HashMap, error::Error};
use tokio::process::Command;

use crate::{command, entity, socket};

const HELP: &str = r#"Usage: watchmen [OPTION|SUBCOMMAND] ...
    -h, --help        display this help
    -v, --version     display version
    -i, --info        display information

    -d, --daemon      startup watchmen daemon
    -t, --terminated  terminated watchmen daemon

    -gd, --guard-daemon         startup watchmen daemon
    -gt, --guardt-terminated    terminated watchmen daemon

    run
        create a task and run it
        run `watchmen run -h` for more information

    add
        create a task but not run it
        run `watchmen add -h` for more information

    drop | exit | rm
        stop and drop a task
        run `watchmen [this] -h` for more information

    start
        start a task if it is exists
        run `watchmen start -h` for more information

    restart
        restart a task
        run `watchmen restart -h` for more information

    stop
        stop a task
        run `watchmen stop -h` for more information

    pause
        pause a scheduled task
        run `watchmen stop -h` for more information

    resume
        resume a scheduled task
        run `watchmen stop -h` for more information

    list
        list tasks
        run `watchmen list -h` for more information

Report bugs to ahriknow@ahriknow.com
Issues: https://git.ahriknow.com/ahriknow/watchmen/issues"#;
const INFO: &str = r#"watchmen 0.1.0
Homepage: https://watchmen.ahriknow.com/"
Report bugs to ahriknow@ahriknow.com
Issues: https://git.ahriknow.com/ahriknow/watchmen/issues"#;
const VERSION: &str = "watchmen 0.1.0";

pub async fn exec(
    args: Vec<String>,
    home_path: String,
) -> Result<entity::Response, Box<dyn Error>> {
    let len = args.len();
    if len < 2 {
        return Ok(entity::Response::ok(HELP));
    }
    let response: entity::Response = match args[1].as_str() {
        "-h" | "--help" => entity::Response {
            code: 10000,
            msg: HELP.to_string(),
            data: None,
        },
        "-i" | "--info" => entity::Response {
            code: 10000,
            msg: INFO.to_string(),
            data: None,
        },
        "-v" | "--version" => entity::Response {
            code: 10000,
            msg: VERSION.to_string(),
            data: None,
        },
        "run" => command::run::run(&args[2..], home_path).await?,
        "add" => command::add::run(&args[2..], home_path).await?,
        "exit" | "rm" | "drop" => command::exit::run(&args[2..], home_path).await?,
        "start" => command::start::run(&args[2..], home_path).await?,
        "restart" => command::restart::run(&args[2..], home_path).await?,
        "stop" => command::stop::run(&args[2..], home_path).await?,
        "pause" => command::pause::run(&args[2..], home_path).await?,
        "resume" => command::resume::run(&args[2..], home_path).await?,
        "list" => command::list::run(&args[2..], home_path).await?,
        "-d" | "--daemon" => start_daemon(&args[2..], home_path).await?,
        "-t" | "--terminated" => terminated_daemon(&args[2..], home_path).await?,
        "-gd" | "--guard-daemon" => start_guard_daemon(&args[2..], home_path).await?,
        "-gt" | "--guardt-terminated" => terminated_guard_daemon(&args[2..], home_path).await?,
        _ => {
            let err: String = format!("watchmen: invalid command option '{}'", args[1]);
            entity::Response {
                code: 40000,
                msg: err,
                data: None,
            }
        }
    };

    Ok(response)
}

async fn start_daemon(
    _args: &[String],
    home_path: String,
) -> Result<entity::Response, Box<dyn Error>> {
    let watchmen_path = std::path::Path::new(&home_path);

    let stdout_path = watchmen_path.join("stdout/").clone();

    let daemon_stdout = watchmen_path.join("daemon.log").clone();
    let file_result = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(daemon_stdout.clone());
    if file_result.is_err() {
        let err: String = format!(
            "watchmen: cannot open stdout file '{}'",
            daemon_stdout.to_str().unwrap()
        );
        return Ok(entity::Response::err(err));
    }
    let stdout = Stdio::from(file_result.unwrap());

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
            let path = std::path::Path::new(&home_path);
            let path = path.join("daemon.pid");
            // 创建文件
            let mut file = std::fs::File::create(path.clone())?;
            // 写入 pid
            file.write_all(pid.to_string().as_bytes())?;

            tokio::spawn(async move {
                child.wait().await.unwrap();
                remove_file(path).unwrap_or_default();
            });
            Ok(entity::Response::ok(format!("Start daemon pid: {}", pid)))
        }
        None => Ok(entity::Response::err("watchmen: failed to start daemon")),
    }
}

extern "C" {
    pub fn kill(pid: i32, sig: i32) -> i32;
}

async fn terminated_daemon(
    args: &[String],
    home_path: String,
) -> Result<entity::Response, Box<dyn Error>> {
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

    let daemon_pid_path = std::path::Path::new(&home_path).join("daemon.pid");
    if !daemon_pid_path.exists() {
        return Ok(entity::Response::err(format!(
            "Pid file not exists: {}",
            daemon_pid_path.display()
        )));
    }
    let pid = std::fs::read_to_string(daemon_pid_path.clone())?;
    let pid = pid.parse::<i32>()?;
    let res = unsafe { kill(pid, signal) };
    if res == 0 {
        // 删除文件
        remove_file(daemon_pid_path).unwrap_or_default();
        return Ok(entity::Response::ok(format!(
            "Terminated daemon pid: {}",
            pid
        )));
    } else if res == -1 {
        return Ok(entity::Response::err(format!(
            "Daemon process not exists: {}",
            pid
        )));
    } else {
        return Ok(entity::Response::err(format!(
            "Terminated daemon error code: {}",
            res
        )));
    }
}

async fn start_guard_daemon(
    _args: &[String],
    home_path: String,
) -> Result<entity::Response, Box<dyn Error>> {
    let watchmen_path = std::path::Path::new(&home_path);

    let stdout_path = watchmen_path.join("stdout/").clone();

    let daemon_stdout = watchmen_path.join("guard.log").clone();
    let file_result = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(daemon_stdout.clone());
    if file_result.is_err() {
        let err: String = format!(
            "watchmen: cannot open stdout file '{}'",
            daemon_stdout.to_str().unwrap()
        );
        return Ok(entity::Response::err(err));
    }
    let stdout = Stdio::from(file_result.unwrap());

    let sock_path = watchmen_path.join("watchmen.sock");

    let path = std::env::current_dir()?.join("guard");
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
            let path = std::path::Path::new(&home_path);
            let path = path.join("guard.pid");
            // 创建文件
            let mut file = std::fs::File::create(path.clone())?;
            // 写入 pid
            file.write_all(pid.to_string().as_bytes())?;

            tokio::spawn(async move {
                child.wait().await.unwrap();
                remove_file(path).unwrap_or_default();
            });
            Ok(entity::Response::ok(format!("Start guard pid: {}", pid)))
        }
        None => Ok(entity::Response::err("watchmen: failed to start daemon")),
    }
}

async fn terminated_guard_daemon(
    _args: &[String],
    home_path: String,
) -> Result<entity::Response, Box<dyn Error>> {
    let options: HashMap<std::string::String, entity::Opt> = HashMap::new();
    let req = entity::Request {
        name: "terminated".to_string(),
        command: entity::Command {
            name: "terminated".to_string(),
            options: options,
            args: vec![],
        },
    };
    let res = socket::request(&req, home_path.clone()).await?;
    if res.code == 10 {
        let path = std::path::Path::new(&home_path).join("daemon.pid");
        remove_file(path).unwrap_or_default();
    }
    Ok(res)
}
