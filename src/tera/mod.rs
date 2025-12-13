use actix_web::HttpResponse;
use lazy_static::lazy_static;
use log::error;
use tera::{Context, Tera};

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec![".html", ".sql"]);
        tera
    };
}

pub async fn render_template(template: &str, context: Context) -> HttpResponse {
    match TEMPLATES.render(template, &context) {
        Ok(rendered) => HttpResponse::Ok().body(rendered),
        Err(e) => {
            error!("tera error: {:#?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
