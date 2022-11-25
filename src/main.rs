use anyhow::{Context, Result};
use red_quic::{ Config};
use red_quic::client::run_client;
use red_quic::config::{ServiceConfig, ServiceType};
use red_quic::services::run_server;

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
    let config = Config {
        service_type: Default::default(),
        name: "test".to_string(),
        service_config: Some(ServiceConfig {
            bind_addr: "127.0.0.1:7799".to_string(),
            default_token: None,
            services: Default::default()
        }),
        client_config: None
    };
    // 判断 运行的类型

    match config.service_type {
        ServiceType::Client => {
            run_client(config.client_config.unwrap(),all_shutdown_rx).await;
        }
        ServiceType::Service => {
            run_server(config.service_config.unwrap(),
                       all_shutdown_rx
            ).await;
        }
    }
    Ok(())
}
