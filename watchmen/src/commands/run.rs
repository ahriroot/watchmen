use common::{
    arg::AddArgs,
    config::Config,
    handle::{Command, Request},
    task::Task,
};
use regex::Regex;
use std::{error::Error, path::Path};

use crate::{
    engine::send,
    utils::{print_result, recursive_search_files},
};

pub async fn run(args: AddArgs, config: Config) -> Result<(), Box<dyn Error>> {
    let requests = if let Some(path) = args.path {
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
                let ts = Task::from_ini(path)?;
                for task in ts.task {
                    let request: Request = Request {
                        command: Command::Add(task),
                    };
                    reqs.push(request);
                }
            } else if path.is_file() && path.extension().unwrap().to_str().unwrap() == "toml" {
                let ts = Task::from_toml(path)?;
                for task in ts.task {
                    let request: Request = Request {
                        command: Command::Add(task),
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
        let mut reqs = Vec::new();
        for task in ts.task {
            let request: Request = Request {
                command: Command::Add(task),
            };
            reqs.push(request);
        }
        reqs
    } else {
        return Err(Box::from(format!(
            "Please specify a configuration file or a folder"
        )));
    };
    print_result(send(config.clone(), requests).await?).await;
    Ok(())
}
