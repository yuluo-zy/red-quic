use s2n_quic::Connection;
use tokio::io::AsyncReadExt;
use tracing::log::{info, log};

pub struct ControlChannel {
    service: Connection,
}

impl ControlChannel {
    pub fn build(service: Connection) -> Self {
        ControlChannel { service  }
    }
    pub async fn handle(mut self) {
        // let rmt_addr = self.remote_address();
        // info!("远程连接{rmt_addr}");
        while let Ok(Some(mut conn)) = self.service.accept_bidirectional_stream().await {
            tokio::spawn(async move {
                eprintln!("Stream opened from {:?}", conn.connection().remote_addr());

                // echo any data back to the stream
                while let Ok(Some(data)) = conn.receive().await {
                    let _ = data.iter().map(|a| info!("{}",1));
                    info!("{:?}", data)
                    // stream.send(data).await.expect("stream should be open");
                }
            });
            // let mut  buf = [0u8; 4];
            // buf.map(|a| info!("{a}") );
            // _reacvStream.read_exact(&mut buf).await;
            // buf.map(|a| info!("{a}") );
        };
    }
}

pub struct ControlChannelHandle {}
