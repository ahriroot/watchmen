use std::{fs::File, io::Write};

#[derive(Clone)]
pub enum Level {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

#[derive(Clone)]
pub struct Logger {
    level: Level,
    format: String,
    path: String,
    filename: String,
    max_size: u64,
}

impl Logger {
    pub fn new(
        level: Level,
        format: String,
        path: String,
        filename: String,
        max_size: u64,
    ) -> Self {
        Self {
            level,
            format,
            path,
            filename,
            max_size,
        }
    }

    fn f(&mut self) -> File {
        let file: File;
        let mut index = 1;
        loop {
            let log_file_name = format!(
                "{}_{}_{}.log",
                self.filename,
                chrono::Local::now().format("%Y%m%d").to_string(),
                index
            );
            let file_path = std::path::Path::new(&self.path).join(log_file_name);
            if file_path.exists() {
                let file_size = std::fs::metadata(file_path.clone()).unwrap().len();
                if file_size >= self.max_size {
                    index += 1;
                    continue;
                } else {
                    file = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(file_path.clone())
                        .unwrap();
                    break;
                }
            } else {
                file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(file_path.clone())
                    .unwrap();
                break;
            }
        }
        return file;
    }

    fn w(&mut self, msg: &str, level: Level) {
        let mut file = self.f();

        /*
         * [T] - 时间
         * [L] - 日志级别
         * [M] - 日志内容
         */
        let t = chrono::Local::now().format("[%Y-%m-%d %H:%M:%S]");
        let l = match level {
            Level::Debug => "[Debug]",
            Level::Info => "[Info]",
            Level::Warn => "[Warn]",
            Level::Error => "[Error]",
        };
        let message: String = self
            .format
            .replace("[T]", &t.to_string())
            .replace("[L]", l)
            .replace("[M]", &msg);

        let message = format!("{}\n", message);

        file.write_all(message.as_bytes()).unwrap();
    }

    pub fn debug(&mut self, msg: &str) {
        self.w(msg, Level::Debug);
    }

    pub fn info(&mut self, msg: &str) {
        match self.level {
            Level::Info => self.w(msg, Level::Info),
            Level::Warn => self.w(msg, Level::Info),
            Level::Error => self.w(msg, Level::Info),
            _ => return,
        }
    }

    pub fn warn(&mut self, msg: &str) {
        match self.level {
            Level::Warn => self.w(msg, Level::Warn),
            Level::Error => self.w(msg, Level::Warn),
            _ => return,
        }
    }

    pub fn error(&mut self, msg: &str) {
        match self.level {
            Level::Error => self.w(msg, Level::Error),
            _ => return,
        }
    }
}

#[macro_export]
macro_rules! log {
    ($logger:ident, $level:ident, $($arg:tt)*) => {
        $logger.$level(&format!($($arg)*));
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut logger = Logger::new(
            Level::Debug,
            "[T] [L] [M]".to_string(),
            "/tmp/watchmen/".to_string(),
            "test".to_string(),
            1024 * 1024 * 10,
        );
        log!(logger, debug, "debug{}", 1);
    }
}
