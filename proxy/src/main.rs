use std::sync::Arc;
use log::info;
use crate::proxy::{ProxyChainConfig, ProxyChainServer};

mod proxy;

#[tokio::main]
async fn main() {
    env_logger::init();
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

