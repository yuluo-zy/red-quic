pub mod client;
pub mod config;
pub mod protocol;
pub mod services;
pub mod utils;

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
