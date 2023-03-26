use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, oneshot};
use anyhow::{anyhow, Context, Result};
use s2n_quic::{Client, Connection};
use s2n_quic::client::Connect;
use s2n_quic::stream::BidirectionalStream;
use socket2::{SockRef, TcpKeepalive};
use tokio::io::copy_bidirectional;
use tokio::net::TcpStream;
use tokio::time;
use tokio::time::sleep;
use tracing::{debug, error, info};
use tracing::log::{log, warn};
use tracing::log::Level::Debug;
use crate::config::{ClientConfig, ClientServiceConfig};
use crate::{CERT_PEM};
use crate::protocol::{Command, ProtocolDigest};
use crate::services::handle::TCP_SIZE;
use crate::utils::digest as utils_digest;

const KEEPALIVE_DURATION: Duration = Duration::new(20, 0);
const KEEPALIVE_INTERVAL: Duration = Duration::new(8, 0);

pub struct ClientChannel {
    pub(crate) digest: ProtocolDigest,
    // 服务名称
    pub(crate) shutdown_rx: broadcast::Receiver<bool>,
    pub(crate) transport: Connection,
    pub(crate) heartbeat_timeout: u64,             // Application layer heartbeat timeout in secs
}

impl ClientChannel {
    pub async fn build(
        remote_addr: &String,
        server_name: &String,
        shutdown_rx: broadcast::Receiver<bool>,
    ) -> Result<Self> {
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
        let digest = utils_digest(server_name.as_bytes());

        Self::new(connection, digest, shutdown_rx).await
    }

    async fn new(conn: Connection, digest: ProtocolDigest, shutdown_rx: broadcast::Receiver<bool>) -> Result<Self> {
        info!("创建连接");
        let conn = Self {
            digest,
            shutdown_rx,
            transport: conn,
            heartbeat_timeout: 10,
        };
        Ok(conn)
    }
    pub async fn run(&mut self, config: &HashMap<String, ClientServiceConfig>) -> Result<()> {
        // 开始写认证链接并创建相关数据通路的代码
        let mut conn = self.transport.handle().open_bidirectional_stream().await.unwrap();
        self.send_authentication(&mut conn).await?;
        self.transmission(&mut conn, config).await?;
        Ok(())
    }
    async fn send_authentication(&self, stream: &mut BidirectionalStream) -> Result<()> {
        info!("开始进行认证操作");

        info!("[{:?}] 认证秘钥", &self.digest);

        // 发送hello
        let cmd = Command::Hello {
            service: self.digest // 服务名称
        };
        cmd.write_to(stream).await?;

        match Command::read_from(stream).await? {
            Command::AckOk => {
                info!("认证成功, 开始传输")
            }
            Command::ErrorAck { error_type } => {
                // todo: error info output
                error!("传输服务未查询到, 建立传输失败");
                return Err(anyhow!("service not find"));
            }
            _ => {
                error!("传输认证失败");
                return Err(anyhow!("ack auth filed"));
            }
        }
        Ok(())
    }

    async fn transmission(&self, stream: &mut BidirectionalStream, config: &HashMap<String, ClientServiceConfig>) -> Result<()> {
        info!("正式开始转发服务");

        for (key, value) in config.iter() {
           let cmd = Command::DataConnect {
                // todo: add udp 1
                transmission_type: 0,
                port: value.service.port as u64,
            };
            cmd.write_to(stream).await?;
        }
        // todo: we should try some times
        loop {
            tokio::select! {
                Ok(command) = Command::read_from( stream) => {
                    match command {
                         Command::ErrorAck {error_type} => {
                    info!("start control channel");

                        // tokio::spawn(async move {
                        //     // create a data channel
                        //    Ok(())
                        // })
                }

                Command::Heartbeat => (),

                _ => {
                    error!("transport protocol error");
                    return Err(anyhow!("transport protocol error"))
                }}

                },

                _ = time::sleep(Duration::from_secs(self.heartbeat_timeout)), if self.heartbeat_timeout > 0 => {
                    return Err(anyhow!("heartbeat timeout!"))
                }

            }
        }

        Ok(())
    }
}

async fn data_channel_tcp(mut stream: BidirectionalStream, local_host: &str) -> Result<()> {
    info!("建立tcp转发连接");
    let mut local = TcpStream::connect(local_host).await.with_context(|| format!("Failed to connect to {}", local_host))?;
    copy_bidirectional(&mut stream, &mut local).await?;
    Ok(())
}
