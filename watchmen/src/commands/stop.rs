use common::{
    arg::FlagArgs,
    config::Config,
    handle::{Command, Request},
    task::TaskFlag,
};
use regex::Regex;
use std::{error::Error, path::Path};

use crate::{
    engine::send,
    utils::{print_result, recursive_search_files},
};

pub async fn stop(args: FlagArgs, config: Config) -> Result<(), Box<dyn Error>> {
    let requests = match args.path {
        Some(path) => {
            let mat;
            if let Some(matc) = args.mat {
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
                            command: Command::Stop(tf),
                        };
                        reqs.push(request);
                    }
                } else if path.is_file() && path.extension().unwrap().to_str().unwrap() == "toml" {
                    let ts = TaskFlag::from_toml(path)?;
                    for tf in ts {
                        let request: Request = Request {
                            command: Command::Stop(tf),
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
        }
        None => match args.config {
            Some(file) => {
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
                let mut reqs = Vec::new();
                for task in ts {
                    let request: Request = Request {
                        command: Command::Stop(task),
                    };
                    reqs.push(request);
                }
                reqs
            }
            None => match args.name {
                Some(name) => {
                    let request: Request = Request {
                        command: Command::Stop(TaskFlag { name }),
                    };
                    vec![request]
                }
                None => {
                    return Err(Box::from(format!(
                        "Please specify a configuration file or a folder"
                    )));
                }
            },
        },
    };
    print_result(send(config.clone(), requests).await?).await;
    Ok(())
}
