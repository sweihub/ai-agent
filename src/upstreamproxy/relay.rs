// Source: /data/home/swei/claudecode/openclaudecode/src/upstreamproxy/relay.ts
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct Relay {
    config: RelayConfig,
    status: Arc<RwLock<RelayStatus>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayConfig {
    pub listen_host: String,
    pub listen_port: u16,
    pub target_host: String,
    pub target_port: u16,
    pub session_token: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RelayStatus {
    Starting,
    Running,
    Stopped,
    Error,
}

impl Relay {
    pub fn new(config: RelayConfig) -> Self {
        Self {
            config,
            status: Arc::new(RwLock::new(RelayStatus::Starting)),
        }
    }

    pub async fn start(&self) -> Result<(), String> {
        let mut status = self.status.write().await;
        *status = RelayStatus::Running;
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), String> {
        let mut status = self.status.write().await;
        *status = RelayStatus::Stopped;
        Ok(())
    }

    pub async fn get_status(&self) -> RelayStatus {
        *self.status.read().await
    }
}

pub async fn start_upstream_proxy_relay(config: RelayConfig) -> Result<Relay, String> {
    let relay = Relay::new(config);
    relay.start().await?;
    Ok(relay)
}
