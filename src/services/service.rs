use anyhow::Result;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, RwLock};
use s2n_quic::Server;
use tracing::error;
use tracing::info;

use crate::config::{ServerServiceConfig, ServiceConfig};
use crate::services::handle::{ControlChannel, ControlChannelHandle};
use crate::{CERT_PEM, Digest, KEY_PEM};

pub struct Services {
    server: Server,
    config: Arc<ServiceConfig>,
    // services: Arc<78<HashMap<Digest, ServerServiceConfig>>>,
    // control_channels: Arc<RwLock<ControlChannelMap>>,
}

impl Services {
    pub async fn init(_config: ServiceConfig) -> Result<Services> {
        let config = Arc::new(_config);
        // let services = Arc::new(RwLock::new(Self::generate_service(&config)));
        // 配置控制通道
        // let control_channels = Arc::new(RwLock::new(ControlChannelMap::new()));
        // 创建 监听套接字
        let socket_addr = config.bind_addr.parse::<SocketAddr>().unwrap();
        // let transport = Endpoint::server(Self::init_endpoint_config().await, socket_addr)?;
        let mut server = Server::builder()
            .with_tls((CERT_PEM,
                       KEY_PEM))?
            .with_io(socket_addr)?
            .start().unwrap();
        info!("监听服务创建成功");
        Ok(Services {
            config,
            server,
        })
    }


    // 生成 服务列表
    // pub fn generate_service(config: &ServiceConfig) -> HashMap<Digest, ServerServiceConfig> {
    //     let mut map = HashMap::new();
    //     for item in &config.services {
    //         map.insert(digest(item.0.as_bytes()), (item.1).clone());
    //     }
    //     map
    // }

    // 开始运行
    pub async fn run(&mut self, mut shutdown_rx: tokio::sync::broadcast::Receiver<bool>) {

        while let Some(mut connection) = self.server.accept().await {
            let control_channel = ControlChannel::build(connection);
            control_channel.handle().await;
        }

    }
}
