use std::{error::Error, fs::remove_file, path::Path, process::exit};
use tokio::sync::mpsc;

use daemon::monitor::run_monitor;
use daemon::socket::run_socket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    // args[0]: daemon
    // args[1]: sock_path
    // args[2]: home_path
    // args[3]: stdout_path
    if args.len() < 4 {
        println!("Exit code: 255 => Missing path argument.");
        exit(255);
    }

    let home_path = args[2].clone();

    // 加载上次运行的任务
    let tasks_result = daemon::global::load_tasks(home_path.clone()).await;
    if tasks_result.is_err() {
        println!("Exit code: 254 => {}", tasks_result.err().unwrap());
        exit(254);
    }
    let monitor = tokio::spawn(async move {
        match run_monitor(home_path).await {
            Ok(_) => {}
            Err(e) => {
                println!("Exit code: 253 => {}", e);
                exit(253);
            }
        }
    });

    let path = args[1].clone();

    // 新线程运行 run_socket
    let p1 = path.clone();
    let socket = tokio::spawn(async move {
        run_socket(&p1).await.unwrap();
    });

    // 协程间通信 / channel, 监听 ctrl c 和 ctrl d 信号 / listen ctrl-c and ctrl-d signal
    let (tx, mut rx) = mpsc::channel::<i32>(12);

    let tx1 = tx.clone();  // 监听到 ctrl c 通信管道
    let tx2 = tx.clone();  // 监听到 ctrl d 通信管道

    
    // ctrl c 停止运行 / terminate on ctrl-c
    let signal1 = tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        tx1.send(9).await.unwrap();
    });
    
    // ctrl d 停止运行 / terminate on ctrl-d
    let signal2 = tokio::spawn(async move {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .unwrap()
            .recv()
            .await;
        tx2.send(15).await.unwrap();
    });

    // 等待停止信号 / wait for stop signal
    let res = rx.recv().await;

    signal1.abort();
    signal2.abort();
    monitor.abort();
    socket.abort();

    // 删除 sock 文件 / remove sock file
    let p2 = path.clone();
    let sock_path = Path::new(&p2);
    remove_file(sock_path).unwrap_or_default();

    if let Some(code) = res {
        println!("Exit code: {} => exited", code);
        exit(code);
    } else {
        println!("Exit code: 0 => exited");
        exit(0);
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test() {}
}
