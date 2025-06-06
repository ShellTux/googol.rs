use crate::serde::host::{deserialize_hosts, serialize_hosts};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashSet, VecDeque},
    net::SocketAddr,
};
use url::{Host, Url};

/// A filter for domain names, containing whitelist and blacklist of hosts.
///
/// It provides methods to check if a URL's host
/// is whitelisted or blacklisted.
///
/// # Examples
///
/// ```rust
/// use url::{Url, Host};
/// use googol::settings::gateway::DomainsFilter;
/// use std::collections::HashSet;
///
/// // Create a new DomainsFilter with some hosts
/// let whitelist = ["example.com"].iter().map(|d| Host::parse(d).unwrap()).collect();
///
/// let blacklist = ["bad.com"].iter().map(|d| Host::parse(d).unwrap()).collect();
///
/// let filter = DomainsFilter {
///     whitelist,
///     blacklist,
/// };
///
/// // Check if a URL host is whitelisted or blacklisted
/// let url = Url::parse("https://example.com/page").unwrap();
/// assert!(filter.is_whitelisted(&url));
///
/// let url2 = Url::parse("https://bad.com/malicious").unwrap();
/// assert!(filter.is_blacklisted(&url2));
/// ```
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct DomainsFilter {
    /// Set of hosts that are explicitly allowed.
    #[serde(
        serialize_with = "serialize_hosts",
        deserialize_with = "deserialize_hosts"
    )]
    pub whitelist: HashSet<Host>,

    /// Set of hosts that are explicitly disallowed.
    #[serde(
        serialize_with = "serialize_hosts",
        deserialize_with = "deserialize_hosts"
    )]
    pub blacklist: HashSet<Host>,
}

impl DomainsFilter {
    /// Checks if the host of the given URL is present in the blacklist.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to check.
    ///
    /// # Returns
    ///
    /// `true` if the host is in the blacklist, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use url::{Url, Host};
    /// use googol::settings::gateway::DomainsFilter;
    /// use std::collections::HashSet;
    ///
    /// let filter = DomainsFilter {
    ///     whitelist: HashSet::default(),
    ///     blacklist: ["bad.com"].iter().map(|d| Host::parse(d).unwrap()).collect(),
    /// };
    ///
    /// let url = Url::parse("https://bad.com/malicious").unwrap();
    /// assert!(filter.is_blacklisted(&url));
    /// ```
    pub fn is_blacklisted(&self, url: &Url) -> bool {
        if let Some(host) = url.host() {
            self.blacklist.contains(&host.to_owned())
        } else {
            false
        }
    }

    /// Checks if the host of the given URL is present in the whitelist.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to check.
    ///
    /// # Returns
    ///
    /// `true` if the host is in the whitelist, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use url::{Url, Host};
    /// use googol::settings::gateway::DomainsFilter;
    /// use std::collections::HashSet;
    ///
    /// let filter = DomainsFilter {
    ///     whitelist: ["example.com"].iter().map(|d| Host::parse(d).unwrap()).collect(),
    ///     blacklist: HashSet::default(),
    /// };
    ///
    /// let url = Url::parse("https://example.com/page").unwrap();
    /// assert!(filter.is_whitelisted(&url));
    /// ```
    pub fn is_whitelisted(&self, url: &Url) -> bool {
        if let Some(host) = url.host() {
            self.whitelist.contains(&host.to_owned())
        } else {
            false
        }
    }
}

/// Configuration for the Gateway component, including network settings,
/// URL queue, barrels, and domain filters.
///
/// # Examples
///
/// ```rust
/// use std::collections::{VecDeque, HashSet};
/// use url::Url;
/// use googol::settings::gateway::{GatewayConfig, DomainsFilter};
///
/// // Example of creating a GatewayConfig instance manually
/// let config = GatewayConfig {
///     address: "127.0.0.1:8080".parse().unwrap(),
///     queue: ["https://example.com"]
///             .iter()
///             .map(|u| Url::parse(u).unwrap())
///             .collect::<VecDeque<_>>(),
///     barrels: HashSet::new(),
///     domains_filter: DomainsFilter::default(),
/// };
/// ```
///
/// You can also load from a configuration file as shown in the tests.
#[derive(Debug, Deserialize)]
pub struct GatewayConfig {
    /// The socket address the gateway listens on.
    pub address: SocketAddr,
    /// A starting queue of URLs to process.
    pub queue: VecDeque<Url>,
    /// A set of socket addresses representing barrel nodes.
    pub barrels: HashSet<SocketAddr>,
    /// Domain filtering rules.
    pub domains_filter: DomainsFilter,
}

impl super::Load for GatewayConfig {
    type Item = Self;

    /// Loads the configuration from a default file `.gateway`.
    ///
    /// # Returns
    ///
    /// - `Ok(Self)` with the loaded configuration if successful.
    /// - `Err(config::ConfigError)` if loading fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use googol::settings::{gateway::GatewayConfig, Load};
    ///
    /// let config = GatewayConfig::default();
    /// ```
    fn default() -> Result<Self::Item, config::ConfigError> {
        Self::load(".gateway")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Load;
    use url::Host;

    /// A valid configuration string in TOML format for testing.
    const VALID: &str = r#"
        address = "0.0.0.0:50051"
        queue = [ "https://en.wikipedia.org/wiki/Rust_(programming_language)" ]
        barrels = [ "127.0.0.1:50052", "192.168.41.13:50052" ]
        [domains_filter]
        whitelist = ["example.com", "test.org"]
        blacklist = ["bad.com"]
    "#;

    /// Invalid configuration strings for testing error handling.
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

    /// Tests parsing of a valid configuration string.
    #[test]
    fn test_valid_config() {
        let config = GatewayConfig::from_str(VALID);

        assert!(config.is_ok());

        let config = config.unwrap();

        assert_eq!(config.address, "0.0.0.0:50051".parse().unwrap());
        assert_eq!(
            config.queue,
            ["https://en.wikipedia.org/wiki/Rust_(programming_language)"]
                .iter()
                .map(|url| Url::parse(url).unwrap())
                .collect::<VecDeque<_>>(),
        );
        assert_eq!(
            config.barrels,
            ["127.0.0.1:50052", "192.168.41.13:50052"]
                .iter()
                .map(|addr| addr.parse().unwrap())
                .collect(),
        );
        assert_eq!(
            config.domains_filter.whitelist,
            ["example.com", "test.org"]
                .iter()
                .map(|d| Host::parse(d).unwrap())
                .collect()
        );

        assert_eq!(
            config.domains_filter.blacklist,
            ["bad.com"]
                .iter()
                .map(|d| Host::parse(d).unwrap())
                .collect()
        );
    }

    /// Tests domain filtering methods.
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

    /// Tests loading configuration from invalid strings.
    #[test]
    fn test_invalid_config() {
        for invalid in INVALID {
            let config = GatewayConfig::from_str(invalid);

            assert!(config.is_err());
        }
    }

    /// Tests loading configuration from an example file.
    #[test]
    fn test_example_config() {
        let config = GatewayConfig::load("examples/config/gateway.toml");

        assert!(config.is_ok());
    }
}
