use std::collections::HashMap;
use std::sync::{Arc};
use std::time::Duration;
use s2n_quic::Connection;
use tracing::log::{info};
use crate::services::auth::{IsAuth, IsClosed};
use tokio::time;
use anyhow::{anyhow, Context, Result};
use parking_lot::Mutex;
use crate::protocol::{Command, ProtocolDigest};
use s2n_quic::application::Error as S2N_Error;
use s2n_quic::connection::Handle;
use s2n_quic::stream::BidirectionalStream;
use thiserror::Error;
use tokio::io::{copy_bidirectional};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};
use tokio::time::Sleep;
use tracing::{error, Instrument, Span, warn};
use crate::config::{ServerServiceConfig, ServiceConfig, TransportType};

pub type ControlChannelMap = HashMap<ProtocolDigest, ControlChannelHandle>;

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
pub const HEART_BEATS: u64 = 60;

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
    digest: ProtocolDigest,
    service_channels: Arc<Mutex<ControlChannelMap>>,
    config: Arc<HashMap<ProtocolDigest, ServerServiceConfig>>,
}

impl ControlChannel {
    pub fn build(digest: [u8; 32],
                 config: Arc<HashMap<ProtocolDigest, ServerServiceConfig>>,
                 service_channel_map: Arc<Mutex<ControlChannelMap>>) -> Self {
        info!("????????????????????????");
        ControlChannel {
            is_auth: IsAuth::new(IsClosed::new()),
            digest,
            config,
            service_channels: service_channel_map,
        }
    }
    pub async fn handle(&mut self, conn: Connection) {
        let addr = conn.remote_addr().unwrap();
        info!("[{addr}] ????????????");
        let res = tokio::select! {
                res = self.handle_authentication_timeout(conn.handle(), Duration::new(20,0)) => res,
                res = self.run(conn) => res,
            };
    }

    pub async fn handle_authentication_timeout(&self,
                                               conn: Handle,
                                               timeout: Duration) -> Result<()> {
        info!("????????????");
        let auth_success = tokio::select! {
           is_auth =  self.is_auth.clone() => is_auth,
           _=  time::sleep(timeout) => false
       };
        info!("????????????");
        if auth_success {
            info!("????????????");
            Ok(())
        } else {
            let error = HandleError::AuthenticationTimeout;
            conn.close(error.clone().into());
            self.is_auth.wake();
            let rmt_addr = conn.remote_addr().unwrap();
            error!("[{rmt_addr}] ??????????????????!");
            Err(anyhow!(error))
        }
    }
    pub async fn run(&self, mut conn: Connection) -> Result<()> {
        info!("run");
        while let Ok(Some(mut data_stream)) = conn.accept_bidirectional_stream().await {
            info!("handshake");
            self.handshake(&mut data_stream).await?;
            self.handle_connection(data_stream).await?;
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
                        info!("???????????? {:?}", addr);
                        self.is_auth.set_auth();
                    } else {
                        error!("????????????");
                        self.is_auth.set_close();
                    }
                    self.is_auth.wake();
                }
                Ok(())
            }
            _ => {
                error!("????????????");
                Err(anyhow!(HandleError::AuthenticationFailed))
            }
        }
    }

    pub async fn handle_connection(&self, mut stream: BidirectionalStream) -> Result<()> {
        let cmd = Command::read_from(&mut stream).await?;
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
                                    digest: ProtocolDigest,
                                    mut stream: BidirectionalStream) -> Result<()> {
        if self.is_auth.clone().await {
            // ?????????????????????, ????????????key?????????????????????
            let service_config = self.config.get(&digest).unwrap();
            // ??????????????????ack, ????????????????????????
            let cmd = Command::ControlAck;
            cmd.write_to(&mut stream).await?;
            // ??????????????????, ?????????????????? ????????????????????????.
            {
                let mut channel_map = self.service_channels.lock();
                let handle = ControlChannelHandle::build(stream, service_config.clone(), HEART_BEATS).await?;
                channel_map.insert(digest, handle);
            }
        }
        Ok(())
    }
}

