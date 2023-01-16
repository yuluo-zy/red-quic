use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::sync::oneshot;
use anyhow::{anyhow, Result};
use bytes::Bytes;
use s2n_quic::{Client, Connection};
use s2n_quic::client::Connect;
use s2n_quic::stream::BidirectionalStream;
use socket2::{SockRef, TcpKeepalive};
use tokio::net::TcpStream;
use tokio::time::sleep;
use tracing::{debug, info};
use tracing::log::{log, warn};
use crate::config::{ClientConfig, ClientServiceConfig};
use crate::{CERT_PEM};
use crate::protocol::Command;

const KEEPALIVE_DURATION: Duration = Duration::new(20, 0);
const KEEPALIVE_INTERVAL: Duration = Duration::new(8, 0);


pub struct ClientChannelHandle {
    shutdown_tx: oneshot::Sender<u8>,
}

impl ClientChannelHandle {
    pub async fn run(&mut self,
                     remote_addr: &String,
                     server_config: &ClientServiceConfig) -> ClientChannelHandle {
        info!("start service {}", server_config.name);
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        // 创建 client 通道
        Self {
            shutdown_tx
        }
    }
    pub fn shutdown(self) {
        self.shutdown_tx.send(0u8).expect("TODO: panic message");
    }
}


pub struct ClientChannel {
    pub(crate) digest: [u8; 32],
    pub(crate) shutdown_rx: oneshot::Receiver<bool>,
    pub(crate) transport: Connection,
    pub(crate) heartbeat_timeout: u64,             // Application layer heartbeat timeout in secs
}

impl ClientChannel {
    pub async fn build(config: &ClientServiceConfig,
                       remote_addr: &String,
                       shut_down_rx: oneshot::Receiver<u8>) -> Result<Self> {
        // 创建 connect
        let client = Client::builder()
            .with_tls(CERT_PEM)?
            .with_io("0.0.0.0:0")?
            .start().unwrap();
        info!("构建本地监听内容");

        let remote_addr: SocketAddr = remote_addr.parse()?;
        let connect = Connect::new(remote_addr).with_server_name("localhost");

        // 创建 代理地址 并设置参数:
        // let tcp_stream: TcpStream = TcpStream::connect(remote_addr).with_context(|| format!("Failed to connect to {}", &remote_addr)).await?;
        // tcp_stream.set_nodelay(true)?;
        //
        // let s = SockRef::from(&tcp_stream);
        // let keepalive = TcpKeepalive::new()
        //     .with_time(KEEPALIVE_DURATION)
        //     .with_interval(KEEPALIVE_INTERVAL);
        //
        // debug!("Set TCP keepalive {:?} {:?}",KEEPALIVE_DURATION,KEEPALIVE_INTERVAL);
        //
        // s.set_tcp_keepalive(&keepalive)?;

        info!("尝试连接远程");
        let mut connection = client.connect(connect).await?;
        connection.keep_alive(true)?;
        let token = config.token.ok_or(anyhow!("token is option"))?;
        Self::new(connection, token,  shut_down_rx).await
    }

    async fn new(conn: Connection, digest: [u8;32],  shut_down_rx: oneshot::Receiver<u8>) -> Result<Self> {
        info!("创建连接");
        let conn = Self {
            digest,
            shutdown_rx,
            transport: conn,
            heartbeat_timeout: 10,
        };
        Ok(conn)
    }
    pub async fn run(&mut self) -> Result<()> {
        // 开始写认证链接并创建相关数据通路的代码
        let mut conn = self.transport.handle().open_bidirectional_stream().await.unwrap();
        self.send_authentication(&mut conn).await;
        Ok(())
    }
    async fn send_authentication(&self, mut conn: &BidirectionalStream) {
        info!("开始进行认证操作");

        info!("[{:?}] 认证秘钥", &self.digest);

        let cmd = Command::ShakeHands {
            digest: self.digest
        };
        cmd.write_to(&mut conn).await.unwrap();

        // match conn.send(digest).await {
        //     Ok(_) => {
        //         info!("[relay] [connection] [authentication]")
        //     }
        //     Err(err) => {
        //         warn!("[relay] [connection] [authentication] {err}")
        //     }
        // }
        //
        loop {
            sleep(Duration::new(5, 0)).await;
            let digest = Bytes::from("hello");
            info!("写入");
            conn.send(digest).await.unwrap();
        }
    }
}