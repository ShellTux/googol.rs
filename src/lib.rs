pub mod address;
pub mod barrel;
pub mod gateway;
pub mod index_store;
pub mod logger;
pub mod page;
pub mod serde;
pub mod settings;
pub mod top_searches;

#[derive(prost::Enumeration, Debug, PartialEq, Eq)]
enum GoogolStatus {
    Success = 0,
    Error = 1,
    InvalidUrl = 2,
    AlreadyIndexedUrl = 3,
    UnavailableBarrels = 4,
}

pub mod proto {
    tonic::include_proto!("googol");
}
