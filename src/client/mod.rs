pub mod handle;

use std::sync::Arc;
use tokio::sync::broadcast;
use crate::config::ClientConfig;
use anyhow::Result;
use crate::client::handle::{ClientChannel};

pub async fn run_client(config: ClientConfig, all_shutdown_rx: broadcast::Receiver<bool>) -> Result<()> {
    let mut handle = ClientChannel::build(
        &config.remote_addr,
        &config.service_name,
        all_shutdown_rx).await?;
    handle.run(&config.services).await?;
    Ok(())
}
