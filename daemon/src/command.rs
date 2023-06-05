// pub mod add;
// pub mod exit;
// pub mod list;
// pub mod pause;
// pub mod restart;
// pub mod resume;
// pub mod run;
// pub mod start;
// pub mod stop;

use std::error::Error;

use common::handle::{Command, Request, Response};
use tracing::info;

use crate::global;

pub async fn handle_exec(request: Request) -> Result<Response, Box<dyn Error>> {
    let req = request.clone();
    info!("Receive request: {:?}", req);
    let r = match request.command {
        Command::Run(task) => global::run(task).await,
        Command::Add(task) => global::add(task).await,
        Command::Start(name) => global::start(name).await,
        Command::Stop(name) => global::stop(name).await,
        Command::Remove(name) => global::remove(name).await,
        Command::Write(name, data) => global::write(name, data).await,
        Command::List(condition) => global::list(condition).await,
    };
    match r {
        Ok(res) => {
            info!("Request success: {:?}", req);
            Ok(res)
        }
        Err(e) => {
            info!("Request failed: {:?}, {}", req, e);
            Ok(Response::failed(e.to_string()))
        }
    }
}
