use std::collections::HashMap;

use anyhow::{Context, Result};
use tokio::sync::broadcast;
use tracing::{Instrument, Span};
use tracing::log::{info, Level, log};

use crate::client::handle::{ClientChannel, ClientChannelHandle};
use crate::config::ClientConfig;

pub struct Clients {
    config: ClientConfig,
    service_handles: HashMap<String, ClientChannelHandle>,
}

impl Clients {
    pub fn build(config: ClientConfig) -> Result<Self> {
        Ok(Clients {
            config,
            service_handles: HashMap::new(),
        })
    }

    pub async fn run(&mut self, mut shutdown_rx: broadcast::Receiver<bool>) -> Result<()> {

        // 循环创建对应的controlChannelHandler
        for (name, handle_config) in &self.config.services {
            let handle = ClientChannelHandle::build( &self.config.remote_addr, handle_config).await?;
            self.service_handles.insert(name.clone(), handle);
        }
        // 创建 链接通道然后开始认证
        info!("clients 开始运行");
        if let Ok(res) = shutdown_rx.recv().await {
            log!(Level::Info,"shutdown now {}", res);
            for (name, handle) in self.service_handles.drain() {
                handle.shutdown();
            };
            log!(Level::Info,"shutdown now");
        }
        Ok(())
    }
}