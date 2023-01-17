pub mod client;
pub mod handle;

use tokio::sync::broadcast;
use crate::client::client::Clients;
use crate::config::ClientConfig;
use anyhow::Result;
pub async fn run_client(config: ClientConfig, all_shutdown_rx: broadcast::Receiver<bool>) -> Result<()> {
    let mut client = Clients::build(config).unwrap();
    client.run(all_shutdown_rx).await
}
