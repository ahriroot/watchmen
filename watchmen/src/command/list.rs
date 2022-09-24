use chrono::{Local, TimeZone};
use colored::Colorize;
use std::{collections::HashMap, error::Error};

use crate::{
    entity::{self, Opt, Task},
    socket,
};

const LIST_HELP: &str = r#"Usage: watchmen list [OPTION...] ...
  -h, --help     display this help of 'list' command

  -n, --name     list tasks by name with regular expression
  -s, --status   list tasks by status
  -p, --pid      list tasks by pid

  -m, --more     list tasks with more details

Report bugs to ahriknow@ahriknow.com.""#;

pub async fn run(args: &[String]) -> Result<entity::Response, Box<dyn Error>> {
    let len = args.len();
    let mut more = false;
    if len == 1 {
        if args[0] == "-h" || args[0] == "--help" {
            return Ok(entity::Response::ok(LIST_HELP));
        } else if args[0] == "-m" || args[0] == "--more" {
            more = true;
        }
    }
    let mut options: HashMap<String, Opt> = HashMap::new();

    let mut args: Vec<String> = args.to_vec();
    while args.len() > 1 {
        if args[0] == "-n" || args[0] == "--name" {
            options.insert("name".to_string(), Opt::Str(args[1].clone()));
            args.remove(0);
            args.remove(0);
        } else if args[0] == "-s" || args[0] == "--status" {
            options.insert("status".to_string(), Opt::Str(args[1].clone()));
            args.remove(0);
            args.remove(0);
        } else if args[0] == "-p" || args[0] == "--pid" {
            let pid = args[1].parse::<u32>();
            match pid {
                Ok(p) => {
                    options.insert("pid".to_string(), Opt::U32(p));
                }
                Err(_) => {
                    return Ok(entity::Response::err(format!(
                        "Arg '{}' must be a number",
                        args[0]
                    )));
                }
            }
            args.remove(0);
            args.remove(0);
        } else if args[0] == "-m" || args[0] == "--more" {
            more = true;
            args.remove(0);
        } else {
            break;
        }
    }

    let req = entity::Request {
        name: "list".to_string(),
        command: entity::Command {
            name: "list".to_string(),
            options: options,
            args: args,
        },
    };
    let res = socket::request(&req).await?;
    if let Some(data) = res.data {
        match data {
            entity::Data::TaskList(tasks) => print_format(tasks, more).await,
            _ => {}
        }
    }
    Ok(entity::Response::ok("list success"))
}

async fn print_format(res: Vec<Task>, more: bool) {
    let sum_all = res.len();
    let len_id = res
        .iter()
        .map(|x| x.id.to_string().len())
        .max()
        .unwrap_or_else(|| 0);
    let len_name = res.iter().map(|x| x.name.len()).max().unwrap_or_else(|| 0);
    let len_status = res
        .iter()
        .map(|x| x.status.len())
        .max()
        .unwrap_or_else(|| 0);
    let len_pid = res
        .iter()
        .map(|x| x.pid.to_string().len())
        .max()
        .unwrap_or_else(|| 0);
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

    if more {
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
        let mut sum_added = 0;
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
            } else if status == "added  " {
                sum_added += 1;
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
            "{} Total: {} running, {} stopped, {} waiting, {} added",
            sum_all,
            sum_running.to_string().green().to_string(),
            sum_stopped.to_string().red().to_string(),
            sum_waiting.to_string().blue().to_string(),
            sum_added.to_string().to_string()
        );
    } else {
        let len_sum = len_id
            + len_name
            + len_status
            + len_pid
            + len_started_at
            + len_stopped_at
            + len_exit_code
            + 7 * 2
            + 8;
        println!("{:-<len_sum$}", "", len_sum = len_sum);
        println!(
            "| {: <len_id$} | {: <len_name$} | {: <len_status$} | {: <len_pid$} | {: <len_started_at$} | {: <len_stopped_at$} | {: <len_exit_code$} |",
            title_id, title_name, title_status, title_pid, title_started_at, title_stopped_at, title_exit_code,
            len_id = len_id, len_name = len_name, len_status = len_status, len_pid = len_pid, len_started_at = len_started_at, len_stopped_at = len_stopped_at, len_exit_code = len_exit_code
        );
        let mut sum_running = 0;
        let mut sum_stopped = 0;
        let mut sum_waiting = 0;
        let mut sum_added = 0;
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
            } else if status == "added  " {
                sum_added += 1;
            }
            let started_at = if task.started_at > 0 {
                Local
                    .timestamp_millis(task.started_at as i64)
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
                "| {: <len_id$} | {: <len_name$} | {: <len_status$} | {: <len_pid$} | {: <len_started_at$} | {: <len_stopped_at$} | {: <len_exit_code$} |",
                task.id, task.name, status, task.pid, started_at, stopped_at, exit_code,
                len_id = len_id, len_name = len_name, len_status = len_status, len_pid = len_pid, len_started_at = len_started_at, len_stopped_at = len_stopped_at, len_exit_code = len_exit_code
            );
            println!("{:-<len_sum$}", "", len_sum = len_sum);
        }
        println!(
            "{} Total: {} running, {} stopped, {} waiting, {} added",
            sum_all,
            sum_running.to_string().green().to_string(),
            sum_stopped.to_string().red().to_string(),
            sum_waiting.to_string().blue().to_string(),
            sum_added.to_string().to_string()
        );
    }
}
