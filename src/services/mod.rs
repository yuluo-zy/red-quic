use tokio::sync::broadcast;

pub use handle::ControlChannelMap;

use crate::config::ServiceConfig;
use crate::services::service::Services;

pub mod service;
pub mod multi_map;
pub mod handle;

pub async fn run_server(config: ServiceConfig, sll_shutdown_rx: broadcast::Receiver<bool>){
    let service = Services::init(config);
}