use googol::{
    debugv,
    gateway::Gateway,
    proto::gateway_service_server::GatewayServiceServer,
    settings::{GoogolConfig, Load, gateway::GatewayConfig},
};
use log::{debug, error, info};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let settings = match GoogolConfig::default() {
        Err(e) => {
            error!("{:#?}", e);

            GatewayConfig::default()?
        }

        Ok(config) => config.gateway,
    };
    debugv!(settings, debug);

    let gateway = Gateway::from(&settings).await;
    debugv!(gateway, debug);

    info!("Gateway listening at {}...", gateway.address);
    Server::builder()
        .add_service(GatewayServiceServer::new(gateway))
        .serve(settings.address)
        .await?;

    Ok(())
}
