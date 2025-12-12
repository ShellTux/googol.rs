use actix_web::{App, HttpRequest, HttpResponse, HttpServer, get, web};
use lazy_static::lazy_static;
use log::{debug, error};
use serde::Serialize;
use std::net::SocketAddr;
use tera::Tera;

#[derive(Serialize)]
struct Product {
    name: String,
    price: f64,
    description: String,
    stock: u32,
}

#[get("/")]
async fn index(tera: web::Data<Tera>, req: HttpRequest) -> HttpResponse {
    debug!("{:#?}", req);

    let mut context = tera::Context::new();
    context.insert("title", "Home Page");

    match tera.render("index.html", &context) {
        Ok(rendered) => HttpResponse::Ok().body(rendered),
        Err(e) => {
            error!("tera error: {:#?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/products")]
async fn products(tera: web::Data<Tera>) -> HttpResponse {
    let products = vec![
        Product {
            name: "Widget".to_string(),
            price: 19.99,
            description: "A useful widget.".to_string(),
            stock: 10,
        },
        Product {
            name: "Gadget".to_string(),
            price: 29.99,
            description: "An essential gadget.".to_string(),
            stock: 5,
        },
    ];

    let mut context = tera::Context::new();
    context.insert("products", &products);

    match tera.render("products.html", &context) {
        Ok(rendered) => HttpResponse::Ok().body(rendered),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("examples/templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec![".html", ".sql"]);
        //tera.register_filter("do_nothing", do_nothing_filter);
        tera
    };
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    debug!("{:?}", &addr);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(TEMPLATES.clone()))
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
            .service(index)
            .service(products)
    })
    .bind(addr)?
    .run()
    .await
}
