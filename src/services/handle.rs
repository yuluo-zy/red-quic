use std::collections::HashMap;
use std::sync::{Arc};
use std::time::Duration;
use s2n_quic::Connection;
use tracing::log::{info};
use crate::services::auth::{IsAuth, IsClosed};
use tokio::time;
use anyhow::{anyhow, Context, Result};
use parking_lot::Mutex;
use crate::protocol::Command;
use s2n_quic::application::Error as S2N_Error;
use s2n_quic::connection::Handle;
use s2n_quic::stream::BidirectionalStream;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt, copy_bidirectional};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};
use tracing::{error, Instrument, Span, warn};
use crate::config::{ServerServiceConfig, TransportType};


pub type ControlChannelMap = HashMap<[u8; 32], ControlChannelHandle>;

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

pub const CONTROL_CONNECT: u8 = 0u8;
pub const DATA_CONNECT: u8 = 1u8;
pub const CHAN_SIZE: usize = 2048;
pub const TCP_SIZE: usize = 4;
pub const UDP_SIZE: usize = 2;

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
    digest: [u8; 32],
    service_channels: Arc<Mutex<ControlChannelMap>>,
}

impl ControlChannel {
    pub fn build(digest: [u8; 32], service_channel_map: Arc<Mutex<ControlChannelMap>>) -> Self {
        info!("创建端点转发服务");
        ControlChannel {
            is_auth: IsAuth::new(IsClosed::new()),
            digest,
            service_channels: service_channel_map,
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
            self.handshake(&mut data_stream).await?;
            self.handle_connection(&mut data_stream).await?;
        }
        Ok(())
    }

    pub async fn handshake(&self, stream: &mut BidirectionalStream) -> Result<()> {
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
                Ok(())
            }
            _ => {
                error!("认证失败");
                Err(anyhow!(HandleError::AuthenticationFailed))
            }
        }
    }

    pub async fn handle_connection(&self, stream: &mut BidirectionalStream) -> Result<()> {
        let cmd = Command::read_from(stream).await?;
        match cmd {
            Command::Connect {
                protocol_type,
                service_digest
            } => {
                match protocol_type {
                    CONTROL_CONNECT => {
                        self.do_control_channel(service_digest, stream).await?;
                    }
                    DATA_CONNECT => {}
                    _ => {
                        todo!()
                    }
                }
                Ok(())
            }
            _ => {
                Err(anyhow!(HandleError::BadCommand))
            }
        }
    }

    pub async fn do_control_channel(&self,
                                    digest: [u8; 32],
                                    stream: &mut BidirectionalStream) -> Result<()> {
        if self.is_auth.clone().await {
            // 发送给客户端ack, 开始进行实际传输
            let cmd = Command::ControlAck;
            cmd.write_to(stream).await?;
            {
                let mut channel_map = self.service_channels.lock();
                let handle = ControlChannelHandle::build()?;
                channel_map.insert(digest, handle);
            }
        }
        Ok(())
    }
}

pub struct ControlChannelHandle {
    _shutdown_tx: broadcast::Sender<bool>,
    data_ch_tx: mpsc::Sender<TcpStream>,
    service_config: ServerServiceConfig,
}


impl ControlChannelHandle {
    pub fn build(
        stream: &mut BidirectionalStream,
        service_config: ServerServiceConfig,
        heartbeats: u64,
    ) -> Result<Self> {
        let (shutdown_tx, shutdown_rx) = broadcast::channel::<bool>(1);
        let (data_ch_tx, mut data_ch_rx) = mpsc::channel(CHAN_SIZE * 2);
        let (data_req_tx, data_req_rx) = mpsc::unbounded_channel::<bool>();

        let pool_size = match service_config.transport_type {
            TransportType::Tcp => TCP_SIZE,
            TransportType::Udp => UDP_SIZE,
        };

        for _item in 0..pool_size {
            if let Err(error) = data_req_tx.send(true) {
                error!("Failed to request data channel {}", error)
            }
        }
        let shutdown_rx_clone = shutdown_tx.subscribe();
        // 创建 连接池
        match service_config.transport_type {
            TransportType::Tcp => {
                tokio::spawn(
                    async move {
                        if let Err(e) = run_tcp_pool(
                            service_config.port,
                            data_ch_rx,
                            data_req_tx,
                            shutdown_rx_clone,
                        ).await
                            .with_context(|| "Failed to run TCP connection pool")
                        {
                            error!("{:#}", e);
                        }
                    }
                        .instrument(Span::current()));
            }
            TransportType::Udp => {}
        }


        Ok(Self {})
    }

