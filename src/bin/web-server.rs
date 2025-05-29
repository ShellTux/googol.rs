use actix_web::{
    App, HttpRequest, HttpResponse, HttpServer, Responder, get, middleware, post, web,
};
use actix_ws::Message;
use futures::StreamExt;
use googol::proto::{
    EnqueueRequest, HealthRequest, RealTimeStatusRequest,
    gateway_service_client::GatewayServiceClient,
};
use log::debug;
use serde::Deserialize;
use serde_json::json;
use tonic::{
    Request,
    transport::{Channel, Error},
};

async fn get_grpc_client() -> Result<GatewayServiceClient<Channel>, Error> {
    GatewayServiceClient::connect("http://127.0.0.1:50051").await
}

#[get("/")]
async fn index(req: HttpRequest) -> &'static str {
    debug!("{:#?}", req);

    "Hello world!"
}

#[get("/health")]
async fn health_handler() -> impl Responder {
    web::Json(match get_grpc_client().await {
        Ok(mut client) => match client.health(Request::new(HealthRequest {})).await {
            Ok(_) => json!({"status": "healthy"}),
            Err(e) => json!({"error": e.to_string()}),
        },
        Err(e) => json!({"error": e.to_string()}),
    })
}

#[derive(Debug, Deserialize)]
struct EnqueueInput {
    url: String,
}

#[post("/enqueue")]
async fn enqueue_handler(item: web::Json<EnqueueInput>) -> impl Responder {
    debug!("{:#?}", item);

    web::Json(match get_grpc_client().await {
        Ok(mut client) => {
            let request = Request::new(EnqueueRequest {
                url: item.url.clone(),
            });

            match client.enqueue_url(request).await {
                Ok(_) => json!({"message": "Enqueued"}),
                Err(e) => json!({"error": e.to_string()}),
            }
        }
        Err(e) => json!({"error": e.to_string()}),
    })
}

#[derive(Debug, Clone, Deserialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Topic {
    Status,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
pub enum ClientMessage {
    #[serde(rename = "subscribe")]
    Subscribe { topic: Topic },

    #[serde(rename = "unsubscribe")]
    Unsubscribe { topic: Topic },
    //#[serde(rename = "publish")]
    //Publish { topic: Topic, message: String },
}

#[get("/ws")]
async fn ws_handler(req: HttpRequest, body: web::Payload) -> actix_web::Result<impl Responder> {
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
                                // TODO: hardcoded string
                                let mut client =
                                    GatewayServiceClient::connect("http://127.0.0.1:50051")
                                        .await
                                        .unwrap();

                                loop {
                                    let request = Request::new(RealTimeStatusRequest {});
                                    let response = client.real_time_status(request).await.unwrap();
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
                                                    "index_size": barrel.index_size,
                                                })
                                            }).collect::<Vec<_>>(),
                                        "queue": response.queue,
                                    });
                                    debug!("{:#?}", json);

                                    session.text(json.to_string()).await.unwrap();
                                }
                            }
                            ClientMessage::Unsubscribe { topic } => todo!(),
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default().log_target("@"))
            .wrap(middleware::Compress::default())
            .service(index)
            .service(health_handler)
            .service(enqueue_handler)
            .service(ws_handler)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
