use std::{error::Error, fs::remove_file, path::Path, process::exit};
use tokio::sync::mpsc;

use daemon::socket::run_socket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    println!("{:?}", args);
    if args.len() < 4 {
        eprintln!("Missing path argument.");
        exit(0);
    }

    match daemon::global::load_tasks().await {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{}", err);
            exit(0);
        }
    };

    let path = args[1].clone();
    let p1 = path.clone();
    let p2 = path.clone();

    let socket = tokio::spawn(async move {
        run_socket(&p1).await.unwrap();
    });

    // 协程间通信 / channel
    let (tx, mut rx) = mpsc::channel::<()>(1);

    let tx1 = tx.clone();
    let tx2 = tx.clone();
    let signal1 = tokio::spawn(async move {
        // ctrl c 停止运行 / terminate on ctrl-c
        tokio::signal::ctrl_c().await.unwrap();
        tx1.send(()).await.unwrap();
    });
    let signal2 = tokio::spawn(async move {
        // ctrl d 停止运行 / terminate on ctrl-d
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .unwrap()
            .recv()
            .await;
        tx2.send(()).await.unwrap();
    });

    // 等待停止信号 / wait for stop signal
    rx.recv().await;

    println!("\nexiting...");
    signal1.abort();
    signal2.abort();
    socket.abort();
    let sock_path = Path::new(&p2);
    remove_file(sock_path).unwrap_or_default();
    println!("exited");

    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test() {}
}
