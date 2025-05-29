use serde::Deserialize;
use std::{collections::HashSet, net::SocketAddr};

#[derive(Debug, Deserialize)]
pub struct DownloaderConfig {
    pub threads: usize,
    pub gateway: SocketAddr,
    pub stop_words: HashSet<String>,
}

impl super::Load for DownloaderConfig {
    type Item = Self;
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::settings::Load;

    use super::*;

    const VALID: &str = r#"
        threads = 4
        gateway = "127.0.0.1:50051"
        stop_words = ["the", "a"]
    "#;

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

    #[test]
    fn test_valid_config() {
        let config = DownloaderConfig::from_str(VALID);

        assert!(config.is_ok());

        let config = config.unwrap();

        let stop_words = {
            let mut stop_words = HashSet::new();

            stop_words.insert(String::from("the"));
            stop_words.insert(String::from("a"));

            stop_words
        };

        assert_eq!(config.threads, 4);
        assert_eq!(
            config.gateway,
            SocketAddr::from_str("127.0.0.1:50051").unwrap()
        );
        assert_eq!(config.stop_words, stop_words);
    }

    #[test]
    fn test_invalid_config() {
        for invalid in INVALID {
            let config = DownloaderConfig::from_str(invalid);

            assert!(config.is_err());
        }
    }
}
