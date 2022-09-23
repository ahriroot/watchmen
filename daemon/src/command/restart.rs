use std::error::Error;

use crate::entity;

use super::start;
use super::stop;

pub async fn restart_task(command: entity::Command) -> Result<entity::Response, Box<dyn Error>> {
    let res1 = stop::stop_task(command.clone()).await?;
    if res1.code == 10000 {
        let res2 = start::start_task(command).await?;
        if res2.code == 10000 {
            return Ok(entity::Response {
                code: 10000,
                msg: "restart success".to_string(),
                data: None,
            });
        } else {
            return Ok(res2);
        }
    } else {
        return Ok(res1);
    }
}
