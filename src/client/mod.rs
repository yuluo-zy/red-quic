pub mod handle;

use std::sync::Arc;
use tokio::sync::broadcast;
use crate::client::client::Clients;
use crate::config::ClientConfig;
use anyhow::Result;
use crate::client::handle::ClientChannelHandle;

pub async fn run_client(config: ClientConfig, all_shutdown_rx: broadcast::Receiver<bool>) -> Result<()> {
    let config_rc = Arc::new(config);
    let handle = ClientChannelHandle::build( config_rc.clone()).await?;
}
