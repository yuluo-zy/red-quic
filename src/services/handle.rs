use std::sync::Arc;
use std::time::Duration;
use s2n_quic::Connection;
use tracing::log::{info};
use crate::services::auth::{IsAuth, IsClosed};
use tokio::time;
use anyhow::{anyhow, Result};
use tokio::sync::RwLock;
use s2n_quic::application::Error as S2N_Error;
use s2n_quic::stream::BidirectionalStream;
use thiserror::Error;
use tracing::{error, warn};

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
    service: Arc<RwLock<Connection>>,
    is_auth: IsAuth,
}

impl ControlChannel {
    pub fn build(service: Connection) -> Self {
        info!("创建端点转发服务");
        ControlChannel {
            service: Arc::new(RwLock::new(service)),
            is_auth: IsAuth::new(IsClosed::new()),
        }
    }
    pub async fn handle(&self) {
        let addr = self.service.read().await.remote_addr().unwrap();
        info!("[{addr}] 远程连接");
        let res = tokio::select! {
                res = self.handle_authentication_timeout(Duration::new(20,0)) => res,
                res = self.echo() => res,
            };
    }

    pub async fn handle_authentication_timeout(&self, timeout: Duration) -> Result<()> {
        info!("认证函数");
        let is_timeout = tokio::select! {
           _=  self.is_auth.clone() => false,
           _=  time::sleep(timeout) => true
       };
        info!("认证完成");
        info!("1");
        if !is_timeout {
            Ok(())
        } else {
            let error = HandleError::AuthenticationTimeout;
            info!("2");
            self.service.read().await.close(error.clone().into());
            info!("3");
            self.is_auth.wake();
            info!("4");
            let rmt_addr = self.service.read().await.remote_addr().unwrap();
            info!("5");
            error!("[{rmt_addr}] 连接认证失败!");
            Err(anyhow!(error))
        }
    }
    pub async fn echo(&self) -> Result<()> {
        let mut stream = None ;
        {
            if let Ok(Some(data_stream)) = self.service.write().await.accept_bidirectional_stream().await {
                stream = Some(data_stream);
            }
        }
        match stream{
            None => {}
            Some(mut stream) => {
                while let Ok(Some(data)) = stream.receive().await {
                    warn!("{:?}", data)
                };
            }
        }
        Ok(())
    }
}

pub struct ControlChannelHandle {}
