//! Representation of a web page, with URL, title, and metadata.
//!
//! Example:
//!
//! ```rust
//! use chrono::Utc;
//! use googol::page::{Page, PageBuilder};
//!
//! let page = PageBuilder::default()
//!     .url("https://example.com".parse().unwrap())
//!     .title("Title")
//!     .timestamp(Utc::now())
//!     .build()
//!     .unwrap();
//! ```

use crate::{fishfish::domain::category::FishDomainCategory, proto};
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use url::Url;

pub mod web_server;

/// Represents a web page with associated metadata such as URL, title, summary,
/// icon, and timestamp. Supports serialization, deserialization, comparison,
/// and conversion to and from protocol buffer representations.
///
/// # Examples
///
/// ```rust
/// use url::Url;
/// use chrono::Utc;
/// use googol::page::{Page, PageBuilder};
///
/// // Create a new page with a URL
/// let page = PageBuilder::default()
///     .url("https://example.com".parse().unwrap())
///     .title("Example")
///     .summary("An example page")
///     .build()
///     .unwrap();
///
/// // Access fields
/// assert_eq!(page.url.as_str(), "https://example.com/");
/// assert_eq!(page.title.as_deref(), Some("Example"));
/// assert_eq!(page.summary.as_deref(), Some("An example page"));
/// ```
///
/// You can serialize and deserialize `Page` instances using Serde:
///
/// ```rust
/// use googol::page::{Page, PageBuilder};
///
/// let page = PageBuilder::default()
///     .url("https://example.com".parse().unwrap())
///     .title("Example")
///     .summary("An example page")
///     .build()
///     .unwrap();
///
/// let json = serde_json::to_string(&page).unwrap();
/// let deserialized: Page = serde_json::from_str(&json).unwrap();
/// assert_eq!(page, deserialized);
/// ```
#[derive(Debug, Clone, Eq, Hash, Builder, Serialize, Deserialize)]
#[allow(clippy::derived_hash_with_manual_eq)]
pub struct Page {
    /// The URL of the page.
    pub url: Url,
    /// Optional title of the page.
    #[builder(setter(into, strip_option), default)]
    pub title: Option<String>,
    /// Optional summary or description of the page.
    #[builder(setter(into, strip_option), default)]
    pub summary: Option<String>,
    /// Optional icon URL or identifier.
    #[builder(setter(into, strip_option), default)]
    pub icon: Option<String>,
    /// The timestamp when the page was indexed.
    #[builder(setter(into, strip_option), default)]
    pub timestamp: DateTime<Utc>,
    /// Fish Domain category
    #[builder(setter(into, strip_option), default)]
    pub category: Option<FishDomainCategory>,
}

impl From<proto::Page> for Page {
    /// Converts from a protocol buffer `Page` to a `Page`.
    ///
    /// # Panics
    ///
    /// Panics if the URL in `proto::Page` is invalid.
    fn from(value: proto::Page) -> Self {
        Self {
            url: value.url.parse().expect("Expecting valid url"),
            title: match value.title.len() {
                0 => None,
                _ => Some(value.title),
            },
            summary: match value.summary.len() {
                0 => None,
                _ => Some(value.summary),
            },
            icon: match value.icon.len() {
                0 => None,
                _ => Some(value.icon),
            },
            timestamp: Utc::now(),
            category: FishDomainCategory::from_string(value.category),
        }
    }
}

impl From<Page> for proto::Page {
    /// Converts a `Page` into its protocol buffer representation.
    fn from(val: Page) -> Self {
        proto::Page {
            url: val.url.to_string(),
            title: val.title.unwrap_or_default(),
            summary: val.summary.unwrap_or_default(),
            icon: val.icon.unwrap_or_default(),
            category: match val.category {
                Some(fish_category) => fish_category.to_string(),
                None => "".to_string(),
            },
        }
    }
}

impl PartialEq for Page {
    /// Checks equality based on URL and date (ignoring time).
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url && self.timestamp.date_naive() == other.timestamp.date_naive()
    }
}

impl PartialOrd for Page {
    /// Orders pages based on their timestamps.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.timestamp.partial_cmp(&other.timestamp)
    }
}

