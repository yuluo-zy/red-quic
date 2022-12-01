use tokio::sync::broadcast;

use crate::config::ServiceConfig;
use crate::services::service::Services;

pub mod handle;
pub mod service;

pub async fn run_server(config: ServiceConfig, all_shutdown_rx: broadcast::Receiver<bool>) {
    let mut service = Services::init(config).await.unwrap();
    service.run(all_shutdown_rx).await
}
