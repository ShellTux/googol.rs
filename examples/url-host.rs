use config::{Config, File, FileFormat};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::collections::HashSet;
use url::{Host, Url};

#[derive(Debug, Serialize, Deserialize)]
struct HostConfig {
    #[serde(
        serialize_with = "serialize_hosts",
        deserialize_with = "deserialize_hosts"
    )]
    pub hosts: HashSet<Host>,
}

/// Serialize `HashSet<Host>` as a list of strings
fn serialize_hosts<S>(hosts: &HashSet<Host>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let host_strings: Vec<String> = hosts.iter().map(|host| host.to_string()).collect();
    host_strings.serialize(serializer)
}

/// Deserialize `HashSet<Host>` from a list of strings
fn deserialize_hosts<'de, D>(deserializer: D) -> Result<HashSet<Host>, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize into Vec<String>
    let host_strings: Vec<String> = Vec::deserialize(deserializer)?;
    let mut hosts = HashSet::new();

    for s in host_strings {
        // Parse each string into url::Host
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

fn main() {
    let url = Url::parse("https://good.com/foo/bar/search?q=rust").unwrap();
    let host = Host::parse("good.com").unwrap();

    dbg!(&url);
    dbg!(&host);

    assert_eq!(url.host_str(), Some(host.to_string().as_str()));

    let hosts: HashSet<Host> = ["example.com", "test.org", "bad.com"]
        .iter()
        .map(|d| Host::parse(d).unwrap())
        .collect();

    dbg!(&hosts);

    let host_config = HostConfig { hosts };

    let host_config_toml = toml::to_string(&host_config).unwrap();
    dbg!(&host_config_toml);

    let input = r#"
        hosts = ["example.com", "test.org", "bad.com"]
    "#;

    let config: HostConfig = Config::builder()
        .add_source(File::from_str(input, FileFormat::Toml))
        .build()
        .unwrap()
        .try_deserialize()
        .unwrap();

    dbg!(&config);
}
