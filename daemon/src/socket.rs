use std::{error::Error, fs::remove_file, path::Path, process::exit};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
};

use crate::{command, entity};

async fn handle_connection(mut stream: UnixStream) -> Result<(), Box<dyn Error>> {
    let (mut reader, mut writer) = stream.split();
    let mut buf: [u8; 1024] = [0; 1024];
    let n = reader.read(&mut buf).await?;

    let req: entity::Request = serde_json::from_slice(&buf[..n])?;

    let res = command::handle_exec(req.command).await?;

    writer.write_all(&serde_json::to_vec(&res)?).await?;

    if res.code == 10 {
        let args: Vec<String> = std::env::args().collect();
        let path = args[1].clone();
        let sock_path = Path::new(&path);
        remove_file(sock_path).unwrap_or_default();
        exit(0);
    }
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
                eprintln!("failed to accept socket; error = {:?}", e);
            }
        }
    }
}
