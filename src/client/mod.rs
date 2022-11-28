use tokio::sync::broadcast;
use crate::client::client::Clients;
use crate::config::ClientConfig;

mod client;
mod handle;

pub async fn run_client(config: ClientConfig, all_shutdown_rx:  broadcast::Receiver<bool>) {
    let mut client = Clients::build(config).unwrap();
    client.run(all_shutdown_rx).await
}