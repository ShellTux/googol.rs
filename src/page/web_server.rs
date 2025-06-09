use crate::{fishfish::domain::category::FishDomainCategory, proto};
use serde::Serialize;

/// Represents a web page with its URL, title, summary, and icon.
/// Supports serialization with Serde and conversion to/from protocol buffer representations.
///
/// # Examples
///
/// Creating a `Page` from a protocol buffer object:
///
/// ```rust
/// use googol::{proto, page::web_server::Page};
///
/// // Assume you have a proto::Page instance
/// let proto_page = proto::Page {
///     url: "https://example.com".to_string(),
///     title: "Example".to_string(),
///     summary: "An example page".to_string(),
///     icon: "icon.png".to_string(),
///     category: "".to_string(),
/// };
///
/// // Convert from proto::Page to Page
/// let page: Page = proto_page.into();
/// assert_eq!(page.href, "https://example.com");
/// assert_eq!(page.title.as_deref(), Some("Example"));
/// ```
///
/// Converting a `Page` into its protocol buffer representation:
///
/// ```rust
/// use googol::{proto, page::web_server::Page};
///
/// let page = Page {
///     href: "https://rust-lang.org".to_string(),
///     title: Some("Rust".to_string()),
///     summary: Some("The Rust Programming Language".to_string()),
///     icon: None,
///     category: None,
/// };
///
/// // Convert to proto::Page
/// let proto_page: proto::Page = page.into();
/// assert_eq!(proto_page.url, "https://rust-lang.org");
/// ```
#[derive(Debug, Serialize)]
pub struct Page {
    /// The URL of the page.
    pub href: String,
    /// Optional title of the page.
    pub title: Option<String>,
    /// Optional summary or description of the page.
    pub summary: Option<String>,
    /// Optional icon URL or identifier.
    pub icon: Option<String>,
    /// Fish domain category
    pub category: Option<FishDomainCategory>,
}

impl From<proto::Page> for Page {
    /// Converts from a protocol buffer `Page` (`proto::Page`) to a `Page`.
    ///
    /// # Arguments
    ///
    /// * `value` - The `proto::Page` to convert from.
    ///
    /// # Returns
    ///
    /// A `Page` instance with fields populated from the protocol buffer.
    fn from(value: proto::Page) -> Self {
        Self {
            href: value.url,
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
            category: FishDomainCategory::from_string(value.category),
        }
    }
}

impl Into<proto::Page> for Page {
    /// Converts a `Page` into its protocol buffer (`proto::Page`) representation.
    ///
    /// # Returns
    ///
    /// A `proto::Page` with fields populated from the `Page`.
    fn into(self) -> proto::Page {
        proto::Page {
            url: self.href,
            title: self.title.unwrap_or_default(),
            summary: self.summary.unwrap_or_default(),
            icon: self.icon.unwrap_or_default(),
            category: match self.category {
                Some(category) => category.to_string(),
                None => "".to_string(),
            },
        }
    }
}
