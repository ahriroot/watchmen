use colored::Colorize;
use std::{env, error::Error, fs, process::exit};

use common::{arg::TaskArgs, task::Task, config::Config};
use watchmen::{args, commands, utils};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let clargs = TaskArgs::new();
    if clargs.version {
        println!(
            "{} {}",
            "Watchmen rust".green(),
            env!("CARGO_PKG_VERSION").green()
        );
        return Ok(());
    }

    // let input = "$HOME/dir";
    // println!("{:#?}", utils::get_with_home_path(input.to_string()));

    if let Some(path) = &clargs.generate {
        return args::generate(path);
    }

    let config: Config = Config::init(clargs.config.clone())?;

    if clargs.daemon {
        return args::daemon(config);
    }

    if let Some(commands) = clargs.commands {
        let config = utils::get_config(clargs.config)?;
        println!("{:#?}", config);
        return Ok(());
    }
    return Ok(());

    let config_path = std::env::current_dir().unwrap().join("task.ini");
    let tasks = match Task::from_ini(&config_path) {
        Ok(tasks) => tasks,
        Err(err) => {
            eprintln!("{} {}", "Error reading config file: ".red(), err);
            exit(1);
        }
    };
    println!("{:#?}", Task::serialize(tasks.task));
    return Ok(());
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
                println!(
                    "{} {}",
                    "Created stdout path: ".green(),
                    stdout_path.display()
                );
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
    let response = commands::exec(args, watchmen_path).await;
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
    fn test_clap() {
        unsafe {
            let lib = libloading::Library::new("/tmp/watchmen/plugins/libplugin.so").unwrap();
            let func: libloading::Symbol<unsafe extern "C" fn(t: i32) -> bool> =
                lib.get(b"func_plugin").unwrap();
            println!("PLUGIN: {:?}", func(1));
        }
        assert_eq!(1, 1);
    }
}
