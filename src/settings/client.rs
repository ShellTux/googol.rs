use serde::Deserialize;
use std::net::SocketAddr;

/// Configuration settings for the Client component.
///
/// This struct contains the gateway address and the maximum number of retries
/// for client operations. It is deserialized from configuration files or strings
/// using Serde.
#[derive(Debug, Deserialize)]
pub struct ClientConfig {
    /// The socket address (IP + port) of the gateway the client connects to.
    pub gateway: SocketAddr,
    /// The maximum number of retry attempts for client requests.
    pub max_retries: usize,
}

impl super::Load for ClientConfig {
    /// The type of item to load, which is `Self`.
    type Item = Self;

    /// Loads the default client configuration from the `.client` file.
    ///
    /// # Returns
    ///
    /// - `Ok(Self)` with the loaded configuration if successful.
    /// - `Err(config::ConfigError)` if loading or deserialization fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use googol::settings::{client::ClientConfig, Load};
    ///
    /// let config = ClientConfig::default();
    /// ```
    fn default() -> Result<Self::Item, config::ConfigError> {
        Self::load(".client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Load;

    /// Tests loading configuration from an example file.
    #[test]
    fn test_example_config() {
        let config = ClientConfig::load("examples/config/client.toml");

        assert!(config.is_ok());
    }
}
