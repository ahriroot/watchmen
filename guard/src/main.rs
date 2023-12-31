use std::{
    env,
    error::Error,
    fs::remove_file,
    io::Write,
    path::Path,
    process::{exit, Stdio},
};

use tokio::process::Command;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Missing path argument.");
        exit(0);
    }

    let valid_args = args[1..].to_vec();

    loop {
        match start_daemon(&valid_args).await {
            Ok(res_code) => match res_code {
                10000 => {
                    println!("Daemon terminated. {}", res_code);
                    exit(0);
                }
                50000 => {
                    println!("Unexpected termination of program. {}", res_code);
                }
                _ => {
                    eprintln!("Unexpected program error. {}", res_code);
                }
            },
            Err(err) => {
                eprintln!("{}", err);
                break;
            }
        }
    }
}

async fn start_daemon(args: &[String]) -> Result<i32, Box<dyn Error>> {
    let now = chrono::Local::now().format("%Y%m%d").to_string();
    let path = Path::new(&args[1]);
    let path_daemon_stdout = path.join(format!("daemon_stdout_{}.log", now));
    let path_daemon_stderr = path.join(format!("daemon_stderr_{}.log", now));
    let file_stdout = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path_daemon_stdout.clone())?;
    let file_stderr = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path_daemon_stderr.clone())?;
    let stdout = Stdio::from(file_stdout);
    let stderr = Stdio::from(file_stderr);

    let path_daemon = std::env::current_dir()?.join("daemon");
    // 子进程
    let mut child = Command::new(path_daemon)
        .args(args)
        .stdout(stdout)
        .stderr(stderr)
        .spawn()?;

    let result = child.id();

    match result {
        Some(pid) => {
            // 保存 pid
            let path_watchmen = path.join("daemon.pid");
            // 创建文件
            let mut file = std::fs::File::create(path_watchmen.clone())?;
            // 写入 pid
            file.write_all(pid.to_string().as_bytes())?;

            let res = child.wait().await.unwrap();
            remove_file(path_watchmen).unwrap_or_default();

            if res.success() {
                let path_guard = path.join("guard.pid");
                remove_file(path_guard).unwrap_or_default();
                return Ok(10000);
            } else {
                return Ok(50000);
            }
        }
        None => Ok(-1),
    }
}
