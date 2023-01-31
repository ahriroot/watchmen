use colored::Colorize;
use std::{env, error::Error, fs, process::exit};

use watchmen::command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let watchmen_path = env::var("WATCHMEN_PATH").unwrap_or_else(|_| "/tmp/watchmen".to_string());

    let sock_path = std::path::Path::new(&watchmen_path);

    if !sock_path.exists() {
        match fs::create_dir_all(sock_path) {
            Ok(_) => {
                println!("{} {}", "Created watchmen path: ".green(), watchmen_path);
            }
            Err(err) => {
                eprintln!("{} {}", "Error creating watchmen path: ".red(), err);
                exit(1);
            }
        }
    }

    let stdout_path = sock_path.join("stdout/").clone();
    if !stdout_path.exists() {
        match fs::create_dir(stdout_path.clone()) {
            Ok(_) => {
                println!("{} {}", "Created stdout path: ".green(), stdout_path.display());
            }
            Err(err) => {
                eprintln!("{} {}", "Error creating stdout path: ".red(), err);
                exit(1);
            }
        }
    }

    // 命令行参数 / command line arguments
    let args: Vec<String> = std::env::args().collect();
    // 执行命令 / execute command
    let response = command::exec(args, watchmen_path).await;
    match response {
        Ok(res) => {
            let code;
            if res.code == 10 {
                println!("{}", res.msg.green());
                code = 0;
            } else if res.code >= 50000 {
                println!("{}", res.msg.blue());
                code = 1;
            } else if res.code >= 40000 {
                println!("{}", res.msg.red());
                code = 1;
            } else if res.code >= 20000 {
                println!("{}", res.msg.yellow());
                code = 1;
            } else if res.code >= 10000 {
                println!("{}", res.msg.green());
                code = 0;
            } else {
                println!("{}", res.msg);
                code = 1;
            }
            exit(code);
        }
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }
}

#[cfg(test)]
pub mod tests {

    #[test]
    fn test_plugin() {
        unsafe {
            let lib = libloading::Library::new("/tmp/watchmen/plugins/libplugin.so").unwrap();
            let func: libloading::Symbol<unsafe extern "C" fn(t: i32) -> bool> =
                lib.get(b"func_plugin").unwrap();
            println!("PLUGIN: {:?}", func(1));
        }
        assert_eq!(1, 1);
    }
}
