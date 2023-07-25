use colored::Colorize;
use common::{
    arg::ListArgs,
    config::Config,
    handle::{Command, Request, Response, Status},
    task::TaskFlag,
};
use regex::Regex;
use std::{error::Error, path::Path};

use crate::{
    engine::send,
    utils::{print_result as pr, recursive_search_files},
};

pub async fn list(args: ListArgs, config: Config) -> Result<(), Box<dyn Error>> {
    let requests = if let Some(path) = args.path {
        let mat;
        if let Some(matc) = args.pattern {
            // 优先使用命令行参数
            mat = matc;
        } else if let Some(matc) = config.watchmen.mat.clone() {
            // 其次使用配置文件参数
            mat = matc;
        } else {
            // 最后使用默认参数
            mat = String::from(r"^.*\.(toml|ini|json)$");
        }
        let regex: Regex = Regex::new(&mat).unwrap();
        let mut matched_files = Vec::new();
        recursive_search_files(&path, &regex, &mut matched_files);

        let mut reqs = Vec::new();

        for file in matched_files {
            let path = Path::new(&file);
            if path.is_file() && path.extension().unwrap().to_str().unwrap() == "ini" {
                let ts = TaskFlag::from_ini(path)?;
                for tf in ts {
                    let request: Request = Request {
                        command: Command::List(Some(tf)),
                    };
                    reqs.push(request);
                }
            } else if path.is_file() && path.extension().unwrap().to_str().unwrap() == "toml" {
                let ts = TaskFlag::from_toml(path)?;
                for tf in ts {
                    let request: Request = Request {
                        command: Command::List(Some(tf)),
                    };
                    reqs.push(request);
                }
            } else if path.is_file() && path.extension().unwrap().to_str().unwrap() == "json" {
                // tasks.push(Task::from_json(path)?);
                // TODO: 读取 JSON 格式的配置文件
            } else {
                return Err(Box::from(format!(
                    "File {} is not a TOML or INI or JSON file",
                    path.to_str().unwrap()
                )));
            }
        }
        reqs
    } else if let Some(file) = args.config {
        let path = Path::new(&file);
        let tfs = if path.is_file() && path.extension().unwrap().to_str().unwrap() == "ini" {
            TaskFlag::from_ini(path)?
        } else if path.is_file() && path.extension().unwrap().to_str().unwrap() == "toml" {
            TaskFlag::from_toml(path)?
        } else if path.is_file() && path.extension().unwrap().to_str().unwrap() == "json" {
            // TODO: 读取 JSON 格式的配置文件
            return Err(Box::from(format!(
                "File {} is not a TOML or INI or JSON file",
                path.to_str().unwrap()
            )));
        } else {
            return Err(Box::from(format!(
                "File {} is not a TOML or INI or JSON file",
                path.to_str().unwrap()
            )));
        };
        let mut reqs = Vec::new();
        for tf in tfs {
            let request: Request = Request {
                command: Command::List(Some(tf)),
            };
            reqs.push(request);
        }
        reqs
    } else if let Some(id) = args.id {
        if args.name.is_some() {
            return Err(Box::from("Cannot use '--id' and '--name' at the same time"));
        }
        let request: Request = Request {
            command: Command::List(Some(TaskFlag {
                id,
                name: "".to_string(),
                mat: args.mat,
            })),
        };
        vec![request]
    } else if let Some(name) = args.name {
        let request: Request = Request {
            command: Command::List(Some(TaskFlag {
                id: 0,
                name,
                mat: args.mat,
            })),
        };
        vec![request]
    } else {
        let request: Request = Request {
            command: Command::List(None),
        };
        vec![request]
    };
    if args.less {
        print_result_less(send(config.clone(), requests).await?).await;
    } else if args.more {
        print_result_more(send(config.clone(), requests).await?).await;
    } else {
        print_result(send(config.clone(), requests).await?).await;
    }
    Ok(())
}

