use std::net::{Ipv4Addr, SocketAddr};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::oneshot;
use anyhow::Result;
use bytes::Bytes;
use s2n_quic::{Client, Connection};
use s2n_quic::client::Connect;
use tracing::info;
use tracing::log::{debug, log, warn};
use crate::config::ClientConfig;
use crate::Digest;

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
        // let bind_addr =  SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0));
        // let remote_addr = config.remote_addr.parse::<SocketAddr>().unwrap();
        let client = Client::builder()
            .with_tls(Path::new("C:\\Users\\Administrator\\CLionProjects\\red-quic\\src\\key\\cert.pem"))?
            .with_io("0.0.0.0:0")?
            .start().unwrap();
        info!("构建本地监听内容");

        let addr: SocketAddr =config.remote_addr.parse()?;
        let connect = Connect::new(addr).with_server_name("tea.yuluo.website");
        info!("发送1");
        let mut connection = client.connect(connect).await?;
        info!("发送2");
        // ensure the connection doesn't time out with inactivity
        connection.keep_alive(true)?;
        info!("发送3");
        let mut client_channel = Self::new(connection, config).await;
        info!("发送4");


        // open a new stream and split the receiving and sending sides
        let stream = client_channel.transport.open_bidirectional_stream().await?;
        info!("发送5");
        let (mut receive_stream, mut send_stream) = stream.split();
        info!("发送6");
        let b = Bytes::from("nishism");
        send_stream.send(b).await?;

        // spawn a task that copies responses from the server to stdout
        // tokio::spawn(async move {
        //     let mut stdout = tokio::io::stdout();
        //     let _ = tokio::io::copy(&mut receive_stream, &mut stdout).await;
        // });
        //
        // // copy data from stdin and send it to the server
        // let mut stdin = tokio::io::stdin();
        // tokio::io::copy(&mut stdin, &mut send_stream).await?;

        info!("连接远程地址");
        // let conn = match conn.into_0rtt() {
        //     Ok((conn, _) ) => conn,
        //     Err(conn) =>{
        //         warn!("[relay] [connection] Unable to convert the connection into 0-RTT");
        //         conn.await.unwrap()
        //     }
        // };

        // ClientChannel{digest: self.config.default_token.unwrap(),
        // shutdown_rx: shutdown_rx.clone(),
        // remote_addr: self.config.remote_addr.clone(),
        // transport: (),
        // heartbeat_timeout: self.config.heartbeat_timeout}
        // let client_channel = Self::new(conn, config).await;
        Ok(client_channel)
    }

    async fn new(conn:Connection, config: ClientConfig) -> Self {
        let conn = Self {
            digest: config.default_token.unwrap(),
            transport: conn,
            heartbeat_timeout: 10
        } ;
        info!("创建连接");
        // 提交任务开始
        // tokio::spawn(Self::send_authentication(conn.clone()));
        // heartbeat
        // tokio::spawn(Self::heartbeat(conn.clone(), config.heartbeat_interval));
        conn
    }
    pub async fn run(&mut self) -> Result<()> {
        // let mut
        // 开始写认证链接并创建相关数据通路的代码
        Ok(())
    }
    // async fn send_authentication(self)  {
    //     async fn send_token(conn: &Connection, token: String) -> Result<()> {
    //         info!("开始认证");
    //         let (mut _sendStream, mut _reacvStream)  = conn.open_bi().await?;
    //         let vec = [4u8, 2u8, 3u8,4u8,4u8, 2u8, 3u8,4u8];
    //         _sendStream.write_all(&vec).await?;
    //         info!("发送完毕");
    //         _sendStream.finish().await?;
    //         Ok(())
    //     }
    //
    //     match send_token(&self.transport, self.digest).await {
    //         Ok(_) => {
    //             debug!("[relay] [connection] [authentication]")
    //         }
    //         Err(err) => {
    //             warn!("[relay] [connection] [authentication] {err}")
    //         }
    //     }
    //
    // }
}