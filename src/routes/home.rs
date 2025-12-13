use actix_web::{HttpRequest, Responder, get};
use log::debug;
use tera::Context;

use crate::tera::render_template;

#[get("/home")]
pub async fn home_get(req: HttpRequest) -> impl Responder {
    debug!("{:#?}", req);

    let context = Context::new();

    render_template("home.html", context).await
}
