pub mod config;
pub mod services;
pub mod utils;
pub mod protocol;

const HASH_WIDTH_IN_BYTES: usize = 32;
pub type Digest = [u8; HASH_WIDTH_IN_BYTES];
pub use anyhow::Result;

pub use config::Config;