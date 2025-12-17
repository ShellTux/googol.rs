use derive_more::{Add, Debug, From};
use std::fmt;

#[derive(PartialEq, From, Add, Debug, Eq, Default)]
pub struct Bytes(pub usize);

impl fmt::Display for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const KB: usize = 1 << 10; // 1024
        const MB: usize = 1 << 20; // 1024 * 1024
        const GB: usize = 1 << 30; // 1024 * 1024 * 1024

        let bytes = self.0;

        let (value, unit) = if bytes >= GB {
            (bytes as f64 / GB as f64, "GB")
        } else if bytes >= MB {
            (bytes as f64 / MB as f64, "MB")
        } else if bytes >= KB {
            (bytes as f64 / KB as f64, "KB")
        } else {
            (bytes as f64, "Bytes")
        };

        write!(f, "{:.2} {}", value, unit)
    }
}
