use crate::proto;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Page {
    pub href: String,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub icon: Option<String>,
}

impl From<proto::Page> for Page {
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
        }
    }
}

impl Into<proto::Page> for Page {
    fn into(self) -> proto::Page {
        proto::Page {
            url: self.href,
            title: self.title.unwrap_or(String::from("")),
            summary: self.summary.unwrap_or(String::from("")),
            icon: self.icon.unwrap_or(String::from("")),
        }
    }
}
