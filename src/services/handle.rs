use std::time::Duration;
use s2n_quic::Connection;
use tracing::log::{info};
use crate::services::auth::{IsAuth, IsClosed};
use tokio::time;
use s2n_quic::application::Error as S2N_Error;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug, Clone)]
pub enum HandleError {
    // #[error(transparent)]
    // Io(#[from] IoError),
    #[error("authentication failed")]
    AuthenticationFailed,
    #[error("authentication timeout")]
    AuthenticationTimeout,
    #[error("bad command")]
    BadCommand,
}

impl From<HandleError> for S2N_Error {
    fn from(value: HandleError) -> Self {
        match value {
            // HandleError::Io(_) => {
            //     S2N_Error::from(1u8)
            // }
            HandleError::AuthenticationFailed => {
                S2N_Error::from(2u8)
            }
            HandleError::AuthenticationTimeout => {
                S2N_Error::from(3u8)
            }
            HandleError::BadCommand => {
                S2N_Error::from(4u8)
            }
        }
    }
}


pub struct ControlChannel {
    service: Connection,
    is_auth: IsAuth,
}

impl ControlChannel {
    pub fn build(service: Connection) -> Self {
        info!("创建端点转发服务");
        ControlChannel {
            service,
            is_auth: IsAuth::new(IsClosed::new()),
        }
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

    pub async fn handle_authentication_timeout(self, timeout: Duration) -> Result<(), HandleError> {
        let is_timeout = tokio::select! {
           _ = self.is_auth.clone() => false,
           () = time::sleep(timeout) => true
       };
        if !is_timeout {
            Ok(())
        } else {
            let error = HandleError::AuthenticationTimeout;
            self.service.close(error.clone().into());
            self.is_auth.wake();
            let rmt_addr = self.service.remote_addr().unwrap();
            error!("[{rmt_addr}] 连接认证失败!");
            Err(error)
        }
    }
}

pub struct ControlChannelHandle {}
