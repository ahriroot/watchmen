// pub mod exit;
pub mod add;
pub mod list;
pub mod pause;
pub mod reload;
pub mod remove;
pub mod restart;
pub mod resume;
pub mod run;
pub mod start;
pub mod stop;

use common::arg::{AddArgs, Commands, FlagArgs};
use common::config::Config;
use common::task::{Task, TaskFlag};
use regex::Regex;
use std::error::Error;
use std::path::Path;

use crate::utils::recursive_search_files;

pub async fn handle_exec(commands: Commands, config: Config) -> Result<(), Box<dyn Error>> {
    match commands {
        Commands::Run(args) => self::run::run(args, config).await?,
        Commands::Add(args) => self::add::add(args, config).await?,
        Commands::Reload(args) => self::reload::reload(args, config).await?,
        Commands::Start(args) => self::start::start(args, config).await?,
        Commands::Restart(args) => self::restart::restart(args, config).await?,
        Commands::Stop(args) => self::stop::stop(args, config).await?,
        Commands::Remove(args) => self::remove::remove(args, config).await?,
        Commands::Pause(args) => self::pause::pause(args, config).await?,
        Commands::Resume(args) => self::resume::resume(args, config).await?,
        Commands::List(args) => self::list::list(args, config).await?,
    }
    Ok(())
}

pub async fn taskflag_to_request(
    args: FlagArgs,
    config: Config,
) -> Result<Vec<TaskFlag>, Box<dyn Error>> {
    let taskflags = if let Some(path) = args.path {
        let mat;
        if let Some(matc) = args.regex {
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

        let mut tfs = Vec::new();

        for file in matched_files {
            let path = Path::new(&file);
            if path.is_file() && path.extension().unwrap().to_str().unwrap() == "ini" {
                let ts = TaskFlag::from_ini(path)?;
                for tf in ts {
                    tfs.push(tf);
                }
            } else if path.is_file() && path.extension().unwrap().to_str().unwrap() == "toml" {
                let ts = TaskFlag::from_toml(path)?;
                for tf in ts {
                    tfs.push(tf);
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
        tfs
    } else if let Some(file) = args.config {
        let path = Path::new(&file);
        let ts = if path.is_file() && path.extension().unwrap().to_str().unwrap() == "ini" {
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
        let mut tfs = Vec::new();
        for tf in ts {
            if args.name.is_some() && tf.name.is_some() {
                if args.name.as_ref().unwrap() != tf.name.as_ref().unwrap() {
                    continue;
                }
            }
            tfs.push(tf);
        }
        tfs
    } else {
        let ts = TaskFlag::from_args(args)?;
        let mut tfs = Vec::new();
        for tf in ts {
            tfs.push(tf);
        }
        tfs
    };
    Ok(taskflags)
}

pub async fn task_to_request(args: AddArgs, config: Config) -> Result<Vec<Task>, Box<dyn Error>> {
    let tasks = if let Some(path) = args.path {
        let mat;
        if let Some(matc) = args.regex {
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

        let mut tasks = Vec::new();

        for file in matched_files {
            let path = Path::new(&file);
            if path.is_file() && path.extension().unwrap().to_str().unwrap() == "ini" {
                let ts = Task::from_ini(path)?;
                for task in ts.task {
                    if let Some(name) = &args.name {
                        if &task.name != name {
                            continue;
                        }
                    }
                    tasks.push(task);
                }
            } else if path.is_file() && path.extension().unwrap().to_str().unwrap() == "toml" {
                let ts = Task::from_toml(path)?;
                for task in ts.task {
                    if let Some(name) = &args.name {
                        if &task.name != name {
                            continue;
                        }
                    }
                    tasks.push(task);
                }
            } else if path.is_file() && path.extension().unwrap().to_str().unwrap() == "json" {
                // tasks.push(Task::from_json(path)?);
                // TODO: 读取 JSON 格式的配置文件
            } else {
                return Err(Box::from(format!(
                    "File [{}] is not a TOML or INI or JSON file",
                    path.to_str().unwrap()
                )));
            }
        }
        tasks
    } else if let Some(file) = args.config {
        let path = Path::new(&file);
        let ts = if path.is_file() && path.extension().unwrap().to_str().unwrap() == "ini" {
            Task::from_ini(path)?
        } else if path.is_file() && path.extension().unwrap().to_str().unwrap() == "toml" {
            Task::from_toml(path)?
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
        ts.task
    } else {
        Task::from_args(args)?.task
    };
    Ok(tasks)
}
