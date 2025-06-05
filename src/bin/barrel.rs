use googol::{
    barrel::Barrel,
    proto::barrel_service_server::BarrelServiceServer,
    settings::{GoogolConfig, Load},
};
use log::{debug, info};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let settings = GoogolConfig::default()?.barrel;
    debug!("settings = {:#?}", settings);

    let barrel = Barrel::from(&settings).await;
    debug!("{:#?}", barrel);

    info!("Barrel listening at {}...", barrel.address);

    Server::builder()
        .add_service(BarrelServiceServer::new(barrel))
        .serve(settings.address)
        .await?;

    Ok(())
}
