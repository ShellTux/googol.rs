use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::collections::HashSet;
use url::Host;

/// Serialize `HashSet<Host>` as a list of strings
pub fn serialize_hosts<S>(hosts: &HashSet<Host>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let host_strings: Vec<String> = hosts.iter().map(|host| host.to_string()).collect();
    host_strings.serialize(serializer)
}

/// Deserialize `HashSet<Host>` from a list of strings
pub fn deserialize_hosts<'de, D>(deserializer: D) -> Result<HashSet<Host>, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize into Vec<String>
    let host_strings: Vec<String> = Vec::deserialize(deserializer)?;
    let mut hosts = HashSet::new();

    for s in host_strings {
        match Host::parse(&s) {
            Ok(host) => {
                hosts.insert(host);
            }
            Err(_) => {
                // Handle parse error, optional: return error or skip
                return Err(de::Error::custom(format!("Invalid host string: {}", s)));
            }
        }
    }
    Ok(hosts)
}
