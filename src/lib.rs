pub mod config;
pub mod services;
pub mod utils;


pub type Digest = [u8; HASH_WIDTH_IN_BYTES];
pub use anyhow::Result;

pub use config::Config;
pub use serve::run;