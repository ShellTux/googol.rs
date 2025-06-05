use clap::{Arg, Command};
use googol::{
    debugv,
    proto::{
        BacklinksRequest, EnqueueRequest, HealthRequest, OutlinksRequest, RealTimeStatusRequest,
        SearchRequest, gateway_service_client::GatewayServiceClient,
    },
    settings::{GoogolConfig, Load, client::ClientConfig},
};
use log::{debug, error};
use std::{net::SocketAddr, str::FromStr, time::Duration};
use tokio::time::sleep;
use tonic::{Request, Status, transport::Channel};

async fn connect_with_backoff<ClientType, F, Fut>(
    max_retries: usize,
    address: SocketAddr,
    f: F,
) -> Result<ClientType, String>
where
    F: Fn(usize, GatewayServiceClient<Channel>) -> Fut,
    Fut: Future<Output = Result<ClientType, Status>> + Send,
{
    let mut attempt = 0;
    let mut delay = Duration::from_millis(1000);

    loop {
        let address = format!("http://{}", address);

        if let Ok(client) = GatewayServiceClient::connect(address).await {
            if let Ok(result) = f(attempt, client).await {
                break Ok(result);
            }
        }

        attempt += 1;

        if attempt >= max_retries {
            break Err(String::from("Failed connecting"));
        }

        eprintln!(
            "Connection attempt {}/{} failed, retrying in {:?}...",
            attempt, max_retries, delay
        );

        sleep(delay).await;

        delay = delay * 2;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let settings = match GoogolConfig::default() {
        Err(e) => {
            error!("{:#?}", e);

            ClientConfig::default()?
        }

        Ok(config) => config.client,
    };
    debugv!(settings);

    let default_address = Box::leak(Box::new(format!("{}", settings.gateway)));
    let default_max_retries = Box::leak(Box::new(settings.max_retries.to_string()));

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
        .arg(
            Arg::new("retries")
                .short('r')
                .long("retries")
                .value_name("RETRIES")
                .help("Number of retries for exponential backoff when connecting via gRPC, e.g.: 5")
                .default_value(default_max_retries.as_str()),
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

    let address = matches.get_one::<String>("address").unwrap();
    let address = SocketAddr::from_str(address).unwrap();

    let retries = matches
        .get_one::<String>("retries")
        .unwrap()
        .parse()
        .unwrap();

    match matches.subcommand() {
        Some(("health", _)) => {
            connect_with_backoff(retries, address, async move |_, mut client| {
                let request = Request::new(HealthRequest {});
                let response = client.health(request).await?;

                println!("Health: {:?}", response.into_inner());

                Ok(())
            })
            .await?;
        }
        Some(("real-time-status", _)) => {
            connect_with_backoff(retries, address, async move |_, mut client| {
                loop {
                    let request = Request::new(RealTimeStatusRequest {});
                    let response = client.real_time_status(request).await?;
                    println!("Status: {:#?}", response.into_inner());
                }
            })
            .await?;
        }
        Some(("enqueue", enqueue)) => {
            connect_with_backoff(retries, address, async move |_, mut client| {
                let url = enqueue.get_one::<String>("url").unwrap().clone();

                let request = Request::new(EnqueueRequest { url });

                let response = client.enqueue_url(request).await?;
                println!("Response: {:#?}", response.into_inner());

                Ok(())
            })
            .await?;
        }
        Some(("search", search)) => {
            connect_with_backoff(retries, address, async move |_, mut client| {
                let words: Vec<String> = search
                    .get_one::<String>("words")
                    .unwrap()
                    .split_whitespace()
                    .filter(|word| !word.is_empty())
                    .map(|string| string.to_string())
                    .collect();

                let request = Request::new(SearchRequest { words });

                let response = client.search(request).await?;

                println!("Response: {:#?}", response.into_inner());

                Ok(())
            })
            .await?;
        }
        Some(("consult", sub_m)) => match sub_m.subcommand() {
            Some(("backlinks", backlinks)) => {
                connect_with_backoff(retries, address, async move |_, mut client| {
                    let url = backlinks.get_one::<String>("url").unwrap().clone();

                    let request = Request::new(BacklinksRequest { url: url.clone() });

                    let response = client.consult_backlinks(request).await?.into_inner();

                    println!("Backlinks of {}: {:#?}", url, response);

                    Ok(())
                })
                .await?;
            }
            Some(("outlinks", outlinks)) => {
                connect_with_backoff(retries, address, async move |_, mut client| {
                    let url = outlinks.get_one::<String>("url").unwrap().clone();

                    let request = Request::new(OutlinksRequest { url: url.clone() });

                    let response = client.consult_outlinks(request).await?.into_inner();

                    println!("Outlinks of {}: {:#?}", url, response);

                    Ok(())
                })
                .await?;
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
