pub mod client;
pub mod config;
pub mod protocol;
pub mod services;
pub mod utils;

const HASH_WIDTH_IN_BYTES: usize = 32;
pub type Digest = [u8; HASH_WIDTH_IN_BYTES];
pub use anyhow::Result;

pub use config::Config;
