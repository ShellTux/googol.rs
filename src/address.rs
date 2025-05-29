use std::{fmt, net::SocketAddr, str::FromStr};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Address(SocketAddr);

impl Default for Address {
    fn default() -> Self {
        Self(SocketAddr::from_str("127.0.0.1:8080").unwrap())
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Address {
    pub fn new(adress: SocketAddr) -> Self {
        Self(SocketAddr::from(adress))
    }
}
