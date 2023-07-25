use common::{
    arg::AddArgs,
    config::Config,
    handle::{Command, Request, Response},
};
use std::error::Error;

use crate::{engine::send, utils::print_result};

use super::task_to_request;

pub async fn reload(args: AddArgs, config: Config) -> Result<(), Box<dyn Error>> {
    let tasks = task_to_request(args, config.clone()).await?;
    if tasks.is_empty() {
        print_result(vec![Response::wrong("No task to reload".to_string())]).await;
    } else {
        let mut requests = Vec::new();
        for task in tasks {
            requests.push(Request {
                command: Command::Reload(task),
            });
        }
        print_result(send(config, requests).await?).await;
    }
    Ok(())
}
