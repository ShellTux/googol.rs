use crate::{
    debugv,
    models::hackernews::{HackerNewsBody, HackerNewsDB, HackerNewsQuery},
    tera::render_template,
};
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use log::debug;
use std::error::Error;
use tera::Context;

#[get("/hackernews")]
async fn hackernews_get(req: HttpRequest) -> impl Responder {
    debug!("{:#?}", req);

    let context = Context::new();

    render_template("hackernews.html", context).await
}

#[post("/hackernews")]
async fn hackernews_post(
    hn_db: web::Data<HackerNewsDB>,
    input: web::Either<web::Json<HackerNewsBody>, web::Form<HackerNewsQuery>>,
) -> Result<impl Responder, Box<dyn Error>> {
    debugv!(input);

    let mut context = Context::new();

    let hn_db = &mut hn_db.lock().await;
    debugv!(&hn_db);

    hn_db.fetch_top_stories_if_expired().await?;

    let keywords: Vec<String> = match input {
        web::Either::Left(web::Json(ref json)) => json.words.clone(),
        web::Either::Right(web::Form(ref form)) => form
            .query
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .collect(),
    };

    let results = hn_db.search(&keywords);
    debugv!(results);

    context.insert("results", &results);

    Ok(match input {
        web::Either::Left(web::Json(_)) => HttpResponse::Ok().json(results),
        web::Either::Right(web::Form(_)) => render_template("hackernews.html", context).await,
    })
}
