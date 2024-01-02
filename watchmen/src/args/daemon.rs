use std::{error::Error, io::Write, process::Stdio};

use crate::common::config::Config;
use tokio::process::Command;

pub fn daemon(config: Config) -> Result<(), Box<dyn Error>> {
    println!("{:#?}", config);

    let mut child = Command::new("path");

    let child = if let Some(stdout) = &config.watchmen.stdout {
        let stdout_path = std::path::Path::new(stdout);
        let stdout_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(stdout_path)
            .unwrap();
        let stdout = Stdio::from(stdout_file);
        child.stdout(stdout)
    } else {
        &mut child
    };
    let child = if let Some(stderr) = &config.watchmen.stderr {
        let stderr_path = std::path::Path::new(stderr);
        let stderr_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(stderr_path)
            .unwrap();
        let stderr = Stdio::from(stderr_file);
        child.stderr(stderr)
    } else {
        child
    };

    let child = child.spawn()?;

    let result = child.id();

    match result {
        Some(pid) => {
            let path = config.watchmen.pid.unwrap();
            let mut file = std::fs::File::create(path.clone())?;
            file.write_all(pid.to_string().as_bytes())?;
            Ok(())
        }
        None => Err("()".into()),
    }
}
