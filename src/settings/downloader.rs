use serde::Deserialize;
use std::{collections::HashSet, net::SocketAddr};

/// Configuration settings for the Downloader component.
///
/// This struct includes the number of threads to spawn, the gateway address
/// for connecting, and a set of stop words to filter out during processing.
/// It is deserialized from configuration files or strings using Serde.
#[derive(Debug, Deserialize)]
pub struct DownloaderConfig {
    /// The number of worker threads to spawn for downloading.
    pub threads: usize,
    /// The socket address (IP + port) of the gateway the downloader connects to.
    pub gateway: SocketAddr,
    /// A set of stop words to be ignored or filtered during processing.
    pub stop_words: HashSet<String>,
}

impl super::Load for DownloaderConfig {
    /// The type of item to load, which is `Self`.
    type Item = Self;

    /// Loads the default downloader configuration from the `.downloader` file.
    ///
    /// # Returns
    ///
    /// - `Ok(Self)` with the loaded configuration if successful.
    /// - `Err(config::ConfigError)` if loading or deserialization fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use googol::settings::{downloader::DownloaderConfig, Load};
    ///
    /// let config = DownloaderConfig::default();
    /// ```
    fn default() -> Result<Self::Item, config::ConfigError> {
        Self::load(".downloader")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Load;

    /// Valid configuration string in TOML format for testing.
    const VALID: &str = r#"
        threads = 4
        gateway = "127.0.0.1:50051"
        stop_words = ["the", "a"]
    "#;

    /// Invalid configuration strings for testing error handling.
    const INVALID: [&str; 3] = [
        r#"
        threads = 4
        "#,
        r#"
        gateway = "127.0.0.1:50051"
        "#,
        r#"
        stop_words = ["the", "a"]
        "#,
    ];

    /// Tests parsing of a valid configuration string.
    #[test]
    fn test_valid_config() {
        let config = DownloaderConfig::from_str(VALID);

        assert!(config.is_ok());

        let config = config.unwrap();

        // Verify the parsed values
        assert_eq!(config.threads, 4);
        assert_eq!(config.gateway, "127.0.0.1:50051".parse().unwrap());
        assert_eq!(
            config.stop_words,
            ["the", "a"].iter().map(|word| word.to_string()).collect()
        );
    }

    /// Tests that invalid configuration strings produce errors.
    #[test]
    fn test_invalid_config() {
        for invalid in INVALID {
            let config = DownloaderConfig::from_str(invalid);

            assert!(config.is_err());
        }
    }

    /// Tests loading configuration from an example file (assuming the file exists).
    #[test]
    fn test_example_config() {
        let config = DownloaderConfig::load("examples/config/downloader.toml");

        assert!(config.is_ok());
    }
}
