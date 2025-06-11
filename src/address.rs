use std::{fmt, net::SocketAddr, str::FromStr};

/// A wrapper around `SocketAddr` providing custom display and default behavior.
///
/// `Address` encapsulates a `SocketAddr` and provides implementations for `Default`
/// and `Display` traits, along with a constructor method.
#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub struct Address(SocketAddr);

impl Default for Address {
    /// Creates a default `Address` set to localhost (`127.0.0.1`) on port `8080`.
    ///
    /// # Examples
    ///
    /// ```
    /// use googol::address::Address;
    /// use std::{fmt, net::SocketAddr, str::FromStr};
    ///
    /// let default_addr = Address::default();
    ///
    /// assert_eq!(default_addr, Address::new(SocketAddr::from_str("127.0.0.1:8080").unwrap()));
    /// println!("{}", default_addr); // Outputs: "127.0.0.1:8080"
    /// ```
    fn default() -> Self {
        Self(SocketAddr::from_str("127.0.0.1:8080").unwrap())
    }
}

impl fmt::Display for Address {
    /// Formats the `Address` for user-friendly display.
    ///
    /// This implementation outputs the inner `SocketAddr` using its `Debug` formatting.
    ///
    /// # Examples
    ///
    /// ```
    /// use googol::address::Address;
    /// use std::{fmt, net::SocketAddr, str::FromStr};
    ///
    /// let addr = Address::default();
    /// println!("{}", addr); // Outputs: "127.0.0.1:8080"
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Address {
    /// Creates a new `Address` from a given `SocketAddr`.
    ///
    /// # Arguments
    ///
    /// * `address` - The `SocketAddr` to encapsulate.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{SocketAddr, ToSocketAddrs};
    /// use googol::address::Address;
    ///
    /// let socket_addr: SocketAddr = "192.168.1.1:1234".parse().unwrap();
    /// let addr = Address::new(socket_addr);
    /// println!("{}", addr); // Outputs: "SocketAddr { ip: V4(192.168.1.1), port: 1234 }"
    /// ```
    pub fn new(address: SocketAddr) -> Self {
        Self(address)
    }
}
