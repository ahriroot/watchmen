pub mod list;
pub mod run;
pub mod stop;

use std::error::Error;
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
    println!("child pid: {}", child.id());
    Ok(ExitCode::SUCCESS)
}
