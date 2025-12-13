use crate::{proto::HealthRequest, routes::get_grpc_client};
use actix_web::{Responder, get, web};
use serde_json::json;
use std::net::SocketAddr;
use tonic::Request;

#[get("/health")]
async fn health_get(gateway_address: web::Data<SocketAddr>) -> impl Responder {
    let gateway_address = *gateway_address.into_inner();

    web::Json(match get_grpc_client(gateway_address).await {
        Err(e) => json!({"error": e.to_string()}),
        Ok(mut client) => match client.health(Request::new(HealthRequest {})).await {
            Err(e) => json!({"error": e.to_string()}),
            Ok(res) => json!({"status": res.into_inner().status}),
        },
    })
}
