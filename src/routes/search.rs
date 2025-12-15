use crate::{
    debugv,
    models::search::{SearchBody, SearchQuery},
    page,
    proto::{SearchRequest, Status},
    routes::get_grpc_client,
    tera::render_template,
};
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use log::{debug, error};
use std::net::SocketAddr;
use tera::Context;

#[get("/search")]
async fn search_get(req: HttpRequest) -> impl Responder {
    debug!("{:#?}", req);

    let context = Context::new();

    render_template("search.html", context).await
}

#[post("/search")]
async fn search_post(
    gateway_address: web::Data<SocketAddr>,
    input: web::Either<web::Json<SearchBody>, web::Form<SearchQuery>>,
) -> impl Responder {
    debugv!(input, debug);

    let gateway_address = *gateway_address.into_inner();

    let mut context = Context::new();

    let words = match input {
        web::Either::Left(web::Json(ref json)) => json.words.clone(),
        web::Either::Right(web::Form(ref form)) => form
            .query
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .collect(),
    };
    debugv!(words);

    let results = match get_grpc_client(gateway_address).await {
        Err(e) => {
            error!("{}", e);
            None
        }
        Ok(mut client) => match client.search(SearchRequest { words }).await {
            Err(e) => {
                error!("error: {}", e);
                None
            }
            Ok(response) => {
                let response = response.into_inner();

                use Status::{AlreadyIndexedUrl, Error, InvalidUrl, Success, UnavailableBarrels};

                match response.status() {
                    Error | InvalidUrl | AlreadyIndexedUrl | UnavailableBarrels => todo!(),
                    Success => {
                        let results: Vec<page::web_server::Page> = response
                            .pages
                            .iter()
                            .cloned()
                            .map(page::web_server::Page::from)
                            .collect();

                        debug!("{:#?}", results);

                        Some(results)
                    }
                }
            }
        },
    }
    .unwrap_or(vec![]);
    debugv!(results);

    context.insert("results", &results);

    match input {
        web::Either::Left(web::Json(_)) => HttpResponse::Ok().json(results),
        web::Either::Right(web::Form(_)) => render_template("search.html", context).await,
    }
}
