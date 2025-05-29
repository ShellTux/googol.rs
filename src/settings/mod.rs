use barrel::BarrelConfig;
use client::ClientConfig;
use config::{Config, ConfigError, File, FileFormat};
use downloader::DownloaderConfig;
use gateway::GatewayConfig;
use serde::{Deserialize, de::DeserializeOwned};

pub mod barrel;
pub mod client;
pub mod downloader;
pub mod gateway;

pub trait Load {
    type Item: DeserializeOwned;

    fn load(file: &str) -> Result<Self::Item, ConfigError> {
        let config = Config::builder()
            .add_source(File::with_name(file))
            .build()?;

        config.try_deserialize()
    }

    fn from_str(input: &str) -> Result<Self::Item, ConfigError> {
        let config = Config::builder()
            .add_source(File::from_str(input, FileFormat::Toml))
            .build()?;

        config.try_deserialize()
    }

    fn default() -> Result<Self::Item, ConfigError> {
        Self::load(".googol")
    }
}

#[derive(Debug, Deserialize)]
pub struct GoogolConfig {
    pub barrel: BarrelConfig,
    pub client: ClientConfig,
    pub downloader: DownloaderConfig,
    pub gateway: GatewayConfig,
}

impl Load for GoogolConfig {
    type Item = Self;
}
