use tokio::sync::broadcast;

pub use handle::ControlChannelMap;

use crate::config::ServiceConfig;
use crate::services::service::Services;

pub mod service;
pub mod multi_map;
pub mod handle;

pub async fn run_server(config: ServiceConfig, all_shutdown_rx: broadcast::Receiver<bool>){
    let mut service = Services::init(config).await.unwrap();
    service.run(all_shutdown_rx).await.expect("TODO: panic message");
}