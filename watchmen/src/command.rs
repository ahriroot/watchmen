pub mod list;
pub mod run;
pub mod stop;

use std::error::Error;
use std::io::Write;
use std::process::Stdio;

use crate::command;
use crate::const_exit_code::ExitCode;

const HTLP: &str = r#"Usage: watchmen [OPTION...] [SECTION] PAGE...
  -h, --help     display this help and exit
  -v, --version  output version information and exit

  run            start command
    -h, --help   display this help of 'run' command
    run `watchmen run -h` for more information

  stop           stop command
    -h, --help   display this help of 'stop' command
    run `watchmen stop -h` for more information

  list           list command
    -h, --help   display this help of 'list' command
    run `watchmen list -h` for more information

Report bugs to ahriknow@ahriknow.com."#;
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
        "stop" => command::stop::run(&args[2..]).await?,
        "list" => command::list::run(&args[2..]).await?,
        "-d" | "--daemon" => start_daemon(&args[2..]).await?,
        "-t" | "--terminated" => terminated_daemon(&args[2..]).await?,
        _ => {
            let err: String = format!("watchmen: invalid command '{}'", args[1]);
            return Err(err.into());
        }
    };

    Ok(exit_code)
}

async fn start_daemon(args: &[String]) -> Result<ExitCode, Box<dyn Error>> {
    if args.len() < 1 {
        eprintln!("watchmen: missing command");
        return Ok(ExitCode::ERROR);
    }

    let path = std::env::current_dir()?.join("daemon");
    // 子进程
    let child = std::process::Command::new(path)
        .arg(args[0].as_str())
        .stdout(Stdio::null())
        .spawn()?;

    // 保存 pid
    let pid = child.id();

    let watchmen_path =
        std::env::var("WATCHMEN_PATH").unwrap_or_else(|_| "/tmp/watchmen".to_string());
    let path = std::path::Path::new(&watchmen_path);
    let path = path.join("watchmen.pid");
    // 创建文件
    let mut file = std::fs::File::create(path)?;
    // 写入 pid
    file.write_all(pid.to_string().as_bytes())?;

    println!("Start daemon: {}", child.id());
    Ok(ExitCode::SUCCESS)
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
    let pid = std::fs::read_to_string(path)?;
    let pid = pid.parse::<i32>()?;
    let res = unsafe { kill(pid, signal) };
    if res == 0 {
        println!("Terminated daemon of pid: {}", pid);
    } else if res == -1 {
        println!("Daemon process not exists: {}", pid);
        return Ok(ExitCode::ERROR);
    } else {
        println!("Terminated daemon error with code: {}", res);
        return Ok(ExitCode::ERROR);
    }
    Ok(ExitCode::SUCCESS)
}
