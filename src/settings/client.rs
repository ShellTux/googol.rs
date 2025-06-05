use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
pub struct ClientConfig {
    pub gateway: SocketAddr,
    pub max_retries: usize,
}

impl super::Load for ClientConfig {
    type Item = Self;

    fn default() -> Result<Self::Item, config::ConfigError> {
        Self::load(".client")
    }
}

#[cfg(test)]
mod tests {
    use crate::settings::Load;

    use super::*;

    #[test]
    fn test_example_config() {
        let config = ClientConfig::load("examples/config/client.toml");

        assert!(config.is_ok());
    }
}
