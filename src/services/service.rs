use anyhow::Result;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, RwLock};
use s2n_quic::Server;
use tracing::error;
use tracing::info;

use crate::config::{ServerServiceConfig, ServiceConfig};
use crate::services::handle::{ControlChannel, ControlChannelHandle};
use crate::{CERT_PEM, Digest, KEY_PEM};

use crate::utils::{digest};

pub struct Services {
    server: Server,
    config: Arc<ServiceConfig>,
    // services: Arc<78<HashMap<Digest, ServerServiceConfig>>>,
    // control_channels: Arc<RwLock<ControlChannelMap>>,
}

impl Services {
    pub async fn init(_config: ServiceConfig) -> Result<Services> {
        let config = Arc::new(_config);
        let services = Arc::new(RwLock::new(Self::generate_service(&config)));
        // 配置控制通道
        // let control_channels = Arc::new(RwLock::new(ControlChannelMap::new()));
        // 创建 监听套接字
        let socket_addr = config.bind_addr.parse::<SocketAddr>().unwrap();
        // let transport = Endpoint::server(Self::init_endpoint_config().await, socket_addr)?;
        let mut server = Server::builder()
            .with_tls((CERT_PEM,
                       KEY_PEM))?
            .with_io("127.0.0.1:7799")?
            .start().unwrap();
        info!("创建成功");
        Ok(Services {
            config,
            server,
        })
    }


    // 生成 服务列表
    pub fn generate_service(config: &ServiceConfig) -> HashMap<Digest, ServerServiceConfig> {
        let mut map = HashMap::new();
        for item in &config.services {
            map.insert(digest(item.0.as_bytes()), (item.1).clone());
        }
        map
    }

    // 开始运行
    pub async fn run(&mut self, mut shutdown_rx: tokio::sync::broadcast::Receiver<bool>) {

        while let Some(mut connection) = self.server.accept().await {
            let control_channel = ControlChannel::build(connection);
            control_channel.handle().await;
        }
        // loop {
        //     tokio::select! {
        //         ret = self.server.accept() => {
        //             info!("一个新的请求");
        //             match ret {
        //                 Some(mut conn) => {
        //                     // 接受的是 正在创建的内容
        //                      info!("conn...");
        //                     // let service = self.clone();
        //                     let control_channel = ControlChannel::build(conn);
        //                     control_channel.handle().await
        //                 }
        //                 _ => {
        //                      error!("conn error")
        //                 }
        //             }
        //         },
        //         _ = shutdown_rx.recv() => {
        //             info!("Shuting down gracefully...");
        //             break;
        //         },
        //     }
        // }
        // while let Some(mut connection) = self.server.accept().await {
        //     // spawn a new task for the connection
        //     tokio::spawn(async move {
        //         eprintln!("Connection accepted from {:?}", connection.remote_addr());
        //
        //         while let Ok(Some(mut stream)) = connection.accept_bidirectional_stream().await {
        //             // spawn a new task for the stream
        //             tokio::spawn(async move {
        //                 eprintln!("Stream opened from {:?}", stream.connection().remote_addr());
        //
        //                 // echo any data back to the stream
        //                 while let Ok(Some(data)) = stream.receive().await {
        //                     // stream.send(data).await.expect("stream should be open");
        //                     info!("{:?}", data)
        //                 }
        //             });
        //         }
        //     });
        // }
    }
}
