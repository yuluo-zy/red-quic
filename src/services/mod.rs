use tokio::sync::broadcast;

use crate::config::ServiceConfig;
use crate::services::service::Services;
use anyhow::Result;
pub mod handle;
pub mod service;

pub async fn run_server(config: ServiceConfig, all_shutdown_rx: broadcast::Receiver<bool>) -> Result<()>{
    let mut service = Services::init(config).await?;
    service.run(all_shutdown_rx).await
}
