use std::io::{Error, ErrorKind};

use anyhow::{anyhow, Result};
use bytes::{BufMut, Bytes, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tracing::error;

pub const HASH_WIDTH_IN_BYTES: usize = 32;
pub type ProtocolDigest = [u8; HASH_WIDTH_IN_BYTES];

#[derive(Debug)]
pub enum Command {
    ShakeHands {
        digest: [u8; 32],
    },
    Connect {
        protocol_type: u8,
        service_digest: [u8; 32],
    },
    ControlAck,
    DataAck,
    TcpAcK,
    UdpAck,
    DateCreate,
    // Ack(bool),
    Heartbeat,
}

impl Command {
    // 添加命令解析长度
    // pub fn Re
    const TYPE_HEART_BEAT: u8 = 0x00;
    const TYPE_CONTROL_HANDS: u8 = 0x01;
    const TYPE_CONTROL_CONNECT: u8 = 0x02;
    const TYPE_DATA_CONNECT: u8 = 0x03;
    const TYPE_CONTROL_ACK: u8 = 0x04;
    const TYPE_DATA_ACK: u8 = 0x05;
    const TYPE_TCP_ACK: u8 = 0x06;
    const TYPE_UDP_ACK: u8 = 0x07;
    const TYPE_DATA_CREATE: u8 = 0x08;
    // const TYPE_DATA_HANDS: u8 = 0x02;
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
            Self::TYPE_CONTROL_CONNECT =>{
                let protocol_type = r.read_u8().await?;
                let mut  service_digest = [0;32];
                r.read_exact(&mut service_digest).await?;
                Ok( Self::Connect {
                    protocol_type,
                    service_digest
                })
            }
            Self::TYPE_CONTROL_ACK => {
                Ok(Self::ControlAck)
            }
            Self::TYPE_DATA_ACK => {
                Ok(Self::DataAck)
            }
            Self::TYPE_TCP_ACK => {
                Ok(Self::TcpAcK)
            }
            Self::TYPE_UDP_ACK => {
                Ok(Self::UdpAck)
            }
            Self::TYPE_DATA_CREATE => {
                Ok(Self::DateCreate)
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
            Command::ShakeHands { digest } => {
                buf.put_u8(Self::TYPE_CONTROL_HANDS);
                buf.put_slice(digest);
            }
            Command::Heartbeat => {
                buf.put_u8(Self::TYPE_HEART_BEAT)
            }
            Command::Connect { service_digest, protocol_type } => {
                buf.put_u8(*protocol_type);
                buf.put_slice(service_digest);
            }
            Command::ControlAck => {
                buf.put_u8(Self::TYPE_CONTROL_ACK)
            }
            Command::DataAck => {
                buf.put_u8(Self::TYPE_CONTROL_ACK)
            }
            Command::TcpAcK => {
                buf.put_u8(Self::TYPE_TCP_ACK)
            }
            Command::UdpAck => {
                buf.put_u8(Self::TYPE_UDP_ACK)
            }
            Command::DateCreate => {
                buf.put_u8(Self::TYPE_DATA_CREATE);
            }
        }
    }

    pub fn serialized_len(&self) -> usize {
        1 + match self {
            Self::ShakeHands { .. } => 32,
            Self::Connect {..} => 1 + 32,
            Self::Heartbeat
            | Self::ControlAck
            | Self::DataAck
            | Self::UdpAck
            | Self::TcpAcK
            | Self::DateCreate => 0,
        }
    }
}
