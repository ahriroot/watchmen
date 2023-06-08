use std::{error::Error, path::Path};

use common::handle::{Request, Response};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

pub async fn send(path: &str, requests: Vec<Request>) -> Result<Vec<Response>, Box<dyn Error>> {
    let path = Path::new(path);
    if !path.exists() {
        return Err(Box::from(format!(
            "Socket file {} not exists",
            path.to_str().unwrap()
        )));
    }
    let mut stream = UnixStream::connect(path).await.unwrap();

    let buf = serde_json::to_vec(&requests).unwrap();
    stream.write_all(&buf).await.unwrap();

    let mut buf: Vec<u8> = Vec::new();
    loop {
        let mut b = vec![0; 1024];
        let n = stream.read(&mut b).await?;
        buf.extend_from_slice(&b[..n]);
        if n < 1024 {
            break;
        }
    }

    let res: Vec<Response> = serde_json::from_slice(&buf).unwrap();
    Ok(res)
}
