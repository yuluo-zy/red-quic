use std::collections::HashMap;

use anyhow::{Context, Result};
use tracing::{Instrument, Span};
use tracing::log::{info, Level, log};

use crate::client::handle::{ClientChannel, ClientChannelHandle};
use crate::config::ClientConfig;

pub struct Clients {
    config: ClientConfig,
    service_handles: HashMap<String, ClientChannelHandle>
}

impl Clients {
    pub fn build(config: ClientConfig) -> Result<Self> {
        Ok(Clients {
            config,
            service_handles: HashMap::new(),
        })
    }

    pub async fn run(&mut self, mut shutdown_rx: tokio::sync::broadcast::Receiver<bool>) {

        // 循环创建对应的controlChannelHandler
        // for item in &self.config.services {
        //     self.service_handles.insert(item.0.clone(), item.1.clone())
        // }
        // 创建 链接通道然后开始认证
        info!("开始创建");
        let mut control_channel = ClientChannel::build(self.config.clone()).await;
        // tokio::spawn(async move {
        //     while let Err(e) = control_channel.run().await.with_context(|| "失败情动"){
        //         // 条件判断是否要重新启动
        //     }.instrument(Span::current())
        // });
       if let Ok(res) = shutdown_rx.recv().await {
           log!(Level::Info,"shutdown now {}", res);
           log!(Level::Info,"shutdown now");
       }

    }
}