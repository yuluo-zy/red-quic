use s2n_quic::Connection;
use s2n_quic::connection::Error as ConnectionError;
use tokio::io::AsyncReadExt;
use tracing::log::{info, log};


pub struct ControlChannel {
    service: Connection,
}

impl ControlChannel {
    pub fn build(service: Connection) -> Self {
        info!("创建端点转发服务");
        ControlChannel { service  }
    }
    pub async fn handle(mut self) {

        while let Ok(Some(mut conn)) = self.service.accept_bidirectional_stream().await {
            tokio::spawn(async move {
                info!("Stream opened from {:?}", conn.connection().remote_addr());
                while let Ok(Some(data)) = conn.receive().await {
                    let _ = data.iter().map(|a| info!("{}",1));
                    info!("{:?}", data)
                }
            });
        };
    }

    pub async fn handle_authentication_timeout() -> Result<(),ConnectionError> {
        
        ConnectionError::Closed
    }
}

pub struct ControlChannelHandle {}
