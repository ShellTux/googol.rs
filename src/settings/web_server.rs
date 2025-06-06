use serde::Deserialize;
use std::net::SocketAddr;

/// Configuration for the web server, including the server's address and the gateway's address.
///
/// This struct is deserializable from configuration files (e.g., TOML) and implements
/// a `Load` trait to load default configurations from a file.
///
/// # Examples
///
/// ```rust
/// use googol::settings::web_server::WebServerConfig;
///
/// // Manually creating a configuration instance
/// let config = WebServerConfig {
///     address: "0.0.0.0:8080".parse().unwrap(),
///     gateway_address: "127.0.0.1:50051".parse().unwrap(),
/// };
///
/// // Accessing the addresses
/// assert_eq!(config.address, "0.0.0.0:8080".parse().unwrap());
/// assert_eq!(config.gateway_address, "127.0.0.1:50051".parse().unwrap());
/// ```
///
/// You can also load the configuration from a file, as demonstrated in the tests.
#[derive(Debug, Deserialize)]
pub struct WebServerConfig {
    /// The address the web server listens on.
    pub address: SocketAddr,
    /// The address of the gateway.
    pub gateway_address: SocketAddr,
}

impl super::Load for WebServerConfig {
    type Item = Self;

    /// Loads the configuration from the default file `.web-server`.
    ///
    /// # Returns
    ///
    /// - `Ok(Self)` if successfully loaded.
    /// - `Err(config::ConfigError)` if loading fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use googol::settings::{web_server::WebServerConfig, Load};
    ///
    /// let config = WebServerConfig::default();
    /// ```
    fn default() -> Result<Self::Item, config::ConfigError> {
        Self::load(".web-server")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Load;
    use std::str::FromStr;

    /// A valid configuration string in TOML format.
    const VALID: &str = r#"
        address = "0.0.0.0:8080"
        gateway_address = "127.0.0.1:50051"
    "#;

    /// Invalid configuration strings for testing error handling.
    const INVALID: [&str; 2] = [
        r#"
        address = "0.0.0.0:8080"
        "#,
        r#"
        gateway_address = "127.0.0.1:50051"
        "#,
    ];

    /// Tests parsing of a valid configuration string.
    #[test]
    fn test_valid_config() {
        let config = WebServerConfig::from_str(VALID);

        assert!(config.is_ok());

        let config = config.unwrap();

        assert_eq!(
            config.address,
            SocketAddr::from_str("0.0.0.0:8080").unwrap()
        );
        assert_eq!(
            config.gateway_address,
            SocketAddr::from_str("127.0.0.1:50051").unwrap()
        );
    }

    /// Tests handling of invalid configuration strings.
    #[test]
    fn test_invalid_config() {
        for invalid in INVALID {
            let config = WebServerConfig::from_str(invalid);

            assert!(config.is_err());
        }
    }

    /// Tests loading configuration from an example file.
    #[test]
    fn test_example_config() {
        let config = WebServerConfig::load("examples/config/web-server.toml");

        assert!(config.is_ok());
    }
}
