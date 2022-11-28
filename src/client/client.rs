use std::collections::HashMap;
use std::net::SocketAddr;
use std::thread::sleep;
use crate::client::handle::{ClientChannel, ClientChannelHandle};
use crate::config::ClientConfig;
use anyhow::{Context, Result};
use quinn::Endpoint;
use tracing::log::{Level, log};
use tracing::{Instrument, Span};

pub struct Clients {
    transport: Endpoint,
    config: ClientConfig,
    service_handles: HashMap<String, ClientChannelHandle>
}

impl Clients {
    pub fn build(config: ClientConfig) -> Result<Self> {
        let socket_addr = config.local_addr.parse::<SocketAddr>().unwrap();
        Ok(Clients {
            config,
            service_handles: HashMap::new(),
            transport: Endpoint::client(socket_addr).unwrap()
        })
    }

    pub async fn run(&mut self, mut shutdown_rx: tokio::sync::broadcast::Receiver<bool>) {

        // 循环创建对应的controlChannelHandler
        for item in &self.config.services {
            self.service_handles.insert(item.0.clone(), item.1.)
        }
        // 创建 链接通道然后开始认证
        let mut controlChannel = ClientChannel {
            digest: self.config.default_token.unwrap(),
            shutdown_rx: shutdown_rx.clone(),
            remote_addr: self.config.remote_addr.clone(),
            transport: self.transport.clone(),
            heartbeat_timeout: self.config.heartbeat_timeout
        };
        tokio::spawn(async move {
            while let Err(e) = controlChannel.run().await.with_context(|| "失败情动"){
                // 条件判断是否要重新启动
            }.instrument(Span::current())
        });
       if let Ok(res) = shutdown_rx.recv().await {
           log!(Level::Info,"shutdown now {}", res);
           log!(Level::Info,"shutdown now");
       }

    }
}