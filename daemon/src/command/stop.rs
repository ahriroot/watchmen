use common::handle::Body;
use std::error::Error;

use crate::global;

pub async fn stop_task(body: TaskFlag) -> Result<(), Box<dyn Error>> {
    match body {
        Body::Task(task) => global::start(task.name).await,
        Body::TaskFlag(_flag) => Ok(()),
    }
}
