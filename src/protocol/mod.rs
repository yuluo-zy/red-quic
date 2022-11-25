use crate::Digest;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub enum Command {
    ShakeHands {
        control_channel: bool,
        digest: Digest,
    },
    Authenticate {
        digest: Digest,
    },
    Ack(bool),
    Heartbeat,
}

impl Command {
    // 添加命令解析长度
    // pub fn Re
}
