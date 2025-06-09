use category::FishDomainCategory;
use serde::{Deserialize, Serialize};

pub mod category;

/// Represents a domain with associated metadata.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FishDomain {
    /// The time the domain was added
    pub added: i64,
    /// The category of the domain
    pub category: FishDomainCategory,
    /// The time the domain was last checked
    pub checked: i64,
    /// The description of the domain
    pub description: String,
    /// The domain
    pub domain: Option<String>,
    /// The target of the domain
    pub target: Option<String>,
}
