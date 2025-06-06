//! # Googol Crate
//!
//! The `googol` crate provides components for web page indexing, search, and a gRPC server for
//! managing a web index. It includes modules for managing the index store, representing web pages,
//! handling gRPC services, and tracking search popularity.

pub mod address;
pub mod barrel;
pub mod gateway;
pub mod index_store;
pub mod logger;
pub mod page;
pub mod serde;
pub mod settings;
pub mod top_searches;
pub mod url;

#[derive(prost::Enumeration, Debug, PartialEq, Eq)]
/// Response Status for the Googol System
enum GoogolStatus {
    Success = 0,
    /// Generic error
    Error = 1,
    InvalidUrl = 2,
    AlreadyIndexedUrl = 3,
    /// No barrels where online on the time of request
    UnavailableBarrels = 4,
}

/// The `proto` module contains gRPC message and service definitions generated from protobuf files.
pub mod proto {
    //! gRPC message and service definitions.
    //!
    //! Generated code from protobuf files using `tonic`.
    //!
    //! # Example:
    //!
    //! ```rust
    //! use googol::proto::{BacklinksRequest, BacklinksResponse};
    //! ```

    tonic::include_proto!("googol");
}
