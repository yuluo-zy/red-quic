pub mod client;
pub mod config;
pub mod protocol;
pub mod services;
pub mod utils;

const HASH_WIDTH_IN_BYTES: usize = 32;
pub type Digest = [u8; HASH_WIDTH_IN_BYTES];
pub use anyhow::Result;

pub use config::Config;

pub static CERT_PEM: &str = include_str!(concat!(
env!("CARGO_MANIFEST_DIR"),
"/certs/cert.pem"
));

pub static KEY_PEM: &str = include_str!(concat!(
env!("CARGO_MANIFEST_DIR"),
"/certs/key.pem"
));
