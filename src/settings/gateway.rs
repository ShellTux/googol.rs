use crate::serde::host::{deserialize_hosts, serialize_hosts};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashSet, VecDeque},
    net::SocketAddr,
};
use url::{Host, Url};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct DomainsFilter {
    #[serde(
        serialize_with = "serialize_hosts",
        deserialize_with = "deserialize_hosts"
    )]
    pub whitelist: HashSet<Host>,

    #[serde(
        serialize_with = "serialize_hosts",
        deserialize_with = "deserialize_hosts"
    )]
    pub blacklist: HashSet<Host>,
}

impl DomainsFilter {
    pub fn is_blacklisted(&self, url: &Url) -> bool {
        if let Some(host) = url.host() {
            self.blacklist.contains(&host.to_owned())
        } else {
            false
        }
    }

    pub fn is_whitelisted(&self, url: &Url) -> bool {
        if let Some(host) = url.host() {
            self.whitelist.contains(&host.to_owned())
        } else {
            false
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GatewayConfig {
    pub address: SocketAddr,
    pub queue: VecDeque<Url>,
    pub barrels: HashSet<SocketAddr>,
    pub domains_filter: DomainsFilter,
}

impl super::Load for GatewayConfig {
    type Item = Self;
}

#[cfg(test)]
mod tests {
    use std::{net::SocketAddr, str::FromStr};

    use url::Host;

    use crate::settings::Load;

    use super::*;

    const VALID: &str = r#"
        address = "0.0.0.0:50051"
        queue = [ "https://en.wikipedia.org/wiki/Rust_(programming_language)" ]
        barrels = [ "127.0.0.1:50052", "192.168.41.13:50052" ]
        [domains_filter]
        whitelist = ["example.com", "test.org"]
        blacklist = ["bad.com"]
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

        assert_eq!(
            config.address,
            SocketAddr::from_str("0.0.0.0:50051").unwrap()
        );

        assert_eq!(
            config.queue,
            VecDeque::from_iter(
                ["https://en.wikipedia.org/wiki/Rust_(programming_language)"]
                    .iter()
                    .map(|url| Url::parse(url).unwrap()),
            )
        );

        assert_eq!(
            config.barrels,
            HashSet::from_iter(
                ["127.0.0.1:50052", "192.168.41.13:50052"]
                    .map(|addr| SocketAddr::from_str(addr).unwrap()),
            )
        );

        assert_eq!(
            config.domains_filter.whitelist,
            vec!["example.com", "test.org"]
                .iter()
                .map(|d| Host::parse(d).unwrap())
                .collect()
        );

        assert_eq!(
            config.domains_filter.blacklist,
            vec!["bad.com"]
                .iter()
                .map(|d| Host::parse(d).unwrap())
                .collect()
        );
    }

    #[test]
    fn test_url_domain() {
        let config = GatewayConfig::from_str(VALID).unwrap();

        let url1 = Url::parse("https://example.com/search?q=rust").unwrap();
        let url2 = Url::parse("https://bad.com/foo/bar/search?q=rust").unwrap();

        assert!(config.domains_filter.is_whitelisted(&url1));
        assert!(!config.domains_filter.is_blacklisted(&url1));
        assert!(!config.domains_filter.is_whitelisted(&url2));
        assert!(config.domains_filter.is_blacklisted(&url2));
    }

    #[test]
    fn test_invalid_config() {
        for invalid in INVALID {
            let config = GatewayConfig::from_str(invalid);

            assert!(config.is_err());
        }
    }
}
