use common::{config::Config, handle::Request};

use log::{error, info};
use std::{error::Error, process::exit};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    task::JoinHandle,
};

use crate::command;

static BUFFER_SIZE: usize = 1024;

pub async fn start(config: Config) -> JoinHandle<()> {
    tokio::spawn(async move {
        match run_socket(&config.socket.host, config.socket.port).await {
            Ok(_) => {
                info!("socket server exit");
            }
            Err(e) => {
                error!("socket server error: {}", e);
            }
        }
    })
}

pub async fn run_socket(host: &str, port: u16) -> Result<(), Box<dyn Error>> {
    // 创建监听器 / create a listener
    let listener = TcpListener::bind((host, port)).await;

    match listener {
        Ok(listener) => {
            info!("Listening\t{:?}", listener.local_addr().unwrap());

            loop {
                // 等待连接 / wait connection
                match listener.accept().await {
                    Ok((stream, _addr)) => {
                        // 处理连接 / handle connection
                        tokio::spawn(async move {
                            if let Err(e) = handle_connection(stream).await {
                                error!("failed to process connection: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("failed to accept socket; error = {:?}", e);
                    }
                }
            }
        }
        Err(e) => {
            error!("failed to bind socket; error = {:?}", e);
            exit(1);
        }
    }
}

async fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let (mut reader, mut writer) = stream.split();
    let mut buf: Vec<u8> = Vec::new();
    loop {
        let mut b = vec![0; BUFFER_SIZE];
        let n = reader.read(&mut b).await?;
        buf.extend_from_slice(&b[..n]);
        if n < 1024 {
            break;
        }
    }

    let request: Request = serde_json::from_slice(&buf)?;

    let response = command::handle_exec(request).await?;

    writer.write_all(&serde_json::to_vec(&response)?).await?;

    // TODO: 根据 response.code 停止守护进程
    // exit(0);

    Ok(())
}
