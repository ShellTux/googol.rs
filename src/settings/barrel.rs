use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
pub struct BarrelConfig {
    pub address: SocketAddr,
    pub filepath: String,
}

impl super::Load for BarrelConfig {
    type Item = Self;
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::settings::Load;

    use super::*;

    const VALID: &str = r#"
        address = "0.0.0.0:50052"
        filepath = "./.barrel-data.json"
    "#;

    const INVALID: [&str; 2] = [
        r#"
        address = "0.0.0.0:50052"
            "#,
        r#"
        filepath = "./.barrel-data.json"
            "#,
    ];

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

    #[test]
    fn test_invalid_config() {
        for invalid in INVALID {
            let config = BarrelConfig::from_str(invalid);

            assert!(config.is_err());
        }
    }
}
