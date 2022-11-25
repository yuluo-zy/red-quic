use std::collections::HashMap;
use std::net::SocketAddr;
use crate::client::handle::ClientChannelHandle;
use crate::config::ClientConfig;
use anyhow::Result;
use quinn::Endpoint;

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

    }
}