use common::config::Config;
use tokio::sync::mpsc;

#[cfg(feature = "sock")]
pub mod sock;

#[cfg(feature = "socket")]
pub mod socket;

#[cfg(feature = "http")]
pub mod http;

pub async fn start(config: Config) {
    let (tx, mut rx) = mpsc::channel::<i32>(12);

    let tx_ctrl_c = tx.clone(); // 监听到 ctrl c 通信管道
    let tx_ctrl_d = tx.clone(); // 监听到 ctrl d 通信管道

    // ctrl c 停止运行 / terminate on ctrl-c
    let s_ctrl_c = tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        tx_ctrl_c.send(9).await.unwrap();
    });

    // ctrl d 停止运行 / terminate on ctrl-d
    let s_ctrl_d = tokio::spawn(async move {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .unwrap()
            .recv()
            .await;
        tx_ctrl_d.send(15).await.unwrap();
    });

    #[cfg(feature = "sock")]
    let joinhandle_sock = sock::start(config.clone()).await;

    #[cfg(feature = "socket")]
    let joinhandle_socket = socket::start(config.clone()).await;

    #[cfg(feature = "http")]
    let joinhandle_http = http::start(config.clone()).await;

    // ================== Wait for all tasks to complete ==================

    let _res = rx.recv().await;

    s_ctrl_c.abort();
    s_ctrl_d.abort();

    println!("Shutting down...");

    #[cfg(feature = "sock")]
    {
        joinhandle_sock.abort();
    }

    #[cfg(feature = "socket")]
    {
        joinhandle_socket.abort();
    }

    #[cfg(feature = "http")]
    {
        joinhandle_http.abort();
    }
}
