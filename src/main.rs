use anyhow::Result;
use red_quic::{run, Config};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志设置
    console_subscriber::init();
    tracing::info!("console_subscriber enabled");
    // 设置参数解析

    // 设置全局监听通道
    let (all_shutdown_tx, all_shutdown_rx) = tokio::sync::broadcast::channel::<bool>(1);

    // 设置退出监听
    tokio::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            panic!("Failed to listen for the ctrl-c signal: {:?}", e);
        }
        if let Err(e) = all_shutdown_tx.send(true) {
            panic!("Failed to send shutdown signal: {:?}", e);
        }
    });
    run(
        Config {
            ..Default::default()
        },
        all_shutdown_rx,
    )
    .await
}
