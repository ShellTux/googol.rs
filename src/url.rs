pub use url::Url;

pub fn parse_url_panic(url: &&str) -> Url {
    Url::parse(url).unwrap()
}
