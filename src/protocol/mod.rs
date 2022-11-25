use std::io::{Error, ErrorKind};

use anyhow::{anyhow, Result};
use bytes::{Bytes, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt};
use tracing::error;

pub enum Command {
    ShakeHands {
        control_channel: bool,
        digest: BytesMut,
    },
    Authenticate {
        digest: BytesMut,
    },
    Ack(bool),
    Heartbeat,
}

impl Command {
    // 添加命令解析长度
    // pub fn Re

    const TYPE_CONTROL_HANDS: u8 = 0x01;
    const TYPE_AUTHENTICATE: u8 = 0x02;

    const HASH_WIDTH_IN_BYTES: usize = 32;

    pub async fn read_from<R>(r: &mut R) -> Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let cmd = r.read_u8().await?;
        match cmd {
            Self::TYPE_CONTROL_HANDS => {
                let mut digest = BytesMut::with_capacity(Self::HASH_WIDTH_IN_BYTES);
                r.read_exact(&mut digest).await?;
                Ok(Self::ShakeHands {
                    control_channel: true,
                    digest,
                })
            }
            _ => {
                error!("Unsupported Server Cmd");

                return Err(anyhow!("Unsupported Server Cmd"));
            }
        }
    }
}
