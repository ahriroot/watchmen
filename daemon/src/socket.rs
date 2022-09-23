use std::{
    error::Error,
    fs::remove_file,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
};

use crate::{
    command,
    entity::{self, Task},
};

async fn handle_exec(command: entity::Command) -> Result<entity::Response, Box<dyn Error>> {
    match command.name.as_str() {
        "run" => {
            let id = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            let task: Task = Task {
                id: id,
                name: command.name,
                command: command.args[0].clone(),
                args: command.args[1..].to_vec(),
                status: "running".to_string(),
                pid: 0,
                created_at: 0,
                started_at: 0,
                exited_at: 0,
                stopped_at: 0,
                exit_code: 100,
            };

            let result = command::run::run_task(task).await?;
            let res = entity::Response {
                code: result,
                msg: "success".to_string(),
                data: None,
            };
            return Ok(res);
        }
        "stop" => {
            if command.args.len() == 0 {
                let res = entity::Response {
                    code: 50000,
                    msg: "args error".to_string(),
                    data: None,
                };
                return Ok(res);
            } else {
                let name = command.args[0].clone();
                let result = command::stop::stop_task(name).await?;
                return Ok(result);
            }
        }
        "list" => {
            let result = command::list::list_tasks(command.args.clone()).await?;
            return Ok(result);
        }
        _ => {
            println!("unknown command: {:?}", command);
            Ok(entity::Response {
                code: 10000,
                msg: "success".to_string(),
                data: None,
            })
        }
    }
}

async fn handle_connection(mut stream: UnixStream) -> Result<(), Box<dyn Error>> {
    let (mut reader, mut writer) = stream.split();
    let mut buf: [u8; 1024] = [0; 1024];
    let n = reader.read(&mut buf).await?;

    let req: entity::Request = serde_json::from_slice(&buf[..n])?;

    let res = handle_exec(req.command).await?;

    writer.write_all(&serde_json::to_vec(&res)?).await?;
    Ok(())
}

pub async fn run_socket(path: &String) -> Result<(), Box<dyn Error>> {
    let sock_path = Path::new(path);
    remove_file(sock_path).unwrap_or_default();

    // 创建监听器 / create a listener
    let listener = UnixListener::bind(path).unwrap();
    println!("Listening on {:?}", listener.local_addr().unwrap());

    loop {
        // 等待连接 / wait connection
        match listener.accept().await {
            Ok((stream, _addr)) => {
                // 处理连接 / handle connection
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream).await {
                        eprintln!("failed to process connection: {}", e);
                    }
                });
            }
            Err(e) => {
                println!("failed to accept socket; error = {:?}", e);
            }
        }
    }
}
