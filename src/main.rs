use std::env;
use anyhow::{Context, Result};
use tracing::{error, info};
use red_quic::{ Config};
use red_quic::client::run_client;
use red_quic::config::{ClientConfig, ServiceConfig, ServiceType};
use red_quic::services::run_server;
use clap::Parser;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {

    #[arg(short, long)]
    service: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();


    // info!("{args}");
    console_subscriber::init();
    tracing::info!("console_subscriber enabled");
    info!("{0}", args.service);
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
    let config ;
   if args.service == "service".to_string() {

       config = Config {
           service_type: Default::default(),
           name: "test".to_string(),
           service_config: Some(ServiceConfig {
               bind_addr: "127.0.0.1:7799".to_string(),
               default_token: Some([1;32]),
               services: Default::default()
           }),
           client_config: None
       };
   } else {
       config = Config {
           service_type: ServiceType::Client,
           name: "test".to_string(),
           service_config: None,
           client_config: Some(ClientConfig{
               remote_addr: "127.0.0.1:7799".to_string(),
               default_token: Some([1;32]),
               services: Default::default(),
               heartbeat_timeout: 0
           })
       };
   }
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
