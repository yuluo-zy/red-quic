use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use quinn::{Endpoint, EndpointConfig, ServerConfig};
use tokio::sync::RwLock;

use crate::config::{ServerServiceConfig, ServiceConfig};
use crate::Digest;
use crate::services::ControlChannelMap;
use crate::services::multi_map::MultiMap;
use crate::utils::digest;

pub struct Services {
    // transport: Endpoint,
    config: Arc<ServiceConfig>,
    services: Arc<RwLock<HashMap<Digest, ServerServiceConfig>>>,
    control_channels: Arc<RwLock<ControlChannelMap>>,
}

impl Services {
    pub fn init(config: ServiceConfig) -> Result<Services>{
        let config = Arc::new(config);
        let services = Arc::new(RwLock::new(Services::generate_service(&config)));
        let control_channels = Arc::new(RwLock::new(ControlChannelMap::new()));
        // let transport = Endpoint::new();
        Ok(Services{
            config,
            services,
            control_channels,
            // transport
        })
    }

    // 生成 服务列表
    pub fn generate_service(config:  &ServiceConfig) -> HashMap<Digest, ServerServiceConfig> {
        let mut map = HashMap::new();
        for item in &config.services {
            map.insert(digest(item.0.as_bytes()), *(item.1).clone());
        }
        map
    }
}