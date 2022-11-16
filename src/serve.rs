use crate::Config;
use anyhow::Result;
use tokio::sync::broadcast;

pub async fn run(config: Config, sll_shutdown_rx: broadcast::Receiver<bool>) -> Result<()> {
    Ok(())
}
