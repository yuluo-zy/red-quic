use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::sync::oneshot;
use anyhow::{anyhow, Context, Result};
use bytes::Bytes;
use s2n_quic::{Client, Connection};
use s2n_quic::client::Connect;
use s2n_quic::stream::BidirectionalStream;
use socket2::{SockRef, TcpKeepalive};
use tokio::time::sleep;
use tracing::{debug, error, info};
use tracing::log::{log, warn};
use tracing::log::Level::Debug;
use crate::config::{ClientConfig, ClientServiceConfig};
use crate::{CERT_PEM};
use crate::protocol::{Command, ProtocolDigest};
use crate::utils::digest as utils_digest;

const KEEPALIVE_DURATION: Duration = Duration::new(20, 0);
const KEEPALIVE_INTERVAL: Duration = Duration::new(8, 0);


pub struct ClientChannelHandle {
    shutdown_tx: oneshot::Sender<bool>,
}

impl ClientChannelHandle {
    pub async fn build(remote_addr: &String, server_config: &ClientServiceConfig) -> Result<ClientChannelHandle> {
        info!("start service {}", server_config.name);
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        // 创建 client 通道
        let mut runner = ClientChannel::build(server_config, remote_addr, shutdown_rx).await?;
        tokio::spawn(async move {
            runner.run().await
        });

        Ok(Self {
            shutdown_tx
        })
    }
    pub fn shutdown(self) {
        if let Err(e) = self.shutdown_tx.send(true) {
            error!("close service error")
        }
    }
}


pub struct ClientChannel {
    pub(crate) digest: ProtocolDigest,
    pub(crate) shutdown_rx: oneshot::Receiver<bool>,
    pub(crate) transport: Connection,
    pub(crate) token: ProtocolDigest,
    pub(crate) heartbeat_timeout: u64,             // Application layer heartbeat timeout in secs
}

impl ClientChannel {
    pub async fn build(config: &ClientServiceConfig,
                       remote_addr: &String,
                       shutdown_rx: oneshot::Receiver<bool>) -> Result<Self> {
        // 创建 connect
        let client = Client::builder()
            .with_tls(CERT_PEM)?
            .with_io("0.0.0.0:0")?
            .start().unwrap();
        info!("构建本地监听内容");

        let remote_addr: SocketAddr = remote_addr.parse()?;
        let connect = Connect::new(remote_addr).with_server_name("localhost");

        info!("尝试连接远程");
        let mut connection = client.connect(connect).await?;
        connection.keep_alive(true)?;
        // let token = config.token.as_ref().ok_or(anyhow!("token is option"))?.clone();
        let digest = utils_digest(config.name.as_bytes());
        let token = utils_digest(config.token.as_ref().unwrap().as_bytes());

        Self::new(connection, digest, token,  shutdown_rx).await
    }

    async fn new(conn: Connection, digest: ProtocolDigest, token: ProtocolDigest, shutdown_rx: oneshot::Receiver<bool>) -> Result<Self> {
        info!("创建连接");
        let conn = Self {
            digest,
            shutdown_rx,
            transport: conn,
            token,
            heartbeat_timeout: 10,
        };
        Ok(conn)
    }
    pub async fn run(&mut self) -> Result<()> {
        // 开始写认证链接并创建相关数据通路的代码
        let mut conn = self.transport.handle().open_bidirectional_stream().await.unwrap();
        self.send_authentication(&mut conn).await?;
        Ok(())
    }
    async fn send_authentication(&self,  stream: &mut BidirectionalStream) -> Result<()> {
        info!("开始进行认证操作");

        info!("[{:?}] 认证秘钥", &self.digest);

        // 发送hello
        let cmd = Command::Hello {
            service: self.digest
        };
        cmd.write_to(stream).await?;

        // 读取 ack 中的 随机秘钥
        let service_digest = match Command::read_from( stream).await.with_context(|| anyhow!("command ack_token is error"))? {
            Command::AckToken { digest } => digest,
            _ => { return Err(anyhow!("command ack_token is error"));}
        };

        // 发送认证
        let mut concat = Vec::from(self.token);
        concat.extend_from_slice(&service_digest);

        let cmd = Command::ShakeHands {
            digest: utils_digest(&concat)
        };
        cmd.write_to( stream).await.unwrap();

        match Command::read_from(stream).await? {
            Command::AckOk => {
                info!("认证成功, 开始传输")
            }
            Command::AckServiceNotFind => {
                error!("传输服务未查询到, 建立传输失败");
                return Err(anyhow!("service not find"));
            }
            _ => {
                error!("传输认证失败");
                return Err(anyhow!("ack auth filed"));
            }
        }

        info!("正式开始转发服务");

        // todo: we should try some times
        loop {
            match Command::read_from(stream) {
                Command::ControlAck => {
                    info!("start control channel");

                    tokio::spawn(async move {
                        // create a data channel
                    })
                }

                Command::Heartbeat => (),

                _ => {
                    error!("transport protocol error");
                    return Err(anyhow!("transport protocol error"))
                }
            }
        }



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
        //     conn.send(digest).await.unwrap();
        }
    }
}