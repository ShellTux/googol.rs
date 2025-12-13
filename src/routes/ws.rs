use crate::{
    debugv,
    models::ws::{ClientMessage, Topic},
    proto::RealTimeStatusRequest,
    routes::get_grpc_client,
};
use actix_web::{HttpRequest, Responder, get, web};
use actix_ws::Message;
use futures::StreamExt as _;
use log::debug;
use serde_json::json;
use std::net::SocketAddr;
use tonic::Request;

#[get("/ws")]
async fn ws_handler(
    gateway_address: web::Data<SocketAddr>,
    req: HttpRequest,
    body: web::Payload,
) -> actix_web::Result<impl Responder> {
    debugv!(req);

    let gateway_address = *gateway_address.into_inner();

    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        return;
                    }
                }
                Message::Text(msg) => {
                    let msg = msg.trim();

                    if let Ok(client_message) = serde_json::from_str::<ClientMessage>(msg) {
                        debug!("{:#?}", client_message);

                        match client_message {
                            ClientMessage::Subscribe { topic } => {
                                debug!("topic = {:#?}", topic);

                                match topic {
                                    Topic::Status => {
                                        let mut client =
                                            get_grpc_client(gateway_address).await.unwrap();

                                        loop {
                                            let request = Request::new(RealTimeStatusRequest {});
                                            let response =
                                                client.real_time_status(request).await.unwrap();
                                            let response = response.into_inner();

                                            let json = json!({
                                                "top10_searches": response.top10_searches,
                                                "avg_response_time_ms": response.avg_response_time_ms,
                                                "barrels": response
                                                    .barrels
                                                    .iter()
                                                    .map(|barrel| {
                                                        json!({
                                                            "online": barrel.online,
                                                            "address": barrel.address,
                                                            "index_size_bytes": barrel.index_size_bytes,
                                                        })
                                                    }).collect::<Vec<_>>(),
                                                "queue": response.queue,
                                            });
                                            debug!("{:#?}", json);

                                            session.text(json.to_string()).await.unwrap();
                                        }
                                    }
                                }
                            }
                            ClientMessage::Unsubscribe { topic } => {
                                debug!("topic = {:#?}", topic);

                                todo!()
                            }
                        };
                    } else {
                        println!("Got text: {msg}");
                        session.text(msg).await.unwrap();
                    }
                }
                _ => break,
            }
        }

        let _ = session.close(None).await;
    });

    Ok(response)
}
