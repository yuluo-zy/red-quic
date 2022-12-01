use anyhow::Result;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use s2n_quic::Server;
use tracing::error;
use tracing::info;

use crate::config::{ServerServiceConfig, ServiceConfig};
use crate::services::handle::{ControlChannel, ControlChannelHandle};
use crate::Digest;

use crate::utils::{digest};

#[derive(Clone)]
pub struct Services {
    server: Server,
    config: Arc<ServiceConfig>,
    // services: Arc<78<HashMap<Digest, ServerServiceConfig>>>,
    // control_channels: Arc<RwLock<ControlChannelMap>>,
}

impl Services {
    pub async fn init(_config: ServiceConfig) -> Result<Services> {
        let config = Arc::new(_config);
        let services = Arc::new(RwLock::new(Self::generate_service(&config)));
        // 配置控制通道
        // let control_channels = Arc::new(RwLock::new(ControlChannelMap::new()));
        // 创建 监听套接字
        let socket_addr = config.bind_addr.parse::<SocketAddr>().unwrap();
        // let transport = Endpoint::server(Self::init_endpoint_config().await, socket_addr)?;
        let mut server = Server::builder()
            .with_tls(("../key/cert.pem", "../key/key.pem"))?
            .with_io("0.0.0.0:7779")?
            .start()?;
        Ok(Services {
            config,
            server,
        })
    }


    // 生成 服务列表
    pub fn generate_service(config: &ServiceConfig) -> HashMap<Digest, ServerServiceConfig> {
        let mut map = HashMap::new();
        for item in &config.services {
            map.insert(digest(item.0.as_bytes()), (item.1).clone());
        }
        map
    }

    // 开始运行
    pub async fn run(&mut self, mut shutdown_rx: tokio::sync::broadcast::Receiver<bool>) {
        loop {
            tokio::select! {
                ret = self.server.accept() => {
                    match ret {
                        Some(mut conn) => {
                            // 接受的是 正在创建的内容
                             info!("conn...");
                            // let service = self.clone();
                            let control_channel = ControlChannel::build(conn.clone());
                            control_channel.handle(conn).await
                        }
                        _ => {
                             error!("conn error")
                        }
                    }
                },
                _ = shutdown_rx.recv() => {
                    info!("Shuting down gracefully...");
                    break;
                },
            }
        }
    }
}
