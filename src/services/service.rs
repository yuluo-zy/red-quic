use std::collections::HashMap;
use anyhow::Result;
use std::net::SocketAddr;
use std::sync::{Arc};
use parking_lot::Mutex;
use s2n_quic::Server;
use sha2::Digest;
use tracing::info;

use crate::config::{ServerServiceConfig, ServiceConfig};
use crate::services::handle::{ControlChannel, ControlChannelMap};
use crate::{CERT_PEM, KEY_PEM};
use crate::protocol::ProtocolDigest;
use crate::utils::digest;


pub struct Services {
    server: Server,
    config: Arc<ServiceConfig>,
    services_config: Arc<HashMap<ProtocolDigest, ServerServiceConfig>>,
    service_channels: Arc<Mutex<ControlChannelMap>>,
}

impl Services {
    pub async fn init(config: ServiceConfig) -> Result<Services> {
        let _config = Arc::new(config);
        let service_map = generate_service(&_config);
        let socket_addr = _config.bind_addr.parse::<SocketAddr>().unwrap();
        let server = Server::builder()
            .with_tls((CERT_PEM,
                       KEY_PEM))?
            .with_io(socket_addr)?
            .start().unwrap();
        let map = Arc::new(Mutex::new(HashMap::new()));
        info!("监听服务创建成功");
        Ok(Services {
            config: _config,
            server,
            service_channels: map,
            services_config: Arc::new(service_map),
        })
    }

    // 开始运行
    pub async fn run(&mut self, mut shutdown_rx: tokio::sync::broadcast::Receiver<bool>) {

        while let Some(connection) = self.server.accept().await {
            info!("构建 server");
            let mut control_channel = ControlChannel::build(
                self.config.default_token.unwrap(),
                self.services_config.clone(),
                self.service_channels.clone(),

            );
            control_channel.handle(connection).await;
        }

    }
}

// 生成 服务列表
pub fn generate_service(config: &ServiceConfig) -> HashMap<ProtocolDigest, ServerServiceConfig> {
    let mut map = HashMap::new();
    for item in &config.services {
        map.insert(digest(item.0.as_bytes()), (*item.1).clone());
    }
    map
}