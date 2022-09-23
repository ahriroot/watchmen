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
    use colored::Colorize;
    use watchmen::entity::Task;

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
        let mut res: Vec<Task> = Vec::new();
        res.push(Task {
            id: 123456789,
            name: "python task".to_string(),
            command: "python".to_string(),
            args: [].to_vec(),
            status: "running".to_string(),
            pid: 0,
            created_at: 1663895588789,
            exited_at: 0,
            started_at: 1663895588789,
            stopped_at: 1663895588789,
            exit_code: 100,
        });
        res.push(Task {
            id: 123456789,
            name: "node task".to_string(),
            command: "node".to_string(),
            args: [].to_vec(),
            status: "waiting".to_string(),
            pid: 0,
            created_at: 1663895588789,
            exited_at: 0,
            started_at: 0,
            stopped_at: 1663895588789,
            exit_code: 100,
        });
        let sum_all = res.len();
        let len_id = res.iter().map(|x| x.id.to_string().len()).max().unwrap();
        let len_name = res.iter().map(|x| x.name.len()).max().unwrap();
        let len_status = res.iter().map(|x| x.status.len()).max().unwrap();
        let len_pid = res.iter().map(|x| x.pid.to_string().len()).max().unwrap();
        let len_created_at = 19;
        let len_started_at = 19;
        let len_exited_at = 19;
        let len_stopped_at = 19;
        let len_exit_code = 5;
        
        let title_id = "ID";
        let len_id = if len_id > title_id.len() {
            len_id
        } else {
            title_id.len()
        };
        let title_name = "NAME";
        let len_name = if len_name > title_name.len() {
            len_name
        } else {
            title_name.len()
        };
        let title_status = "STATUS";
        let len_status = if len_status > title_status.len() {
            len_status
        } else {
            title_status.len()
        };
        let title_pid = "PID";
        let len_pid = if len_pid > title_pid.len() {
            len_pid
        } else {
            title_pid.len()
        };
        let title_created_at = "CREATED_AT";
        let len_created_at = if len_created_at > title_created_at.len() {
            len_created_at
        } else {
            title_created_at.len()
        };
        let title_started_at = "STARTED_AT";
        let len_started_at = if len_started_at > title_started_at.len() {
            len_started_at
        } else {
            title_started_at.len()
        };
        let title_exited_at = "EXITED_AT";
        let len_exited_at = if len_exited_at > title_exited_at.len() {
            len_exited_at
        } else {
            title_exited_at.len()
        };
        let title_stopped_at = "STOPPED_AT";
        let len_stopped_at = if len_stopped_at > title_stopped_at.len() {
            len_stopped_at
        } else {
            title_stopped_at.len()
        };
        let title_exit_code = "EXIT_CODE";
        let len_exit_code = if len_exit_code > title_exit_code.len() {
            len_exit_code
        } else {
            title_exit_code.len()
        };

        let len_sum = len_id
            + len_name
            + len_status
            + len_pid
            + len_created_at
            + len_started_at
            + len_exited_at
            + len_stopped_at
            + len_exit_code
            + 9 * 2
            + 10;
        println!("{:-<len_sum$}", "", len_sum = len_sum);
        println!(
            "| {: <len_id$} | {: <len_name$} | {: <len_status$} | {: <len_pid$} | {: <len_created_at$} | {: <len_started_at$} | {: <len_exited_at$} | {: <len_stopped_at$} | {: <len_exit_code$} |",
            title_id, title_name, title_status, title_pid, title_created_at, title_started_at, title_exited_at, title_stopped_at, title_exit_code,
            len_id = len_id, len_name = len_name, len_status = len_status, len_pid = len_pid, len_created_at = len_created_at, len_started_at = len_started_at, len_exited_at = len_exited_at, len_stopped_at = len_stopped_at, len_exit_code = len_exit_code
        );
        let mut sum_running = 0;
        let mut sum_stopped = 0;
        let mut sum_waiting = 0;
        println!("{:-<len_sum$}", "", len_sum = len_sum);
        for task in res {
            let mut status = task.status;
            if status == "running" {
                status = status.green().to_string();
                sum_running += 1;
            } else if status == "stopped" {
                status = status.red().to_string();
                sum_stopped += 1;
            } else if status == "waiting" {
                status = status.blue().to_string();
                sum_waiting += 1;
            }
            let created_at = if task.created_at > 0 {
                Local
                    .timestamp_millis(task.created_at as i64)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            } else {
                "".to_string()
            };
            let started_at = if task.started_at > 0 {
                Local
                    .timestamp_millis(task.started_at as i64)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            } else {
                "".to_string()
            };
            let exited_at = if task.exited_at > 0 {
                Local
                    .timestamp_millis(task.exited_at as i64)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            } else {
                "".to_string()
            };
            let stopped_at = if task.stopped_at > 0 {
                Local
                    .timestamp_millis(task.stopped_at as i64)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            } else {
                "".to_string()
            };
            let exit_code = if task.exit_code != 100 {
                task.exit_code.to_string()
            } else {
                "".to_string()
            };
            println!(
                "| {: <len_id$} | {: <len_name$} | {: <len_status$} | {: <len_pid$} | {: <len_created_at$} | {: <len_started_at$} | {: <len_exited_at$} | {: <len_stopped_at$} | {: <len_exit_code$} |",
                task.id, task.name, status, task.pid, created_at, started_at, exited_at, stopped_at, exit_code,
                len_id = len_id, len_name = len_name, len_status = len_status, len_pid = len_pid, len_created_at = len_created_at, len_started_at = len_started_at, len_exited_at = len_exited_at, len_stopped_at = len_stopped_at, len_exit_code = len_exit_code
            );
            println!("{:-<len_sum$}", "", len_sum = len_sum);
        }
        println!(
            "{} Total: {} running, {} stopped, {} waiting",
            sum_all,
            sum_running.to_string().green().to_string(),
            sum_stopped.to_string().red().to_string(),
            sum_waiting.to_string().blue().to_string()
        );
    }
}
