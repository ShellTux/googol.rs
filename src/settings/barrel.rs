use serde::Deserialize;
use std::net::SocketAddr;

/// Configuration settings for the Barrel component.
///
/// This struct holds the network address and file path for the Barrel service.
/// It is deserialized from configuration files or strings using Serde.
#[derive(Debug, Deserialize)]
pub struct BarrelConfig {
    /// The socket address (IP + port) where the Barrel service will listen.
    pub address: SocketAddr,
    /// The file path to store or load data related to the Barrel service.
    pub filepath: String,
}

impl super::Load for BarrelConfig {
    /// The type of item to load, which is `Self`.
    type Item = Self;

    /// Loads the default configuration for Barrel from the `.barrel` file.
    ///
    /// # Returns
    ///
    /// - `Ok(Self)` with the loaded configuration if successful.
    /// - `Err(config::ConfigError)` if loading or deserialization fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use googol::settings::{barrel::BarrelConfig, Load};
    ///
    /// let config = BarrelConfig::default();
    /// ```
    fn default() -> Result<Self::Item, config::ConfigError> {
        Self::load(".barrel")
    }
}

#[cfg(test)]
mod tests {
    use crate::settings::Load;
    use std::str::FromStr; // Assuming Load trait is in crate::settings

    use super::*;

    /// Valid configuration string in TOML format for testing.
    const VALID: &str = r#"
        address = "0.0.0.0:50052"
        filepath = "./.barrel-data.json"
    "#;

    /// Invalid configuration strings for testing error handling.
    const INVALID: [&str; 2] = [
        r#"
        address = "0.0.0.0:50052"
        "#,
        r#"
        filepath = "./.barrel-data.json"
        "#,
    ];

    /// Tests parsing of a valid configuration string.
    #[test]
    fn test_valid_config() {
        let config = BarrelConfig::from_str(VALID);
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(
            config.address,
            SocketAddr::from_str("0.0.0.0:50052").unwrap()
        );
        assert_eq!(config.filepath, "./.barrel-data.json".to_string());
    }

    /// Tests that invalid configuration strings produce errors.
    #[test]
    fn test_invalid_config() {
        for invalid in INVALID {
            let config = BarrelConfig::from_str(invalid);

            assert!(config.is_err());
        }
    }

    /// Tests loading configuration from a file (assuming the file exists).
    #[test]
    fn test_example_config() {
        let config = BarrelConfig::load("examples/config/barrel.toml");

        assert!(config.is_ok());
    }
}
