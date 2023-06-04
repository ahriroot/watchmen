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

use common::handle::{Body, Command, Request, Response};

use crate::global;

pub async fn handle_exec(request: Request) -> Result<Response<String>, Box<dyn Error>> {
    let r: Result<(), Box<dyn Error>> = match (request.command, request.body) {
        (Command::Run, Body::Task(task)) => global::run(task).await,
        (Command::Start, Body::TaskFlag(name)) => global::start(name).await,
        (Command::Stop, Body::TaskFlag(name)) => global::stop(name).await,
        _ => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "command not found",
        ))),
    };
    match r {
        Ok(_) => Ok(Response::<String> {
            code: 10000,
            msg: "Success".to_string(),
            data: None,
        }),
        Err(e) => Ok(Response::<String> {
            code: 50000,
            msg: "Failed".to_string(),
            data: Some(e.to_string()),
        }),
    }
}