pub async fn print_result(res: Vec<Response>) {
    let mut status: Vec<Status> = Vec::new();
    for r in res {
        if r.code != 10000 {
            pr(vec![r]).await;
            return;
        }
        if let Some(data) = r.data {
            match data {
                common::handle::Data::None => {}
                common::handle::Data::String(_) => {}
                common::handle::Data::Status(s) => {
                    for i in &s {
                        status.push(i.clone());
                    }
                }
            }
        }
    }

    let mut total = 0;
    let mut total_added = 0;
    let mut total_running = 0;
    let mut total_stopped = 0;

    let mut column_id = Vec::new();
    column_id.push("ID".bold());

    let mut column_name = Vec::new();
    column_name.push("Name".bold());

    let mut column_status = Vec::new();
    column_status.push("Status".bold());

    let mut column_command = Vec::new();
    column_command.push("Command".bold());

    let mut column_pid = Vec::new();
    column_pid.push("Pid".bold());

    let mut column_code = Vec::new();
    column_code.push("ExitCode".bold());

    let mut column_type = Vec::new();
    column_type.push("Type".bold());

    for s in status {
        total += 1;
        column_id.push(s.id.to_string().italic());
        column_name.push(s.name.normal());
        match s.status {
            Some(t) => match t.as_str() {
                "added" => {
                    total_added += 1;
                    column_status.push(t.magenta())
                }
                "running" => {
                    total_running += 1;
                    column_status.push(t.green())
                }
                "stopped" => {
                    total_stopped += 1;
                    column_status.push(t.red())
                }
                "auto restart" => column_status.push(t.truecolor(128, 128, 128)),
                "waiting" => column_status.push(t.blue()),
                "interval" => column_status.push(t.cyan()),
                _ => column_status.push(t.normal()),
            },
            None => column_status.push("".normal()),
        }
        column_command.push(s.command.split("/").last().unwrap().normal());
        match s.pid {
            Some(t) => column_pid.push(t.to_string().normal()),
            None => column_pid.push("".normal()),
        }
        match s.code {
            Some(t) => column_code.push(t.to_string().normal()),
            None => column_code.push("".normal()),
        }
        match s.task_type {
            common::task::TaskType::Scheduled(_) => column_type.push("Scheduled".normal()),
            common::task::TaskType::Async(_) => column_type.push("Async".normal()),
            common::task::TaskType::Periodic(_) => column_type.push("Periodic".normal()),
            common::task::TaskType::None => column_type.push("".normal()),
        }
    }
    let max_id = column_id.iter().map(|s| s.len()).max().unwrap();
    let max_name = column_name.iter().map(|s| s.len()).max().unwrap();
    let max_status = column_status.iter().map(|s| s.len()).max().unwrap();
    let max_command = column_command.iter().map(|s| s.len()).max().unwrap();
    let max_pid = column_pid.iter().map(|s| s.len()).max().unwrap();
    let max_code = column_code.iter().map(|s| s.len()).max().unwrap();
    let max_type = column_type.iter().map(|s| s.len()).max().unwrap();

    let max_sum = max_id
        + max_name
        + max_status
        + max_command
        + max_pid
        + max_code
        + max_type
        + 3 * (7 - 1)
        + 4;

    for i in 0..column_id.len() {
        println!("{:-<max_sum$}", "", max_sum = max_sum);
        println!(
            "| {: <max_id$} | {: <max_name$} | {: <max_status$} | {: <max_command$} | {: <max_pid$} | {: <max_code$} | {: <max_type$} |",
            column_id[i],
            column_name[i],
            column_status[i],
            column_command[i],
            column_pid[i],
            column_code[i],
            column_type[i],
            max_id = max_id,
            max_name = max_name,
            max_status = max_status,
            max_command = max_command,
            max_pid = max_pid,
            max_code = max_code,
            max_type = max_type,
        );
    }
    println!("{:-<max_sum$}", "", max_sum = max_sum);
    println!(
        "{} Total: {} running, {} stopped, {} added, {} waiting, {} interval",
        total.to_string().bold().yellow(),
        total_running.to_string().green(),
        total_stopped.to_string().red(),
        total_added.to_string().magenta(),
        0.to_string().blue(),
        0.to_string().cyan(),
    );
}

