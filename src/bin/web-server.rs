use actix_web::{App, HttpRequest, HttpServer, Responder, get, middleware, post, web};
use actix_ws::Message;
use futures::StreamExt;
use googol::{
    debugv, page,
    proto::{
        EnqueueRequest, HealthRequest, RealTimeStatusRequest, SearchRequest, Status,
        gateway_service_client::GatewayServiceClient,
    },
    settings::{GoogolConfig, Load, web_server::WebServerConfig},
};
use log::{debug, error, info};
use serde::Deserialize;
use serde_json::json;
use std::net::SocketAddr;
use tonic::{
    Request,
    transport::{Channel, Error},
};

async fn get_grpc_client(
    gateway_address: SocketAddr,
) -> Result<GatewayServiceClient<Channel>, Error> {
    let gateway_address = format!("http://{}", gateway_address);
    GatewayServiceClient::connect(gateway_address).await
}

#[get("/")]
async fn index(req: HttpRequest) -> &'static str {
    debug!("{:#?}", req);

    "Hello world!"
}

#[get("/health")]
async fn health_handler(gateway_address: web::Data<SocketAddr>) -> impl Responder {
    let gateway_address = *gateway_address.into_inner();

    web::Json(match get_grpc_client(gateway_address).await {
        Err(e) => json!({"error": e.to_string()}),
        Ok(mut client) => match client.health(Request::new(HealthRequest {})).await {
            Err(e) => json!({"error": e.to_string()}),
            Ok(_) => json!({"status": "healthy"}),
        },
    })
}

#[derive(Debug, Deserialize)]
struct EnqueueInput {
    url: String,
}

#[post("/enqueue")]
async fn enqueue_handler(
    gateway_address: web::Data<SocketAddr>,
    item: web::Json<EnqueueInput>,
) -> impl Responder {
    debugv!(item);

    let gateway_address = *gateway_address.into_inner();

    let json = web::Json(match get_grpc_client(gateway_address).await {
        Err(e) => json!({"error": e.to_string()}),
        Ok(mut client) => {
            let request = Request::new(EnqueueRequest {
                url: item.url.clone(),
            });

            match client.enqueue_url(request).await {
                Err(e) => json!({"error": e.to_string()}),
                Ok(_) => json!({"message": "Enqueued"}),
            }
        }
    });
    debugv!(json);

    json
}

#[derive(Debug, Deserialize)]
struct SearchBody {
    words: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SearchParams {
    words: String,
}

#[get("/search")]
async fn search_handler(
    gateway_address: web::Data<SocketAddr>,
    input: web::Either<web::Json<SearchBody>, web::Query<SearchParams>>,
) -> impl Responder {
    debugv!(input, debug);

    let gateway_address = *gateway_address.into_inner();

    let words = match input {
        web::Either::Left(json) => json.into_inner().words,
        web::Either::Right(params) => params
            .into_inner()
            .words
            .split(',')
            .filter(|word| !word.is_empty())
            .map(|word| word.to_string())
            .collect(),
    };
    debugv!(words);

    let json = web::Json(match get_grpc_client(gateway_address).await {
        Err(e) => json!({"error": e.to_string()}),
        Ok(mut client) => {
            let request = SearchRequest { words };

            match client.search(request).await {
                Err(e) => json!({"error": e.to_string()}),
                Ok(response) => {
                    let response = response.into_inner();

                    match response.status() {
                        Status::Success => {
                            let results: Vec<page::web_server::Page> = response
                                .pages
                                .iter()
                                .cloned()
                                .map(page::web_server::Page::from)
                                .collect();

                            debug!("{:#?}", results);

                            json!(results)
                        }
                        _ => json!({"error": "Error searching"}),
                    }
                }
            }
        }
    });
    debugv!(json);

    json
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
}

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

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let settings = match GoogolConfig::default() {
        Err(e) => {
            error!("{:#?}", e);

            WebServerConfig::default()?
        }

        Ok(config) => config.web_server,
    };
    debugv!(settings, debug);

    info!("Starting web-server at {}...", settings.address);

    HttpServer::new(move || {
        let gateway_address = settings.gateway_address;

        App::new()
            .app_data(web::Data::new(gateway_address))
            .wrap(middleware::Logger::default().log_target("@"))
            .wrap(middleware::Compress::default())
            .service(index)
            .service(search_handler)
            .service(health_handler)
            .service(enqueue_handler)
            .service(ws_handler)
    })
    .bind(settings.address)?
    .run()
    .await?;

    Ok(())
}
