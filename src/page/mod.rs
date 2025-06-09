//! Representation of a web page, with URL, title, and metadata.
//!
//! Example:
//!
//! ```rust
//! use googol::page::Page;
//!
//! let page = Page::create("https://example.com")
//!     .with_title("Example");
//! ```

use crate::{fishfish::domain::category::FishDomainCategory, proto};
use chrono::{DateTime, Utc};
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
/// use googol::page::Page;
///
/// // Create a new page with a URL
/// let page = Page::create("https://example.com")
///     .with_title("Example")
///     .with_summary("An example page");
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
/// use googol::page::Page;
///
/// let page = Page::create("https://example.com")
///     .with_title("Example")
///     .with_summary("An example page");
///
/// let json = serde_json::to_string(&page).unwrap();
/// let deserialized: Page = serde_json::from_str(&json).unwrap();
/// assert_eq!(page, deserialized);
/// ```
#[derive(Debug, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct Page {
    /// The URL of the page.
    pub url: Url,
    /// Optional title of the page.
    pub title: Option<String>,
    /// Optional summary or description of the page.
    pub summary: Option<String>,
    /// Optional icon URL or identifier.
    pub icon: Option<String>,
    /// The timestamp when the page was indexed.
    pub timestamp: DateTime<Utc>,
    /// Fish Domain category
    pub category: Option<FishDomainCategory>,
}

impl Page {
    /// Creates a new `Page` instance with the specified URL.
    ///
    /// # Arguments
    ///
    /// * `url` - A string slice representing the page URL.
    ///
    /// # Panics
    ///
    /// Panics if the URL cannot be parsed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use googol::page::Page;
    ///
    /// let page = Page::create("https://rust-lang.org");
    /// assert_eq!(page.url.as_str(), "https://rust-lang.org/");
    /// ```
    pub fn create(url: &str) -> Self {
        let mut page = Self::default();
        page.url = Url::parse(url).unwrap();
        page
    }

    /// Sets the title of the page.
    ///
    /// # Arguments
    ///
    /// * `title` - A string slice representing the page title.
    ///
    /// # Returns
    ///
    /// Self, allowing method chaining.
    ///
    /// # Example
    ///
    /// ```rust
    /// use googol::page::Page;
    ///
    /// let page = Page::create("https://example.com").with_title("Example");
    /// assert_eq!(page.title.as_deref(), Some("Example"));
    /// ```
    #[allow(dead_code)]
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Sets the summary of the page.
    ///
    /// # Arguments
    ///
    /// * `summary` - A string slice representing the page summary.
    ///
    /// # Returns
    ///
    /// Self, allowing method chaining.
    ///
    /// # Example
    ///
    /// ```rust
    /// use googol::page::Page;
    ///
    /// let page = Page::create("https://example.com").with_summary("An example page");
    /// assert_eq!(page.summary.as_deref(), Some("An example page"));
    /// ```
    #[allow(dead_code)]
    pub fn with_summary(mut self, summary: &str) -> Self {
        self.summary = Some(summary.to_string());
        self
    }

    /// Sets the icon for the page.
    ///
    /// # Arguments
    ///
    /// * `icon` - A string slice representing the icon URL or identifier.
    ///
    /// # Returns
    ///
    /// Self, allowing method chaining.
    ///
    /// # Example
    ///
    /// ```rust
    /// use googol::page::Page;
    ///
    /// let page = Page::create("https://example.com").with_icon("icon.png");
    /// assert_eq!(page.icon.as_deref(), Some("icon.png"));
    /// ```
    #[allow(dead_code)]
    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    /// Sets the timestamp of the page.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - A `DateTime<Utc>` representing the timestamp.
    ///
    /// # Returns
    ///
    /// Self, allowing method chaining.
    ///
    /// # Example
    ///
    /// ```rust
    /// use chrono::{Utc, TimeZone};
    /// use url::Url;
    /// use googol::page::Page;
    ///
    /// // Create a specific DateTime for June 1, 2025, at midnight UTC
    /// let custom_time = Utc.ymd(2025, 6, 1).and_hms(0, 0, 0);
    ///
    /// // Create a new page and set the timestamp
    /// let page = Page::create("https://example.com")
    ///     .with_title("Example")
    ///     .with_timestamp(custom_time);
    ///
    /// assert_eq!(page.timestamp, custom_time);
    /// ```
    #[allow(dead_code)]
    pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }
}

