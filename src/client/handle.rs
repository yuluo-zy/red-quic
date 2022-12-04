use std::net::{Ipv4Addr, SocketAddr};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::oneshot;
use anyhow::Result;
use bytes::Bytes;
use s2n_quic::{Client, Connection};
use s2n_quic::client::Connect;
use s2n_quic::stream::SendStream;
use tracing::{callsite, info};
use tracing::log::{debug, log, warn};
use crate::config::ClientConfig;
use crate::{CERT_PEM, Digest};

pub struct ClientChannelHandle {
    shutdown_tx: oneshot::Sender<u8>,
}

impl ClientChannelHandle {
    pub async fn run(&mut self) -> Result<()> {
        Ok(())
    }
    pub fn shutdown(self) {
        self.shutdown_tx.send(0u8).expect("TODO: panic message");
    }
//     pub async fn build()-> Self{
// //
//     }
}

pub struct ClientChannel {
    pub(crate) digest: String,
    // pub(crate) shutdown_rx: oneshot::Receiver<bool>,
    // pub(crate) remote_addr: String,
    // `client.remote_addr`
    pub(crate) transport: Connection,
    pub(crate) heartbeat_timeout: u64,             // Application layer heartbeat timeout in secs
}

impl ClientChannel {
    pub async fn build(config: ClientConfig) ->Result<Self> {
        let client = Client::builder()
            .with_tls(CERT_PEM)?
            .with_io("0.0.0.0:0")?
            .start().unwrap();
        info!("构建本地监听内容");

        let addr: SocketAddr =config.remote_addr.parse()?;
        let connect = Connect::new(addr).with_server_name("localhost");

        info!("尝试连接远程");
        let mut connection = client.connect(connect).await?;
        connection.keep_alive(true)?;
        let mut client_channel = Self::new(connection, config).await;
        Ok(client_channel)
    }

    async fn new(conn:Connection, config: ClientConfig) -> Self {
        info!("创建连接");
        let conn = Self {
            digest: config.default_token.unwrap(),
            transport: conn,
            heartbeat_timeout: 10
        } ;
        conn
    }
    pub async fn run(&mut self) -> Result<()> {
        // 开始写认证链接并创建相关数据通路的代码
        Ok(())
    }
    async fn send_authentication(mut self)  {

        info!("开始进行认证操作");
        let mut conn = self.transport.open_send_stream().await?;
        let digest = Bytes::from(self.digest.clone());

        match conn.send(digest).await {
            Ok(_) => {
                info!("[relay] [connection] [authentication]")
            }
            Err(err) => {
                warn!("[relay] [connection] [authentication] {err}")
            }
        }

    }
}