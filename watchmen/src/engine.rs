use std::error::Error;

use common::{
    config::Config,
    handle::{Request, Response},
};

mod sock;

pub async fn send(config: Config, requests: Vec<Request>) -> Result<Vec<Response>, Box<dyn Error>> {
    match config.watchmen.engine.as_str() {
        "sock" => sock::send(config.sock.path.as_str(), requests).await,
        _ => Err("No engine found".into()),
    }
}
