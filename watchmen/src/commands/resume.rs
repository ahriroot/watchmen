use common::{
    arg::FlagArgs,
    config::Config,
    handle::{Command, Request, Response},
};
use std::error::Error;

use crate::{engine::send, utils::print_result};

use super::taskflag_to_request;

pub async fn resume(args: FlagArgs, config: Config) -> Result<(), Box<dyn Error>> {
    let taskflags = taskflag_to_request(args, config.clone()).await?;
    if taskflags.is_empty() {
        print_result(vec![Response::wrong("No task to resume".to_string())]).await;
    } else {
        let mut requests = Vec::new();
        for taskflag in taskflags {
            requests.push(Request {
                command: Command::Resume(taskflag),
            });
        }
        print_result(send(config, requests).await?).await;
    }
    Ok(())
}
