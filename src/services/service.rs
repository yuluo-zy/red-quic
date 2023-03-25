use std::collections::HashMap;
use anyhow::Result;
use std::net::SocketAddr;
use std::sync::{Arc};
use parking_lot::Mutex;
use crate::utils::digest;
use s2n_quic::Server;
use tracing::info;


use crate::config::{ServiceConfig};
use crate::services::handle::{ControlChannel, ControlChannelMap};
use crate::{CERT_PEM, KEY_PEM};
use crate::protocol::ProtocolDigest;

/// 服务端程序内容
pub struct Services {
    /// 服务runner
    server: Server,
    /// 配置
    config: Arc<ServiceConfig>,
    /// 转发服务的控制句柄
    service_channels: Arc<Mutex<ControlChannelMap>>,
}

impl Services {
    pub async fn init(config: ServiceConfig) -> Result<Services> {
        let _config = Arc::new(config);
        let socket_addr = _config.bind_addr.parse::<SocketAddr>()?;
        let server = Server::builder()
            .with_tls((CERT_PEM, KEY_PEM))?
            .with_io(socket_addr)?
            .start().unwrap();
        let map = Arc::new(Mutex::new(HashMap::new()));
        info!("监听服务创建成功");
        Ok(Services {
            config: _config,
            server,
            service_channels: map,
        })
    }

    // 开始运行
    pub async fn run(&mut self, mut shutdown_rx: tokio::sync::broadcast::Receiver<bool>) -> Result<()>{

        while let Some(connection) = self.server.accept().await {
            info!("构建 server");
            let mut control_channel = ControlChannel::build(
                self.service_channels.clone()
            );
            control_channel.run(connection).await?;
        }
    
        Ok(())
    }
}