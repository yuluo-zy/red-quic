use std::collections::HashMap;
use std::io::Error as IoError;
use std::sync::{Arc};
use std::time::Duration;
use s2n_quic::Connection;
use tracing::log::{info};
use tokio::time;
use anyhow::{anyhow, Context, Result};
use parking_lot::Mutex;
use rand::RngCore;
use crate::protocol::{Command, HASH_WIDTH_IN_BYTES, ProtocolDigest};
use s2n_quic::application::Error as S2N_Error;
use s2n_quic::connection::Handle;
use s2n_quic::stream::BidirectionalStream;
use thiserror::Error;
use tokio::io::{copy_bidirectional};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};
use tracing::{error};
use crate::config::{ErrorAckType, TransportType};
use crate::utils::digest as utils_digest;

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
            HandleError::AuthenticationFailed => {
                S2N_Error::from(2u8)
            }
            HandleError::AuthenticationTimeout => {
                S2N_Error::from(3u8)
            }
            HandleError::BadCommand => {
                S2N_Error::from(4u8)
            }
            // _ => {}
        }
    }
}

pub struct ControlChannel {
    service_channels: Arc<Mutex<ControlChannelMap>>,
}

impl ControlChannel {
    pub fn build(service_channel_map: Arc<Mutex<ControlChannelMap>>) -> Self {
        info!("创建端点转发服务");
        ControlChannel {
            service_channels: service_channel_map,
        }
    }

    pub async fn run(&self, mut conn: Connection) -> Result<()> {
        let addr = conn.remote_addr().unwrap();
        info!("[{addr}] 远程连接");
        // conn.open_bidirectional_stream()
        while let Ok(Some(mut data_stream)) = conn.accept_bidirectional_stream().await {
            info!("开始创建stream");
            let service_digest = self.handshake(&mut data_stream).await?;
            let handle = ControlChannelHandle::build(data_stream, HEART_BEATS).await?;
            {
                let mut channel_map = self.service_channels.lock();
                channel_map.insert(service_digest, handle);
            }
        }
        Ok(())
    }

    pub async fn handshake(&self, stream: &mut BidirectionalStream) -> Result<ProtocolDigest> {
        let service_digest = match Command::read_from(stream).await.with_context(|| anyhow!("command hello is error"))? {
            Command::Hello { service } => service,
            _ => { return Err(anyhow!("command hello is error")); }
        };

        let cmd = Command::AckOk;
        cmd.write_to(stream).await?;
        Ok(service_digest)
    }
}

pub struct ControlChannelHandle {
    shutdown_tx: broadcast::Sender<bool>,
    data_ch_tx: mpsc::Sender<TcpStream>,
}


impl ControlChannelHandle {
    pub async fn build(
        mut stream: BidirectionalStream,
        heartbeats: u64,
    ) -> Result<Self> {
        let (shutdown_tx, shutdown_rx) = broadcast::channel::<bool>(1);
        let (data_ch_tx, data_ch_rx) = mpsc::channel(CHAN_SIZE * 2);
        // let (data_req_tx, data_req_rx) = mpsc::unbounded_channel::<bool>();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                 Ok(command) = Command::read_from(&mut stream) => {
                    match command {
                        Command::DataConnect { transmission_type, port} => {
                            // 通过 客户端传递的指令进行连接池创建和端口监听
                                match transmission_type {
                                    0 => {
                                        // tcp connect
                                    },
                                    1 => {
                                        // udp connect
                                    }
                                }
                        } ,
                        Command::Heartbeat => ()
                    }
                },

