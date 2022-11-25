use crate::services::service::Services;
use quinn::Connecting;
use std::sync::{Arc, RwLock};

pub struct ControlChannel {
    service: Arc<RwLock<Services>>,
}

impl ControlChannel {
    pub fn build(service: Arc<RwLock<Services>>) -> Self {
        ControlChannel { service }
    }
    pub async fn handle(self, conn: Connecting) {
        let rmt_addr = conn.remote_address();
        match conn.await {
            Ok(_) => {}
            Err(_) => {}
        }
    }
}

pub struct ControlChannelHandle {}
