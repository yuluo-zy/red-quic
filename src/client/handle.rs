use tokio::sync::oneshot;
use anyhow::Result;
use quinn::Endpoint;
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
    pub(crate) shutdown_rx: oneshot::Receiver<u8>,
    pub(crate) remote_addr: String,
    // `client.remote_addr`
    pub(crate) transport: Endpoint,
    pub(crate) heartbeat_timeout: u64,             // Application layer heartbeat timeout in secs
}

impl ClientChannel {
    pub async fn run(&mut self) -> Result<()> {
        // let mut
        // 开始写认证链接并创建相关数据通路的代码
        Ok(())
    }
}