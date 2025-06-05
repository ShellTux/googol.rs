use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
pub struct WebServerConfig {
    pub address: SocketAddr,
    pub gateway_address: SocketAddr,
}

impl super::Load for WebServerConfig {
    type Item = Self;
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::settings::Load;

    const VALID: &str = r#"
        address = "0.0.0.0:8080"
        gateway_address = "127.0.0.1:50051"
    "#;

    const INVALID: [&str; 2] = [
        r#"
        address = "0.0.0.0:8080"
            "#,
        r#"
        gateway_address = "127.0.0.1:50051"
            "#,
    ];

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

    #[test]
    fn test_invalid_config() {
        for invalid in INVALID {
            let config = WebServerConfig::from_str(invalid);

            assert!(config.is_err());
        }
    }

    #[test]
    fn test_example_config() {
        let config = WebServerConfig::load("examples/config/web-server.toml");

        assert!(config.is_ok());
    }
}
