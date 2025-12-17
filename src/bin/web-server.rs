use actix_web::{
    App, HttpServer, middleware,
    web::{self, Redirect},
};
use googol::{
    debugv,
    models::hackernews::HackerNewsDBGuard,
    routes,
    settings::{GoogolConfig, Load, web_server::WebServerConfig},
};
use log::{debug, info, warn};

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    googol::logger::init();

    let settings = match GoogolConfig::default() {
        Err(e) => {
            warn!("{:#?}", e);

            WebServerConfig::default()?
        }

        Ok(config) => config.web_server,
    };
    debugv!(settings, debug);

    info!("Starting web-server at {}...", settings.address);

    let hn_db = HackerNewsDBGuard::new_safe(HackerNewsDBGuard::load()?);

    let hn_db_cloned = hn_db.clone();

    HttpServer::new(move || {
        let gateway_address = settings.gateway_address;

        App::new()
            .app_data(web::Data::new(gateway_address))
            .app_data(web::Data::new(hn_db.clone()))
            .wrap(middleware::Logger::default().log_target("@"))
            .wrap(middleware::Compress::default())
            .service(actix_files::Files::new("/static", "./static"))
            .service(Redirect::new("/", "/home"))
            .service(routes::home::home_get)
            .service(routes::health::health_get)
            .service(routes::enqueue::enqueue_get)
            .service(routes::enqueue::enqueue_post)
            .service(routes::search::search_get)
            .service(routes::search::search_post)
            .service(routes::hackernews::hackernews_get)
            .service(routes::hackernews::hackernews_post)
            .service(routes::ws::ws_handler)
            .service(routes::user_agent::user_agent_get)
    })
    .bind(settings.address)?
    .run()
    .await?;

    hn_db_cloned.read().await.save()?;

    Ok(())
}
