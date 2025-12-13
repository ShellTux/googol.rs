use crate::proto::gateway_service_client::GatewayServiceClient;
use std::net::SocketAddr;
use tonic::transport::{Channel, Error};

pub mod enqueue;
pub mod health;
pub mod home;
pub mod search;
pub mod ws;

pub async fn get_grpc_client(
    gateway_address: SocketAddr,
) -> Result<GatewayServiceClient<Channel>, Error> {
    let gateway_address = format!("http://{}", gateway_address);
    GatewayServiceClient::connect(gateway_address).await
}
