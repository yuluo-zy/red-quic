use crate::services::service::Services;
use quinn::Connecting;
use std::sync::{Arc, RwLock};
use tokio::io::AsyncReadExt;
use tracing::log::{info, log};

pub struct ControlChannel {
    service: Arc<RwLock<Services>>,
}

impl ControlChannel {
    pub fn build(service: Arc<RwLock<Services>>) -> Self {
        ControlChannel { service }
    }
    pub async fn handle(self, conn: Connecting) {
        let rmt_addr = conn.remote_address();
        info!("远程连接{rmt_addr}");
        if let Ok(conn) = conn.await {
           let (mut _sendStream, mut _reacvStream) =  conn.open_bi().await.unwrap();
            let mut  buf = [0u8; 4];
            buf.map(|a| info!("{a}") );
            _reacvStream.read_exact(&mut buf).await;
            buf.map(|a| info!("{a}") );
        };
    }
}

pub struct ControlChannelHandle {}
