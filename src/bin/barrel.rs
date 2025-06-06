use googol::{
    barrel::Barrel,
    debugv,
    proto::barrel_service_server::BarrelServiceServer,
    settings::{GoogolConfig, Load, barrel::BarrelConfig},
};
use log::{debug, error, info};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let settings = match GoogolConfig::default() {
        Err(e) => {
            error!("{:#?}", e);

            BarrelConfig::default()?
        }

        Ok(config) => config.barrel,
    };
    debugv!(settings, debug);

    let settings = BarrelConfig {
        address: "127.0.0.1:50052".parse().unwrap(),
        filepath: ".barrel-data.json".to_string(),
    };

    let barrel = Barrel::new(&settings).await;
    debugv!(barrel, debug);

    info!("Barrel listening at {}...", barrel.address);

    Server::builder()
        .add_service(BarrelServiceServer::new(barrel))
        .serve(settings.address)
        .await?;

    Ok(())
}
