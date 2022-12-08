use std::sync::Arc;
use std::time::Duration;
use s2n_quic::Connection;
use tracing::log::{info};
use crate::services::auth::{IsAuth, IsClosed};
use tokio::time;
use anyhow::{anyhow, Result};
use tokio::sync::RwLock;
use s2n_quic::application::Error as S2N_Error;
use s2n_quic::connection::Handle;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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
    is_auth: IsAuth,
}

impl ControlChannel {
    pub fn build() -> Self {
        info!("创建端点转发服务");
        ControlChannel {
            is_auth: IsAuth::new(IsClosed::new()),
        }
    }
    pub async fn handle(&mut self, conn: Connection) {
        let addr = conn.remote_addr().unwrap();
        info!("[{addr}] 远程连接");
        let res = tokio::select! {
                res = self.handle_authentication_timeout(conn.handle(), Duration::new(20,0)) => res,
                res = self.echo(conn) => res,
            };
    }

    pub async fn handle_authentication_timeout(&self,
                                               conn: Handle,
                                               timeout: Duration) -> Result<()> {
        info!("认证函数");
        let is_timeout = tokio::select! {
           _=  self.is_auth.clone() => false,
           _=  time::sleep(timeout) => true
       };
        info!("认证完成");
        if !is_timeout {
            Ok(())
        } else {
            let error = HandleError::AuthenticationTimeout;
            conn.close(error.clone().into());
            self.is_auth.wake();
            let rmt_addr = conn.remote_addr().unwrap();
            error!("[{rmt_addr}] 连接认证失败!");
            Err(anyhow!(error))
        }
    }
    pub async fn echo(&self, mut conn: Connection) -> Result<()> {
        if let Ok(Some(data_stream)) = conn.accept_bidirectional_stream().await {
            match Some(data_stream) {
                None => {}
                Some(mut stream) => {
                    while let Ok(Some(data)) = stream.receive().await {
                        warn!("{:?}", data)
                    };
                }
            }
        }
        Ok(())
    }
}

pub struct ControlChannelHandle {}
