use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

/// Represents the category of a domain in the FishFish system.
///
/// The `FishDomainCategory` enum categorizes domains into predefined types,
/// such as safe, malware, phishing, or unknown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FishDomainCategory {
    /// Safe domains
    Safe,
    /// Domains associated with malware
    Malware,
    /// Domains used for phishing attacks
    Phishing,
    /// Unknown category
    //#[serde(skip)]
    Unknown,
}

impl FishDomainCategory {
    pub fn from_string(s: String) -> Option<Self> {
        match s.len() {
            0 => None,
            _ => match s.parse::<Self>() {
                Ok(fish_domain) => Some(fish_domain),
                Err(_) => None,
            },
        }
    }
}

impl FromStr for FishDomainCategory {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "safe" => Ok(Self::Safe),
            "malware" => Ok(Self::Malware),
            "phishing" => Ok(Self::Phishing),
            "unknown" => Ok(Self::Unknown),
            _ => match s.to_lowercase().as_str() {
                "safe" => Ok(Self::Safe),
                "malware" => Ok(Self::Malware),
                "phishing" => Ok(Self::Phishing),
                "unknown" => Ok(Self::Unknown),
                _ => Err(()),
            },
        }
    }
}

impl fmt::Display for FishDomainCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FishDomainCategory::Safe => "safe",
                FishDomainCategory::Malware => "malware",
                FishDomainCategory::Phishing => "phishing",
                FishDomainCategory::Unknown => "unknown",
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_to_string() {
        use FishDomainCategory::*;

        assert_eq!(Safe.to_string(), "safe");
        assert_eq!(Malware.to_string(), "malware");
        assert_eq!(Phishing.to_string(), "phishing");
        assert_eq!(Unknown.to_string(), "unknown");
    }

    #[test]
    fn test_category_from_string() {
        use FishDomainCategory::*;

        assert_eq!("safe".parse(), Ok(Safe));
        assert_eq!("malware".parse(), Ok(Malware));
        assert_eq!("phishing".parse(), Ok(Phishing));
        assert_eq!("unknown".parse(), Ok(Unknown));
        assert_eq!("".parse::<FishDomainCategory>(), Err(()));
        assert_eq!("asdfadsf".parse::<FishDomainCategory>(), Err(()));
    }
}
