use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
pub struct ClientConfig {
    pub gateway: SocketAddr,
}

impl super::Load for ClientConfig {
    type Item = Self;

    fn default() -> Result<Self::Item, config::ConfigError> {
        Self::load(".client")
    }
}
