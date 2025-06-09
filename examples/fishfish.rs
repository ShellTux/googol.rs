use googol::fishfish::{FishFish, domain::category::FishDomainCategory};
use url::Host;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    pretty_env_logger::init();

    let mut fishfish = FishFish::new();

    //debugv!(&fishfish);

    for (domain, category) in [
        ("stieamcommunitiy.com", FishDomainCategory::Phishing),
        ("google.com", FishDomainCategory::Safe),
        ("google.pt", FishDomainCategory::Unknown),
    ]
    .iter()
    .map(|(d, c)| (Host::parse(d).unwrap(), c))
    {
        assert_eq!(fishfish.domain_category(&domain).await, *category);
    }

    Ok(())
}
