use serde::Deserialize;
use std::{
    collections::{HashSet, VecDeque},
    net::SocketAddr,
};
use url::Url;

#[derive(Debug, Deserialize)]
pub struct GatewayConfig {
    pub address: SocketAddr,
    pub queue: VecDeque<Url>,
    pub barrels: HashSet<SocketAddr>,
}

impl super::Load for GatewayConfig {
    type Item = Self;
}

#[cfg(test)]
mod tests {
    use std::{net::SocketAddr, str::FromStr};

    use crate::settings::Load;

    use super::*;

    const VALID: &str = r#"
        address = "0.0.0.0:50051"
        queue = [ "https://en.wikipedia.org/wiki/Rust_(programming_language)" ]
        barrels = [ "127.0.0.1:50052", "192.168.41.13:50052" ]
    "#;

    const INVALID: [&str; 3] = [
        r#"
        address = "0.0.0.0:50051"
            "#,
        r#"
        queue = [ "url1" ]
            "#,
        r#"
        barrels = [ "127.0.0.1:50052", "192.168.41.13:50052" ]
            "#,
    ];

    #[test]
    fn test_valid_config() {
        let config = GatewayConfig::from_str(VALID);

        assert!(config.is_ok());

        let config = config.unwrap();

        let queue = VecDeque::from_iter(
            ["https://en.wikipedia.org/wiki/Rust_(programming_language)"]
                .iter()
                .map(|url| Url::parse(url).unwrap()),
        );
        let barrels = HashSet::from_iter(
            ["127.0.0.1:50052", "192.168.41.13:50052"]
                .map(|addr| SocketAddr::from_str(addr).unwrap()),
        );

        assert_eq!(
            config.address,
            SocketAddr::from_str("0.0.0.0:50051").unwrap()
        );
        assert_eq!(config.queue, queue);
        assert_eq!(config.barrels, barrels);
    }

    #[test]
    fn test_invalid_config() {
        for invalid in INVALID {
            let config = GatewayConfig::from_str(invalid);

            assert!(config.is_err());
        }
    }
}
