//! # Googol Configuration Module
//!
//! This module provides mechanisms to load and deserialize configuration data
//! for the Googol application from various sources such as files or strings.
//! It supports multiple submodules representing different components of the system,
//! such as `barrel`, `client`, `downloader`, `gateway`, and `web_server`.
//!
//! # Main Components
//!
//! - `Load` trait: Defines methods to load configuration data from files or strings.
//! - `GoogolConfig` struct: Encapsulates configuration for all components.
//!
//! # Usage
//!
//! Implementations allow loading configuration from a file or string with default fallback.
//! Example usage:
//!
//! ```rust
//! use googol::settings::{GoogolConfig, Load};
//!
//! let config = GoogolConfig::load("config.toml");
//! match config {
//!     Ok(cfg) => println!("Loaded config: {:?}", cfg),
//!     Err(e) => eprintln!("Failed to load config: {}", e),
//! }
//! ```
//!
//! # Modules
//!
//! The module re-exports submodules:
//! - `barrel`
//! - `client`
//! - `downloader`
//! - `gateway`
//! - `web_server`
//!
//! Each module contains specific configuration options relevant to its component.

use barrel::BarrelConfig;
use client::ClientConfig;
use config::{Config, ConfigError, File, FileFormat};
use downloader::DownloaderConfig;
use gateway::GatewayConfig;
use serde::{Deserialize, de::DeserializeOwned};
use web_server::WebServerConfig;

pub mod barrel;
pub mod client;
pub mod downloader;
pub mod gateway;
pub mod web_server;

/// Trait for loading configuration data from files or strings.
///
/// Implemented for types that can be deserialized from configuration files or strings.
/// Provides default loading behavior.
pub trait Load {
    /// The type of the configuration item.
    type Item: DeserializeOwned;

    /// Loads configuration from a file specified by `file`.
    ///
    /// # Arguments
    ///
    /// * `file` - Path to the configuration file.
    ///
    /// # Returns
    ///
    /// `Ok(Self::Item)` if successful, or a `ConfigError` if loading or deserialization fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use googol::settings::{GoogolConfig, Load};
    ///
    /// let config = GoogolConfig::load("googol"); // Load googol.toml
    /// ```
    fn load(file: &str) -> Result<Self::Item, ConfigError> {
        Config::builder()
            .add_source(File::with_name(file))
            .build()?
            .try_deserialize()
    }

    /// Loads configuration from a string input, expected to be in TOML format.
    ///
    /// # Arguments
    ///
    /// * `input` - String containing the configuration data.
    ///
    /// # Returns
    ///
    /// `Ok(Self::Item)` if successful, or a `ConfigError` if parsing or deserialization fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use googol::settings::{GoogolConfig, Load};
    ///
    /// let toml_str = r#"
    /// [section]
    /// key = "value"
    /// "#;
    ///
    /// let config = GoogolConfig::from_str(toml_str);
    /// assert!(config.is_err());
    /// ```
    fn from_str(input: &str) -> Result<Self::Item, ConfigError> {
        Config::builder()
            .add_source(File::from_str(input, FileFormat::Toml))
            .build()?
            .try_deserialize()
    }

    /// Loads configuration from a default file.
    ///
    /// # Returns
    ///
    /// `Ok(Self::Item)` if successful, or a `ConfigError` if loading/deserialization fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use googol::settings::{GoogolConfig, Load};
    ///
    /// let default_config = GoogolConfig::default();
    /// ```
    fn default() -> Result<Self::Item, ConfigError>;
}

/// Main configuration struct aggregating configurations for all components.
///
/// This struct is deserialized from configuration sources
/// and contains nested configurations for each subsystem.
#[derive(Debug, Deserialize)]
pub struct GoogolConfig {
    /// Configuration for the `barrel` component.
    pub barrel: BarrelConfig,
    /// Configuration for the `client` component.
    pub client: ClientConfig,
    /// Configuration for the `downloader` component.
    pub downloader: DownloaderConfig,
    /// Configuration for the `gateway` component.
    pub gateway: GatewayConfig,
    /// Configuration for the `web_server` component.
    pub web_server: WebServerConfig,
}

impl Load for GoogolConfig {
    type Item = Self;

    fn default() -> Result<Self::Item, ConfigError> {
        Self::load(".googol")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests loading a sample configuration file.
    #[test]
    fn test_example_config() {
        let config = GoogolConfig::load("example.googol.toml");

        assert!(config.is_ok(), "Failed to load example configuration");
    }
}
