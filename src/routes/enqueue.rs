use crate::{
    debugv, models::enqueue::EnqueueInput, proto::EnqueueRequest, routes::get_grpc_client,
    tera::TEMPLATES,
};
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use log::{debug, error};
use serde_json::json;
use std::net::SocketAddr;
use tera::Context;
use tonic::Request;

#[get("/enqueue")]
pub async fn enqueue_get(req: HttpRequest) -> impl Responder {
    debug!("{:#?}", req);

    let context = Context::new();

    match TEMPLATES.render("enqueue.html", &context) {
        Ok(rendered) => HttpResponse::Ok().body(rendered),
        Err(e) => {
            error!("tera error: {:#?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[post("/enqueue")]
pub async fn enqueue_post(
    gateway_address: web::Data<SocketAddr>,
    item: web::Either<web::Json<EnqueueInput>, web::Form<EnqueueInput>>,
) -> impl Responder {
    debugv!(item);

    let gateway_address = *gateway_address.into_inner();

    let context = Context::new();

    let url = match &item {
        web::Either::Left(json) => json.url.clone(),
        web::Either::Right(form) => form.url.clone(),
    };

    let json = web::Json(match get_grpc_client(gateway_address).await {
        Err(e) => json!({"error": e.to_string()}),
        Ok(mut client) => {
            let request = Request::new(EnqueueRequest { url });

            match client.enqueue_url(request).await {
                Err(e) => json!({"error": e.to_string()}),
                Ok(_) => json!({"message": "Enqueued"}),
            }
        }
    });
    debugv!(json);

    match item {
        web::Either::Left(web::Json(_)) => HttpResponse::Ok().json(json),
        web::Either::Right(web::Form(_)) => match TEMPLATES.render("enqueue.html", &context) {
            Ok(rendered) => HttpResponse::Ok().body(rendered),
            Err(e) => {
                error!("tera error: {:#?}", e);
                HttpResponse::InternalServerError().finish()
            }
        },
    }
}
