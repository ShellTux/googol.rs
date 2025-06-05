use std::cmp::Ordering;

use crate::proto;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

pub mod web_server;

#[derive(Debug, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct Page {
    pub url: Url,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub icon: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl Page {
    pub fn create(url: &str) -> Self {
        let mut page = Self::default();
        page.url = Url::parse(url).unwrap();

        page
    }

    #[allow(dead_code)]
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());

        self
    }

    #[allow(dead_code)]
    pub fn with_summary(mut self, summary: &str) -> Self {
        self.summary = Some(summary.to_string());

        self
    }

    #[allow(dead_code)]
    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());

        self
    }

    #[allow(dead_code)]
    fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = timestamp;

        self
    }
}

impl Default for Page {
    fn default() -> Self {
        Self {
            url: Url::parse("https://example.com").unwrap(),
            title: None,
            summary: None,
            icon: None,
            timestamp: Utc::now(),
        }
    }
}

impl From<proto::Page> for Page {
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
        }
    }
}

impl Into<proto::Page> for Page {
    fn into(self) -> proto::Page {
        proto::Page {
            url: self.url.to_string(),
            title: self.title.unwrap_or("".to_string()),
            summary: self.summary.unwrap_or("".to_string()),
            icon: self.icon.unwrap_or("".to_string()),
        }
    }
}

impl PartialEq for Page {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url && self.timestamp.date_naive() == other.timestamp.date_naive()
    }
}

impl PartialOrd for Page {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.timestamp.partial_cmp(&other.timestamp)
    }
}

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

        assert_eq!(page1, page2); // equality based on URL

        let page1 = Page::create("https://example.com")
            .with_timestamp(Utc.with_ymd_and_hms(2025, 6, 1, 0, 0, 0).unwrap());
        let page2 = Page::create("https://example.com")
            .with_timestamp(Utc.with_ymd_and_hms(2025, 6, 1, 12, 0, 0).unwrap());

        // equality based on date

        assert_eq!(page1, page2);

        let page2 = Page::create("https://example.com")
            .with_timestamp(Utc.with_ymd_and_hms(2025, 6, 2, 0, 0, 0).unwrap());
        assert_ne!(page1, page2);
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

    //#[test]
    //fn test_new_page() {
    //    let url = Url::parse("https://example.com").unwrap();
    //    let page = Page::new(url.clone(), Some("Title".to_string()), None, None);
    //    assert_eq!(page.url, url);
    //    assert!(page.timestamp > 0);
    //}
}
