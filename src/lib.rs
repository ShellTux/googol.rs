pub mod address;
pub mod barrel;
pub mod gateway;
pub mod settings;
pub mod top_searches;

#[derive(prost::Enumeration)]
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
