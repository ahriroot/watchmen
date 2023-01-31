use std::{error::Error, path::Path};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

use crate::entity;

pub async fn request(request: &entity::Request, home_path: String) -> Result<entity::Response, Box<dyn Error>> {
    // 判断文件夹是否存在
    let path = Path::new(&home_path);

    let socket_path: &Path = &path.join("watchmen.sock");
    if !socket_path.exists() {
        return Err("Daemon is not running".into());
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
