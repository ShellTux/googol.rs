use googol::{
    gateway::Gateway,
    proto::gateway_service_server::GatewayServiceServer,
    settings::{GoogolConfig, Load},
};
use log::{debug, info};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = GoogolConfig::default()?.gateway;

    pretty_env_logger::init();

    let gateway = Gateway::from(&settings).await;
    debug!("{:#?}", gateway);

    info!("Gateway listening at {}...", gateway.address);
    Server::builder()
        .add_service(GatewayServiceServer::new(gateway))
        .serve(settings.address)
        .await?;

    Ok(())
}
