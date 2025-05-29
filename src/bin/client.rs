use clap::{Arg, Command};
use googol::{
    proto::{
        BacklinksRequest, EnqueueRequest, HealthRequest, OutlinksRequest, RealTimeStatusRequest,
        SearchRequest, gateway_service_client::GatewayServiceClient,
    },
    settings::{GoogolConfig, Load},
};
use log::{debug, error};
use tonic::Request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let settings = GoogolConfig::default()?.client;

    let default_address = Box::leak(Box::new(format!("{}", settings.gateway)));

    let cli = Command::new("Gateway Client")
        .version("1.0")
        .author("Luís Góis")
        .about("CLI to interact with GatewayService")
        .arg(
            Arg::new("address")
                .short('a')
                .long("address")
                .value_name("ADDRESS")
                .help("Server address, e.g., 127.0.0.1:50051")
                .default_value(default_address.as_str()),
        )
        .subcommand(
            Command::new("enqueue").about("Enqueue a URL").arg(
                Arg::new("url")
                    .short('u')
                    .long("url")
                    .value_name("URL")
                    .help("URL to enqueue")
                    .required(true),
            ),
        )
        .subcommand(
            Command::new("search").about("Search pages by words").arg(
                Arg::new("words")
                    .short('w')
                    .long("words")
                    .value_name("WORDS")
                    .help("Words to search for")
                    .required(true),
            ),
        )
        .subcommand(
            Command::new("consult")
                .about("Consult backlinks or outlinks of a given page")
                .subcommand(
                    Command::new("backlinks")
                        .about("Consult backlinks of a given page")
                        .arg(
                            Arg::new("url")
                                .short('u')
                                .long("url")
                                .value_name("url")
                                .help("url to consult backlinks of")
                                .required(true),
                        ),
                )
                .subcommand(
                    Command::new("outlinks")
                        .about("Consult outlinks of a given page")
                        .arg(
                            Arg::new("url")
                                .short('u')
                                .long("url")
                                .value_name("url")
                                .help("url to consult backlinks of")
                                .required(true),
                        ),
                ),
        )
        .subcommand(Command::new("real-time-status").about("Real Time Status of the system"))
        .subcommand(Command::new("health").about("Perform health check"));

    let matches = cli.clone().get_matches();

    let address = format!("http://{}", matches.get_one::<String>("address").unwrap());
    debug!("address = {:#?}", address);
    dbg!(&address);

    match matches.subcommand() {
        Some(("health", _)) => {
            let mut client = GatewayServiceClient::connect(address).await?;
            let request = Request::new(HealthRequest {});
            let response = client.health(request).await?;

            println!("Health: {:?}", response.into_inner());
        }
        Some(("real-time-status", _)) => {
            let mut client = GatewayServiceClient::connect(address).await?;

            loop {
                let request = Request::new(RealTimeStatusRequest {});
                let response = client.real_time_status(request).await?;
                println!("Status: {:#?}", response.into_inner());
            }
        }
        Some(("enqueue", enqueue)) => {
            let mut client = GatewayServiceClient::connect(address).await?;

            let url = enqueue.get_one::<String>("url").unwrap().clone();

            let request = Request::new(EnqueueRequest { url });

            let response = client.enqueue_url(request).await?;
            println!("Response: {:#?}", response.into_inner());
        }
        Some(("search", search)) => {
            let mut client = GatewayServiceClient::connect(address).await?;

            let words: Vec<String> = search
                .get_one::<String>("words")
                .unwrap()
                .split_whitespace()
                .map(|string| string.to_string())
                .collect();

            let request = tonic::Request::new(SearchRequest { words });

            let response = client.search(request).await?;

            println!("Response: {:#?}", response.into_inner());
        }
        Some(("consult", sub_m)) => match sub_m.subcommand() {
            Some(("backlinks", backlinks)) => {
                let mut client = GatewayServiceClient::connect(address).await?;

                let url = backlinks.get_one::<String>("url").unwrap().clone();

                let request = Request::new(BacklinksRequest { url: url.clone() });

                let response = client.consult_backlinks(request).await?.into_inner();

                println!("Backlinks of {}: {:#?}", url, response);
            }
            Some(("outlinks", outlinks)) => {
                let mut client = GatewayServiceClient::connect(address).await?;

                let url = outlinks.get_one::<String>("url").unwrap().clone();

                let request = Request::new(OutlinksRequest { url: url.clone() });

                let response = client.consult_outlinks(request).await?.into_inner();

                println!("Outlinks of {}: {:#?}", url, response);
            }
            _ => {
                error!("Invalid consult subcommand");
                cli.clone().print_help().unwrap();
            }
        },
        _ => {
            error!("Invalid subcommand");
            cli.clone().print_help().unwrap();
        }
    }

    Ok(())
}
