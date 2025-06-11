//! The main module for the FishFish system.
//!
//! Provides functionality to manage domain categories and phishing.

use crate::{debugv, errorv};
use domain::{FishDomain, category::FishDomainCategory};
use log::{debug, error};
use std::collections::HashMap;
use url::Host;

pub mod domain;

/// Represents the main structure managing host to fish domain mappings.
#[derive(Debug)]
pub struct FishFish {
    host2domain: HashMap<Host, Option<FishDomain>>,
}

impl Default for FishFish {
    fn default() -> Self {
        Self::new()
    }
}

impl FishFish {
    /// Creates a new instance of `FishFish`.
    pub fn new() -> Self {
        Self {
            host2domain: HashMap::new(),
        }
    }

    /// Retrieves the category of the domain associated with the given host.
    ///
    /// This method first checks the cache; if the domain info is not cached,
    /// it performs an HTTP request to fetch data from the API.
    ///
    /// # Arguments
    ///
    /// * `host` - The host for which to determine the domain category.
    ///
    /// # Returns
    ///
    /// A `FishDomainCategory` indicating the category of the domain.
    pub async fn domain_category(&mut self, host: &Host) -> FishDomainCategory {
        match self.host2domain.get(host) {
            Some(fish_domain) => match fish_domain {
                Some(fish_domain) => fish_domain.category,
                None => FishDomainCategory::Unknown,
            },
            None => {
                let url = format!("https://api.fishfish.gg/v1/domains/{}", host);

                let response = reqwest::get(url).await.expect("Failed to send request");

                if !response.status().is_success() {
                    self.host2domain.insert(host.clone(), None);
                    return FishDomainCategory::Unknown;
                }

                let response = response.text().await.unwrap();

                debugv!(response);

                match serde_json::from_str::<FishDomain>(&response) {
                    Ok(domain) => {
                        self.host2domain.insert(host.clone(), Some(domain.clone()));

                        domain.category
                    }
                    Err(e) => {
                        errorv!(e);

                        FishDomainCategory::Unknown
                    }
                }
            }
        }
    }
}

//#[tokio::test]
//async fn test_fishfish_new() {
//    let mut fishfish = FishFish::new();
//
//    for (domain, category) in [("stieamcommunitiy.com", FishDomainCategory::Phishing)]
//        .iter()
//        .map(|(d, c)| (Host::parse(d).unwrap(), c))
//    {
//        assert_eq!(fishfish.domain_category(&domain).await, *category);
//    }
//}
