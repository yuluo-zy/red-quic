use std::collections::HashMap;
use anyhow::Result;
use std::net::SocketAddr;
use std::sync::{Arc};
use parking_lot::Mutex;
use s2n_quic::Server;
use tracing::info;

use crate::config::{ServiceConfig};
use crate::services::handle::{ControlChannel, ControlChannelMap};
use crate::{CERT_PEM, KEY_PEM};



pub struct Services {
    server: Server,
    config: Arc<ServiceConfig>,
    // services: Arc<<HashMap<Digest, ServerServiceConfig>>>,
    service_channels: Arc<Mutex<ControlChannelMap>>,
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
        let map = Arc::new(Mutex::new(HashMap::new()));
        info!("监听服务创建成功");
        Ok(Services {
            config,
            server,
            service_channels: map
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

        while let Some(connection) = self.server.accept().await {
            info!("构建 server");
            let mut control_channel = ControlChannel::build(
                self.config.default_token.unwrap(),
                self.service_channels.clone()
            );
            control_channel.handle(connection).await;
        }

    }
}
