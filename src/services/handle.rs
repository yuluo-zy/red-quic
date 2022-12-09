use std::sync::Arc;
use std::time::Duration;
use s2n_quic::Connection;
use tracing::log::{info};
use crate::services::auth::{IsAuth, IsClosed};
use tokio::time;
use anyhow::{anyhow, Context, Result};
use crate::protocol::Command;
use tokio::sync::RwLock;
use s2n_quic::application::Error as S2N_Error;
use s2n_quic::connection::Handle;
use s2n_quic::stream::BidirectionalStream;
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
    digest: [u8;32]
}

impl ControlChannel {
    pub fn build(digest: [u8;32]) -> Self {
        info!("创建端点转发服务");
        ControlChannel {
            is_auth: IsAuth::new(IsClosed::new()),
            digest
        }
    }
    pub async fn handle(&mut self, conn: Connection) {
        let addr = conn.remote_addr().unwrap();
        info!("[{addr}] 远程连接");
        let res = tokio::select! {
                res = self.handle_authentication_timeout(conn.handle(), Duration::new(20,0)) => res,
                res = self.run(conn) => res,
            };
    }

    pub async fn handle_authentication_timeout(&self,
                                               conn: Handle,
                                               timeout: Duration) -> Result<()> {
        info!("认证函数");
        let auth_success = tokio::select! {
           is_auth =  self.is_auth.clone() => is_auth,
           _=  time::sleep(timeout) => false
       };
        info!("认证完成");
        if auth_success {
            info!("连接成功");
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
    pub async fn run(&self, mut conn: Connection) -> Result<()> {
        info!("run");
        while let Ok(Some(mut data_stream)) = conn.accept_bidirectional_stream().await {
            info!("handshake");
            self.handshake(&mut data_stream).await;

        }
        Ok(())
    }

    pub async fn handshake(&self, stream: &mut  BidirectionalStream) -> Result<()> {
        let token = Command::read_from(stream).await;
        let addr = stream.connection().remote_addr();
        info!("handshakeing");
        info!("{:?}", token);
        match token {
            Ok(cmd) => {
                if let Command::ShakeHands {
                    digest
                } = cmd {
                    if self.digest.eq(&digest) {
                        info!("握手成功 {:?}", addr);
                        self.is_auth.set_auth();
                    } else {
                        error!("秘钥错误");
                        self.is_auth.set_close();
                    }
                    self.is_auth.wake();
                }
            }
            _ => {
                error!("认证失败")
            }
        }
        Ok(())
    }
}

pub struct ControlChannelHandle {}