                _ =  time::sleep(Duration::from_secs(heartbeats)), if heartbeats > 0 => {
                        return Err(anyhow!("heartbeat timeout!"))
                    }
            }

            }
        });



        // let pool_size = match service_config.transport_type {
        //     TransportType::Tcp => TCP_SIZE,
        //     TransportType::Udp => UDP_SIZE,
        // };

        // for _item in 0..pool_size {
        //     if let Err(error) = data_req_tx.send(true) {
        //         error!("Failed to request data channel {}", error)
        //     }
        // }
        // let shutdown_rx_clone = shutdown_tx.subscribe();
        // let bind_addr = service_config.port.clone();
        // // 创建 连接池
        // match service_config.transport_type {
        //     TransportType::Tcp => {
        //         tokio::spawn(
        //             async move {
        //                 if let Err(e) = run_tcp_pool(
        //                     bind_addr,
        //                     data_ch_rx,
        //                     data_req_tx,
        //                     shutdown_rx_clone,
        //                 ).await
        //                     .with_context(|| "Failed to run TCP connection pool")
        //                 {
        //                     error!("{:#}", e);
        //                 }
        //             });
        //     }
        //     TransportType::Udp => {}
        // }

        // let mut ch = ControlChannelRunner::build(
        //     stream,
        //     data_req_rx,
        //     shutdown_rx,
        //     heartbeats,
        // )?;

        // tokio::spawn(async move {
        //     if let Err(e) = ch.run().await {
        //         error!(" runner error {:?}", e);
        //     }
        // });

        Ok(Self {
            data_ch_tx,
            shutdown_tx,
        })
    }
}

//
// pub struct ControlChannelRunner {
//     stream: BidirectionalStream,
//     data_req_rx: mpsc::UnboundedReceiver<bool>,
//     shutdown_rx: broadcast::Receiver<bool>,
//     heartbeats: u64,
// }

// impl ControlChannelRunner {
//     async fn data_start(&mut self) -> Result<()> {
//         let data_cmd = Command::DateCreate;
//         data_cmd.write_to(&mut self.stream).await
//     }
//
//     async fn heart_beats(&mut self) -> Result<()> {
//         let heartbeat = Command::Heartbeat;
//         heartbeat.write_to(&mut self.stream).await
//     }
//
//     pub fn build(
//         stream: BidirectionalStream,
//         data_req_rx: mpsc::UnboundedReceiver<bool>,
//         shutdown_rx: broadcast::Receiver<bool>,
//         heartbeats: u64,
//     ) -> Result<Self> {
//         Ok(Self {
//             stream,
//             data_req_rx,
//             shutdown_rx,
//             heartbeats,
//         })
//     }
//
//     pub async fn run(
//         &mut self,
//     ) -> Result<()> {
//         loop {
//             tokio::select! {
//                     new_req = self.data_req_rx.recv() => {
//                         match new_req {
//                             Some(_) => {
//                                 // 创建对应的数据通道
//                                 if let Err(error) = self.data_start().await {
//                                     error!("{:?}", error);
//                                     break;
//                                 }
//                             },
//                             None =>{
//                                 break;
//                             }
//                         }
//                     },
//                        _ = time::sleep(Duration::from_secs(self.heartbeats)), if self.heartbeats != 0 => {
//                             if let Err(e) = self.heart_beats().await {
//                                 error!("{:#}", e);
//                                 break;
//                             }
//                 }
//                     _ = self.shutdown_rx.recv() => {
//                         // 接收到停止信号, 退出监听
//                         break;
//                     }
//                 }
//         }
//         Ok(())
//     }
// }


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

    // let cmd = Command::TcpAcK;
    //
    // 'pool: while let Some(mut visitor) = req_connect.recv().await {
    //     loop {
    //         if let Some(mut stream)
    //             = data_ch_rx.recv().await {
    //             // 接受 服务器远端访问和  内网连接
    //             if cmd.write_to(&mut stream).await.is_ok() {
    //                 // 开启双向复制
    //                 info!("开启转发{:?} : {:?}", &visitor.peer_addr()?.clone(), &stream.peer_addr()?.clone());
    //                 tokio::spawn(async move {
    //                     let _ = copy_bidirectional(&mut visitor, &mut stream);
    //                 });
    //                 break;
    //             } else if data_req_tx.send(true).is_err() {
    //                 // 这里说明 写入操作没有成功, 所以重新创建一个数据通道
    //                 break 'pool;
    //             }
    //         } else {
    //             break 'pool;
    //         }
    //     }
    // }
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