pub struct ControlChannelHandle {
    shutdown_tx: broadcast::Sender<bool>,
    data_ch_tx: mpsc::Sender<TcpStream>,
    service_config: ServerServiceConfig,
}


impl ControlChannelHandle {
    pub async fn build(
        stream: BidirectionalStream,
        service_config: ServerServiceConfig,
        heartbeats: u64,
    ) -> Result<Self> {
        let (shutdown_tx, shutdown_rx) = broadcast::channel::<bool>(1);
        let (data_ch_tx, data_ch_rx) = mpsc::channel(CHAN_SIZE * 2);
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
        let bind_addr = service_config.port.clone();
        // ?????? ?????????
        match service_config.transport_type {
            TransportType::Tcp => {
                tokio::spawn(
                    async move {
                        if let Err(e) = run_tcp_pool(
                            bind_addr,
                            data_ch_rx,
                            data_req_tx,
                            shutdown_rx_clone,
                        ).await
                            .with_context(|| "Failed to run TCP connection pool")
                        {
                            error!("{:#}", e);
                        }
                    });
            }
            TransportType::Udp => {}
        }

        let mut ch = ControlChannelRunner::build(
            stream,
            data_req_rx,
            shutdown_rx,
            heartbeats,
        )?;

        tokio::spawn(async move {
            if let Err(e) = ch.run().await {
                error!(" runner error {:?}", e);
            }
        });

        Ok(Self {
            service_config: service_config.clone(),
            data_ch_tx,
            shutdown_tx,
        })
    }
}


pub struct ControlChannelRunner {
    stream: BidirectionalStream,
    data_req_rx: mpsc::UnboundedReceiver<bool>,
    shutdown_rx: broadcast::Receiver<bool>,
    heartbeats: u64,
}

impl ControlChannelRunner {
    async fn data_start(&mut self) -> Result<()> {
        let data_cmd = Command::DateCreate;
        data_cmd.write_to(&mut self.stream).await
    }

    async fn heart_beats(&mut self) -> Result<()> {
        let heartbeat = Command::Heartbeat;
        heartbeat.write_to(&mut self.stream).await
    }

    pub fn build(
        stream: BidirectionalStream,
        data_req_rx: mpsc::UnboundedReceiver<bool>,
        shutdown_rx: broadcast::Receiver<bool>,
        heartbeats: u64,
    ) -> Result<Self> {
        Ok(Self {
            stream,
            data_req_rx,
            shutdown_rx,
            heartbeats,
        })
    }

    pub async fn run(
        &mut self,
    ) -> Result<()> {
        loop {
            tokio::select! {
                    new_req = self.data_req_rx.recv() => {
                        match new_req {
                            Some(_) => {
                                // ???????????????????????????
                                if let Err(error) = self.data_start().await {
                                    error!("{:?}", error);
                                    break;
                                }
                            },
                            None =>{
                                break;
                            }
                        }
                    },
                       _ = time::sleep(Duration::from_secs(self.heartbeats)), if self.heartbeats != 0 => {
                            if let Err(e) = self.heart_beats().await {
                                error!("{:#}", e);
                                break;
                            }
                }
                    _ = self.shutdown_rx.recv() => {
                        // ?????????????????????, ????????????
                        break;
                    }
                }
        }
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
                // ?????? ????????????????????????  ????????????
                if cmd.write_to(&mut stream).await.is_ok() {
                    // ??????????????????
                    info!("????????????{:?} : {:?}", &visitor.peer_addr()?.clone(), &stream.peer_addr()?.clone());
                    tokio::spawn(async move {
                        let _ = copy_bidirectional(&mut visitor, &mut stream);
                    });
                    break;
                } else if data_req_tx.send(true).is_err() {
                    // ???????????? ?????????????????????????????????, ????????????????????????????????????
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

        // ????????????????????????
        loop {
            tokio::select! {
                connect = stream.accept() =>{
                    match connect {
                        Ok((_socket, addr)) => {
                            if data_ch_req_tx.send(true).with_context(|| "Failed to send data chan create request").is_err() {
                                break;
                            }
                            info!("new client form : {}", &addr);
                            // ????????????????????????????????? ????????????
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