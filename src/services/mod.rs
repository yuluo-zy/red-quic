use anyhow::Result;
use tokio::sync::broadcast;

pub use handle::ControlChannelMap;

use crate::config::ServiceConfig;

pub mod service;
pub mod multi_map;
pub mod handle;

pub async fn run(config: ServiceConfig, sll_shutdown_rx: broadcast::Receiver<bool>) -> Result<()> {

    Ok(())
}