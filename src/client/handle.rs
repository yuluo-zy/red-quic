use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::sync::oneshot;
use anyhow::Result;
use bytes::Bytes;
use s2n_quic::{Client, Connection};
use s2n_quic::client::Connect;
use tokio::time::sleep;
use tracing::{callsite, info};
use tracing::log::{debug, log, warn};
use crate::config::ClientConfig;
use crate::{CERT_PEM};
use crate::protocol::Command;
use crate::utils::digest;

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
    pub(crate) digest: [u8;32],
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
       Self::new(connection, config).await
    }

    async fn new(conn:Connection, config: ClientConfig) -> Result<Self> {
        info!("创建连接");
        let conn = Self {
            digest: config.default_token.unwrap(),
            transport: conn,
            heartbeat_timeout: 10
        } ;
        Ok(conn)
    }
    pub async fn run(&mut self) -> Result<()> {
        // 开始写认证链接并创建相关数据通路的代码
        self.send_authentication().await;
        Ok(())
    }
    async fn send_authentication(& self)  {

        info!("开始进行认证操作");
        let  mut conn = self.transport.handle().open_bidirectional_stream().await.unwrap();
        info!("[{:?}] 认证秘钥", self.digest.clone());
        
        let cmd = Command::ShakeHands {
            digest: self.digest.clone()
        };
        cmd.write_to(&mut conn);

        // match conn.send(digest).await {
        //     Ok(_) => {
        //         info!("[relay] [connection] [authentication]")
        //     }
        //     Err(err) => {
        //         warn!("[relay] [connection] [authentication] {err}")
        //     }
        // }
        // 
        // loop {
        //     sleep(Duration::new(5,0)).await;
        //     let digest = Bytes::from("hello");
        //     info!("写入");
        //     conn.send(digest).await.unwrap();
        // }

    }
}