    pub async fn channel_run(
        conn: BidirectionalStream,
        mut data_req_rx: mpsc::UnboundedReceiver<bool>,
        mut shutdown_rx: broadcast::Receiver<bool>,
        heartbeats: u64,
    ) -> Result<()> {
        tokio::spawn(async move {
            let data_cmd = Command::DateCreate;
            let heartbeat = Command::Heartbeat;

            loop {
                tokio::select! {
                    new_req = data_req_rx.recv() => {
                        match new_req {
                            Some(_) => {
                                // 创建对应的数据通道
                                if let Err(error) = data_cmd.write_to(&mut conn).await {
                                    error!("{:?}", error);
                                    break;
                                }
                            },
                            None =>{
                                break;
                            }
                        }
                    },
                       _ = time::sleep(Duration::from_secs(heartbeats)), if heartbeats != 0 => {
                            if let Err(e) = heartbeat.write_to(&mut conn).await {
                                error!("{:#}", e);
                                break;
                            }
                }
                    _ = shutdown_rx.recv() => {
                        // 接收到停止信号, 退出监听
                        break;
                    }
                }
            }
        }.instrument(Span::current()));
        Ok(())
    }
}

pub async fn run_tcp_pool(
    bind_addr: String,
    mut data_ch_rx: mpsc::Receiver<TcpStream>,
    data_req_tx: mpsc::UnboundedSender<bool>,
    shutdown_rx: broadcast::Receiver<bool>,
) -> Result<()> {
    let mut req_connect = create_tcp_connect(
        bind_addr,
        data_req_tx.clone(),
        shutdown_rx,
    ).await?;

    let cmd = Command::TcpAcK;

    'pool: while let Some(mut visitor) = req_connect.recv().await {
        loop {
            if let Some(mut stream) = data_ch_rx.recv().await {
                // 接受 服务器远端访问和  内网连接
                if cmd.write_to(&mut stream).await.is_ok() {
                    // 开启双向复制
                    info!("开启转发{:?} : {:?}", &visitor.peer_addr()?.clone(), &stream.peer_addr()?.clone());
                    tokio::spawn(async move {
                        let _ = copy_bidirectional(&mut visitor, &mut stream);
                    });
                    break;
                } else if data_req_tx.send(true).is_err() {
                    // 这里说明 没有可以使用的数据通道, 所以重新创建一个数据通道
                    break 'pool;
                }
            } else {
                break 'pool;
            }
        }
    }
    info!("end of forwarding");
    Ok(())
}

pub async fn create_tcp_connect(
    addr: String,
    data_ch_req_tx: mpsc::UnboundedSender<bool>,
    mut shutdown_rx: broadcast::Receiver<bool>,
) -> Result<mpsc::Receiver<TcpStream>> {
    let (data_tx, data_rx) = mpsc::channel(CHAN_SIZE);
    tokio::spawn(async move {
        let res_stream = TcpListener::bind(&addr).await;
        let stream = match res_stream {
            Ok(res) => res,
            Err(error) => {
                error!("Failed to request data channel {}", error);
                return;
            }
        };
        info!("listening at {}", &addr);

        // 开始监听创建内容
        loop {
            tokio::select! {
                connect = stream.accept() =>{
                    match connect {
                        Ok((_socket, addr)) => {
                            if data_ch_req_tx.send(true).with_context(|| "Failed to send data chan create request").is_err() {
                                break;
                            }
                            info!("new client form : {}", &addr);
                            // 将接受的连接句柄发送到 使用通道
                            let _ = data_tx.send(_socket);
                        },
                        Err(e) => error!("couldn't get client: {:?}", e),
                    }

                }
                _ = shutdown_rx.recv() => {
                    break;
                }
            }
        }
    });
    Ok(data_rx)
}