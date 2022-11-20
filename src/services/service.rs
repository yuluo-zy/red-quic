use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use tracing::info;
use anyhow::{Context, Result};
use quinn::{default_runtime, Endpoint, ServerConfig as EndpointServerConfig};
use tokio::sync::RwLock;

use crate::config::{ServerServiceConfig, ServiceConfig};
use crate::Digest;
use crate::services::ControlChannelMap;
use crate::services::multi_map::MultiMap;
use crate::utils::{digest, generate_self_signed_cert};

pub struct Services {
    transport: Endpoint,
    config: Arc<ServiceConfig>,
    services: Arc<RwLock<HashMap<Digest, ServerServiceConfig>>>,
    control_channels: Arc<RwLock<ControlChannelMap>>,
}

impl Services {
    pub async fn init(_config: ServiceConfig) -> Result<Services> {
        let config = Arc::new(_config);
        let services = Arc::new(RwLock::new(Self::generate_service(&config)));
        // 配置控制通道
        let control_channels = Arc::new(RwLock::new(ControlChannelMap::new()));
        // 创建 监听套接字
        let socketAddr = config.bind_addr.parse::<SocketAddr>().unwrap();
        let transport = Endpoint::server(Self::init_endpoint_config().await, socketAddr)?;
        Ok(Services {
            config,
            services,
            control_channels,
            transport,
        })
    }

    pub async fn init_endpoint_config() -> EndpointServerConfig {
        let (cert, key) = generate_self_signed_cert().unwrap();
        EndpointServerConfig::with_single_cert(vec![cert], key).unwrap()
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
    pub async fn run(&mut self, mut shutdown_rx: tokio::sync::broadcast::Receiver<bool>) -> Result<()> {
        // loop {
        tokio::select! {
                ret = self.transport.accept() => {
                    match ret {
                        Some(conn) => {
                             info!("conn...");
                        }
                        _ => {
                             info!("error...");
                        }
                    }
                },
                _ = shutdown_rx.recv() => {
                    info!("Shuting down gracefully...");
                    // break;
                },
            }
        // }
        Ok(())
    }
}