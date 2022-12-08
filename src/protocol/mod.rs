use std::io::{Error, ErrorKind};

use anyhow::{anyhow, Result};
use bytes::{BufMut, Bytes, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tracing::error;

#[derive(Debug)]
pub enum Command {
    ShakeHands {
        digest: [u8; 32],
    },
    // Ack(bool),
    Heartbeat,
}

impl Command {
    // 添加命令解析长度
    // pub fn Re
    const TYPE_HEART_BEAT: u8 = 0x00;
    const TYPE_CONTROL_HANDS: u8 = 0x01;
    const TYPE_DATA_HANDS: u8 = 0x02;
    // const TYPE_AUTHENTICATE: u8 = 0x02;

    const HASH_WIDTH_IN_BYTES: usize = 32;

    pub async fn read_from<R>(r: &mut R) -> Result<Self>
        where
            R: AsyncRead + Unpin,
    {
        let cmd = r.read_u8().await?;
        match cmd {
            Self::TYPE_CONTROL_HANDS => {
                let mut digest = [0; 32];
                r.read_exact(&mut digest).await?;
                Ok(Self::ShakeHands {
                    digest,
                })
            }
            _ => {
                error!("Unsupported Server Cmd");

                return Err(anyhow!("Unsupported Server Cmd"));
            }
        }
    }

    pub async fn write_to<W>(&self, w: &mut W) -> Result<()> where W: AsyncWrite + Unpin {
        let mut buf = Vec::with_capacity(self.serialized_len());
        self.write_to_buf(&mut buf);
        w.write_all(&buf).await?;
        Ok(())
    }

    fn write_to_buf<B: BufMut>(&self, buf: &mut B) {
        match self {
            Command::ShakeHands { digest } => {
                buf.put_u8(Self::TYPE_CONTROL_HANDS);
                buf.put_slice(digest);
            }
            Command::Heartbeat => {
                buf.put_u8(Self::TYPE_HEART_BEAT)
            }
        }
    }

    pub fn serialized_len(&self) -> usize {
        1 + match self {
            Command::ShakeHands { .. } => 32,
            // Command::Ack(_) => {}
            Command::Heartbeat => 0
        }
    }
}
