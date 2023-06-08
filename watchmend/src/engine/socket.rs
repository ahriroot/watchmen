use common::{
    config::Config,
    handle::{Request, Response},
};
use tracing::{error, info};

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
            info!("socket server listen on {}:{}", host, port);

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

    match serde_json::from_slice::<Vec<Request>>(&buf) {
        Ok(requests) => {
            let mut responses: Vec<Response> = Vec::new();
            for request in requests {
                match command::handle_exec(request).await {
                    Ok(response) => {
                        responses.push(response);
                    }
                    Err(e) => {
                        let response = Response::failed(e.to_string());
                        responses.push(response);
                    }
                }
            }

            writer.write_all(&serde_json::to_vec(&responses)?).await?;
            Ok(())
        }
        Err(e) => {
            let response = Response::failed(e.to_string());
            writer.write_all(&serde_json::to_vec(&[response])?).await?;
            Ok(())
        }
    }
}