pub async fn print_result_more(res: Vec<Response>) {
    let mut status: Vec<Status> = Vec::new();
    for r in res {
        if r.code != 10000 {
            pr(vec![r]).await;
            return;
        }
        if let Some(data) = r.data {
            match data {
                common::handle::Data::None => {}
                common::handle::Data::String(_) => {}
                common::handle::Data::Status(s) => {
                    for i in &s {
                        status.push(i.clone());
                    }
                }
            }
        }
    }

    let mut total = 0;
    let mut total_added = 0;
    let mut total_running = 0;
    let mut total_stopped = 0;

    let mut column_id = Vec::new();
    column_id.push("ID".bold());

    let mut column_name = Vec::new();
    column_name.push("Name".bold());

    let mut column_status = Vec::new();
    column_status.push("Status".bold());

    let mut column_command = Vec::new();
    column_command.push("Command".bold());

    let mut column_args = Vec::new();
    column_args.push("Args".bold());

    let mut column_pid = Vec::new();
    column_pid.push("Pid".bold());

    let mut column_code = Vec::new();
    column_code.push("ExitCode".bold());

    let mut column_type = Vec::new();
    column_type.push("Type".bold());

    for s in status {
        total += 1;
        column_id.push(s.id.to_string().italic());
        column_name.push(s.name.normal());
        match s.status {
            Some(t) => match t.as_str() {
                "added" => {
                    total_added += 1;
                    column_status.push(t.magenta())
                }
                "running" => {
                    total_running += 1;
                    column_status.push(t.green())
                }
                "stopped" => {
                    total_stopped += 1;
                    column_status.push(t.red())
                }
                "auto restart" => column_status.push(t.truecolor(128, 128, 128)),
                "waiting" => column_status.push(t.blue()),
                "interval" => column_status.push(t.cyan()),
                _ => column_status.push(t.normal()),
            },
            None => column_status.push("".normal()),
        }
        column_command.push(s.command.normal());
        column_args.push(s.args.join(" ").normal());
        match s.pid {
            Some(t) => column_pid.push(t.to_string().normal()),
            None => column_pid.push("".normal()),
        }
        match s.code {
            Some(t) => column_code.push(t.to_string().normal()),
            None => column_code.push("".normal()),
        }
        match s.task_type {
            common::task::TaskType::Scheduled(_) => column_type.push("Scheduled".normal()),
            common::task::TaskType::Async(_) => column_type.push("Async".normal()),
            common::task::TaskType::Periodic(_) => column_type.push("Periodic".normal()),
            common::task::TaskType::None => column_type.push("".normal()),
        }
    }
    let max_id = column_id.iter().map(|s| s.len()).max().unwrap();
    let max_name = column_name.iter().map(|s| s.len()).max().unwrap();
    let max_status = column_status.iter().map(|s| s.len()).max().unwrap();
    let max_command = column_command.iter().map(|s| s.len()).max().unwrap();
    let max_args = column_args.iter().map(|s| s.len()).max().unwrap();
    let max_pid = column_pid.iter().map(|s| s.len()).max().unwrap();
    let max_code = column_code.iter().map(|s| s.len()).max().unwrap();
    let max_type = column_type.iter().map(|s| s.len()).max().unwrap();

    let max_sum = max_id
        + max_name
        + max_status
        + max_command
        + max_args
        + max_pid
        + max_code
        + max_type
        + 3 * (8 - 1)
        + 4;

    for i in 0..column_id.len() {
        println!("{:-<max_sum$}", "", max_sum = max_sum);
        println!(
            "| {: <max_id$} | {: <max_name$} | {: <max_status$} | {: <max_command$} | {: <max_args$} | {: <max_pid$} | {: <max_code$} | {: <max_type$} |",
            column_id[i],
            column_name[i],
            column_status[i],
            column_command[i],
            column_args[i],
            column_pid[i],
            column_code[i],
            column_type[i],
            max_id = max_id,
            max_name = max_name,
            max_status = max_status,
            max_command = max_command,
            max_args = max_args,
            max_pid = max_pid,
            max_code = max_code,
            max_type = max_type,
        );
    }
    println!("{:-<max_sum$}", "", max_sum = max_sum);
    println!(
        "{} Total: {} running, {} stopped, {} added, {} waiting, {} interval",
        total.to_string().bold().yellow(),
        total_running.to_string().green(),
        total_stopped.to_string().red(),
        total_added.to_string().magenta(),
        0.to_string().blue(),
        0.to_string().cyan(),
    );
}

pub async fn print_result_less(res: Vec<Response>) {
    let mut status: Vec<Status> = Vec::new();
    for r in res {
        if r.code != 10000 {
            pr(vec![r]).await;
            return;
        }
        if let Some(data) = r.data {
            match data {
                common::handle::Data::None => {}
                common::handle::Data::String(_) => {}
                common::handle::Data::Status(s) => {
                    for i in &s {
                        status.push(i.clone());
                    }
                }
            }
        }
    }

    let mut total = 0;
    let mut total_added = 0;
    let mut total_running = 0;
    let mut total_stopped = 0;

    let mut column_id = Vec::new();
    column_id.push("ID".bold());

    let mut column_name = Vec::new();
    column_name.push("Name".bold());

    let mut column_status = Vec::new();
    column_status.push("Status".bold());

    for s in status {
        total += 1;
        column_id.push(s.id.to_string().italic());
        column_name.push(s.name.normal());
        match s.status {
            Some(t) => match t.as_str() {
                "added" => {
                    total_added += 1;
                    column_status.push(t.magenta())
                }
                "running" => {
                    total_running += 1;
                    column_status.push(t.green())
                }
                "stopped" => {
                    total_stopped += 1;
                    column_status.push(t.red())
                }
                "auto restart" => column_status.push(t.truecolor(128, 128, 128)),
                "waiting" => column_status.push(t.blue()),
                "interval" => column_status.push(t.cyan()),
                _ => column_status.push(t.normal()),
            },
            None => column_status.push("".normal()),
        }
    }
    let max_id = column_id.iter().map(|s| s.len()).max().unwrap();
    let max_name = column_name.iter().map(|s| s.len()).max().unwrap();
    let max_status = column_status.iter().map(|s| s.len()).max().unwrap();

    let max_sum = max_id + max_name + max_status + 3 * (4 - 1) + 1;

    for i in 0..column_id.len() {
        println!("{:-<max_sum$}", "", max_sum = max_sum);
        println!(
            "| {: <max_id$} | {: <max_name$} | {: <max_status$} |",
            column_id[i],
            column_name[i],
            column_status[i],
            max_id = max_id,
            max_name = max_name,
            max_status = max_status,
        );
    }
    println!("{:-<max_sum$}", "", max_sum = max_sum);
    println!(
        "{} Total: {} running, {} stopped, {} added, {} waiting, {} interval",
        total.to_string().bold().yellow(),
        total_running.to_string().green(),
        total_stopped.to_string().red(),
        total_added.to_string().magenta(),
        0.to_string().blue(),
        0.to_string().cyan(),
    );
}