/// Tests for the `Page` struct and its methods.
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_into_proto() {
        let page = PageBuilder::default()
            .url("https://google.com".parse().unwrap())
            .title("example")
            .summary("summary")
            .build()
            .unwrap();

        let proto_page = proto::Page {
            url: "https://google.com/".to_string(),
            title: "example".to_string(),
            summary: "summary".to_string(),
            icon: "".to_string(),
            category: "".to_string(),
        };

        let expected_proto_page: proto::Page = page.into();

        assert_eq!(expected_proto_page, proto_page);
    }

    #[test]
    fn test_from_proto() {
        let page = PageBuilder::default()
            .url("https://google.com".parse().unwrap())
            .title("example")
            .summary("summary")
            .timestamp(Utc::now())
            .build()
            .unwrap();

        let proto_page = proto::Page {
            url: "https://google.com".to_string(),
            title: "example".to_string(),
            summary: "summary".to_string(),
            icon: "".to_string(),
            category: "".to_string(),
        };

        assert_eq!(Page::from(proto_page), page);
    }

    #[test]
    fn test_serialization() {
        let page = PageBuilder::default()
            .url("https://google.com".parse().unwrap())
            .title("Google")
            .build()
            .unwrap();

        let page_json = serde_json::to_string(&page).expect("Serialization failed");

        let page_deserialized: Page =
            serde_json::from_str(&page_json).expect("Deserialization failed");

        assert_eq!(page, page_deserialized);
    }

    #[test]
    fn test_equality() {
        let page1 = PageBuilder::default()
            .url("https://example.com".parse().unwrap())
            .title("Title")
            .timestamp(Utc::now())
            .build()
            .unwrap();
        let page2 = PageBuilder::default()
            .url("https://example.com".parse().unwrap())
            .title("Another Title")
            .timestamp(Utc::now())
            .build()
            .unwrap();

        assert_eq!(page1, page2); // equality based on URL and date

        let date = Utc.with_ymd_and_hms(2025, 6, 1, 0, 0, 0).unwrap();

        let page1 = PageBuilder::default()
            .url("https://example.com".parse().unwrap())
            .timestamp(date)
            .build()
            .unwrap();
        let page2 = PageBuilder::default()
            .url("https://example.com".parse().unwrap())
            .timestamp(date)
            .build()
            .unwrap();

        assert_eq!(page1, page2); // same date

        let later_date = Utc.with_ymd_and_hms(2025, 6, 2, 0, 0, 0).unwrap();
        let page3 = PageBuilder::default()
            .url("https://example.com".parse().unwrap())
            .timestamp(later_date)
            .build()
            .unwrap();

        assert_ne!(page1, page3);
    }

    #[test]
    fn test_ordering() {
        let ts1 = Utc::now() - chrono::Duration::seconds(10);
        let ts2 = Utc::now();

        let page_old = PageBuilder::default()
            .url("https://example.com/1".parse().unwrap())
            .timestamp(ts1)
            .build()
            .unwrap();
        let page_new = PageBuilder::default()
            .url("https://example.com/2".parse().unwrap())
            .timestamp(ts2)
            .build()
            .unwrap();

        assert!(page_old < page_new);
        assert!(page_new > page_old);
    }

    #[test]
    fn test_new_with_current_time() {
        let url = Url::parse("https://example.com").unwrap();
        let page = PageBuilder::default()
            .url("https://example.com".parse().unwrap())
            .title("Title")
            .timestamp(Utc::now())
            .build()
            .unwrap();

        assert_eq!(page.url, url);
        assert!(page.timestamp <= Utc::now());
    }

    #[test]
    fn test_ordering_with_different_timestamps() {
        let page1 = PageBuilder::default()
            .url("https://example.com".parse().unwrap())
            .title("Title")
            .timestamp(DateTime::from_timestamp(0, 0).unwrap())
            .build()
            .unwrap();
        let page2 = PageBuilder::default()
            .url("https://example.com".parse().unwrap())
            .title("Title")
            .timestamp(DateTime::from_timestamp(1, 0).unwrap())
            .build()
            .unwrap();

        assert!(page1 < page2);
    }
}