impl Default for Page {
    /// Creates a default `Page` with a fixed URL, no title, summary, or icon,
    /// and the current UTC time as the timestamp.
    fn default() -> Self {
        Self {
            url: Url::parse("https://example.com").unwrap(),
            title: None,
            summary: None,
            icon: None,
            timestamp: Utc::now(),
            category: None,
        }
    }
}

impl From<proto::Page> for Page {
    /// Converts from a protocol buffer `Page` to a `Page`.
    ///
    /// # Panics
    ///
    /// Panics if the URL in `proto::Page` is invalid.
    fn from(value: proto::Page) -> Self {
        Self {
            url: Url::parse(&value.url).expect("Expecting valid url"),
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

impl Into<proto::Page> for Page {
    /// Converts a `Page` into its protocol buffer representation.
    fn into(self) -> proto::Page {
        proto::Page {
            url: self.url.to_string(),
            title: self.title.unwrap_or_default(),
            summary: self.summary.unwrap_or_default(),
            icon: self.icon.unwrap_or_default(),
            category: match self.category {
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
        let page = Page::create("https://google.com")
            .with_title("example")
            .with_summary("summary");

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
        let page = Page::create("https://google.com")
            .with_title("example")
            .with_summary("summary");

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
        let page = Page::create("https://google.com").with_title("Google");

        let page_json = serde_json::to_string(&page).expect("Serialization failed");

        let page_deserialized: Page =
            serde_json::from_str(&page_json).expect("Deserialization failed");

        assert_eq!(page, page_deserialized);
    }

    #[test]
    fn test_equality() {
        let page1 = Page::create("https://example.com")
            .with_title("Title")
            .with_timestamp(Utc::now());
        let page2 = Page::create("https://example.com")
            .with_title("Another Title")
            .with_timestamp(Utc::now());

        assert_eq!(page1, page2); // equality based on URL and date

        let date = Utc.with_ymd_and_hms(2025, 6, 1, 0, 0, 0).unwrap();

        let page1 = Page::create("https://example.com").with_timestamp(date);
        let page2 = Page::create("https://example.com").with_timestamp(date);

        assert_eq!(page1, page2); // same date

        let later_date = Utc.with_ymd_and_hms(2025, 6, 2, 0, 0, 0).unwrap();
        let page3 = Page::create("https://example.com").with_timestamp(later_date);

        assert_ne!(page1, page3);
    }

    #[test]
    fn test_ordering() {
        let ts1 = Utc::now() - chrono::Duration::seconds(10);
        let ts2 = Utc::now();

        let page_old = Page::create("https://example.com/1").with_timestamp(ts1);
        let page_new = Page::create("https://example.com/2").with_timestamp(ts2);

        assert!(page_old < page_new);
        assert!(page_new > page_old);
    }

    #[test]
    fn test_new_with_current_time() {
        let url = Url::parse("https://example.com").unwrap();
        let page = Page::create("https://example.com")
            .with_title("Title")
            .with_timestamp(Utc::now());

        assert_eq!(page.url, url);
        assert!(page.timestamp <= Utc::now());
    }

    #[test]
    fn test_ordering_with_different_timestamps() {
        let page1 = Page::create("https://example.com")
            .with_title("Title")
            .with_timestamp(DateTime::from_timestamp(0, 0).unwrap());

        let page2 = Page::create("https://example.com")
            .with_title("Title")
            .with_timestamp(DateTime::from_timestamp(1, 0).unwrap());

        assert!(page1 < page2);
    }
}
