use std::{error::Error, path::Path};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

use crate::entity;

pub async fn request(request: &entity::Request) -> Result<entity::Response, Box<dyn Error>> {
    // 获取环境变量,默认为 /tmp/watchmen.sock
    let watchmen_path =
        std::env::var("WATCHMEN_PATH").unwrap_or_else(|_| "/tmp/watchmen".to_string());
    // 判断文件夹是否存在
    let path = Path::new(&watchmen_path);
    if !path.exists() {
        std::fs::create_dir_all(path)?;
        return Err("socket file not exists".into());
    }
    // 判断是不是文件夹
    if !path.is_dir() {
        return Err("socket file is not a directory".into());
    }

    let socket_path: &Path = &path.join("watchmen.sock");
    if !socket_path.exists() {
        return Err("socket file not found".into());
    }
    let mut stream = UnixStream::connect(socket_path).await?;
    // Send request
    let buf = serde_json::to_vec(request)?;
    stream.write_all(&buf).await?;
    // 接收数据
    let mut buf: [u8; 1024] = [0; 1024];
    let n = stream.read(&mut buf).await?;
    let res: entity::Response = serde_json::from_slice(&buf[..n])?;
    Ok(res)
}
