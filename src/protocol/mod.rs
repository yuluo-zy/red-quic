use anyhow::{anyhow, Result};
use bytes::{BufMut};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tracing::error;

pub const HASH_WIDTH_IN_BYTES: usize = 32;

pub type ProtocolDigest = [u8; HASH_WIDTH_IN_BYTES];

#[derive(Debug)]
pub enum Command {
    // 认证握手阶段
    Hello {
        service: ProtocolDigest,
    },
    AckOk,
    DataConnect {
        transmission_type: u8,
        port: u64
    },
    Heartbeat,
    ErrorAck {
        error_type: u32
    }
}

impl Command {
    const TYPE_HEART_BEAT: u8 = 0x00;
    const TYPE_HELLO: u8 = 0x01;
    const TYPE_ACK_OK: u8 = 0x02;
    const TYPE_DATA_CONNECT: u8 = 0x03;
    const TYPE_ERROR_ACK: u8 = 0x04;

    const HASH_WIDTH_IN_BYTES: usize = 32;

    pub async fn read_from<R>(r: &mut R) -> Result<Self>
        where
            R: AsyncRead + Unpin,
    {
        let cmd = r.read_u8().await?;
        match cmd {
            Self::TYPE_HEART_BEAT => {
                Ok(Self::Heartbeat)
            }
            Self::TYPE_HELLO => {
                let mut digest = [0; 32];
                r.read_exact(&mut digest).await?;
                Ok(Self::Hello{
                    service: digest
                })
            }
            Self::TYPE_ACK_OK => {
                Ok(Self::AckOk)
            }
            Self::TYPE_DATA_CONNECT => {
                let transmission_type =  r.read_u8().await?;
                let port =  r.read_u64().await?;
                Ok(Self::DataConnect {
                    transmission_type,
                    port
                })
            }
            Self::TYPE_ERROR_ACK => {
                Ok(Self::ErrorAck {
                    error_type: r.read_u32().await?
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
        w.flush().await?;
        Ok(())
    }

    fn write_to_buf<B: BufMut>(&self, buf: &mut B) {
        match self {
            Command::Heartbeat => {
                buf.put_u8(Self::TYPE_HEART_BEAT)
            }
            Command::Hello {service}  => {
                buf.put_u8(Self::TYPE_HELLO);
                buf.put_slice(service)
            }
            Command::AckOk => {
                buf.put_u8(Self::TYPE_ACK_OK)
            }
            Command::DataConnect { transmission_type, port } => {
                buf.put_u8(Self::TYPE_DATA_CONNECT);
                buf.put_u8(*transmission_type);
                buf.put_u64(*port);
            }

            Command::ErrorAck {error_type} => {
                buf.put_u8(Self::TYPE_ACK_OK);
                buf.put_u32(*error_type)
            }
            _ => {
                todo!()
            }
        }
    }

    fn serialized_len(&self) -> usize {
        1 + match self {
            Command::Hello{..}  =>  32,
            Command::ErrorAck {..}=> 1 + 4,
            Command::DataConnect {..} => 1 + 8,
            Command::Heartbeat
            | Command::AckOk => 0,
        }
    }
}
