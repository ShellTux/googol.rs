use crate::{debugv, tera::render_template};
use actix_web::{HttpRequest, Responder, get};
use log::debug;
use tera::Context;

#[get("/dashboard")]
pub async fn dashboard_get(req: HttpRequest) -> impl Responder {
    debugv!(req);

    let context = Context::new();

    render_template("dashboard.html", context).await
}
