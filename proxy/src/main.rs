use crate::proxy::{ProxyChainConfig, ProxyChainServer};
use common::logging::setup_logging;
use std::sync::Arc;
use tracing::info;

mod proxy;

#[tokio::main]
async fn main() {
    setup_logging("proxy");

    let proxy_server_notifier = Arc::new(tokio::sync::Notify::new());
    ProxyChainServer::new(ProxyChainConfig::new("proxy/config.yaml".into()))
        .run(8100, Arc::clone(&proxy_server_notifier))
        .await;
    proxy_server_notifier.notified().await;

    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl c event");
    info!("exiting");
}
