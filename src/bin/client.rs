use clap::{Parser, Subcommand};
use googol::{
    debugv,
    proto::{
        BacklinksRequest, EnqueueRequest, HealthRequest, OutlinksRequest, RealTimeStatusRequest,
        SearchRequest, gateway_service_client::GatewayServiceClient,
    },
    settings::{GoogolConfig, Load, client::ClientConfig},
};
use log::{debug, error};
use std::{net::SocketAddr, time::Duration};
use tokio::time::sleep;
use tonic::{Request, Status, transport::Channel};
use url::Url;

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Client to interact with GatewayService Server",
    version = "1.0",
    author = "Luís Góis"
)]
struct Cli {
    /// Gateway server address (e.g., 127.0.0.1:8080)
    #[arg(short, long, help = "Gateway server address")]
    address: Option<SocketAddr>,

    /// Number of retries for exponential backoff when connecting via gRPC
    #[arg(
        short,
        long,
        help = "Number of retries for exponential backoff when connecting via gRPC"
    )]
    retries: Option<usize>,

    /// Subcommands for specific operations
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Enqueue a URL for processing
    Enqueue {
        /// URL to enqueue
        url: Url,
    },

    /// Search for provided words
    Search {
        /// Words to search for
        #[arg(required = true)]
        words: Vec<String>,
    },

    /// Consult backlinks or outlinks of a given page
    Consult {
        #[command(subcommand)]
        consult_command: ConsultCommand,
    },

    /// Get real-time status of the system
    RealTimeStatus,

    /// Perform a health check
    Health,
}

#[derive(Debug, Subcommand)]
enum ConsultCommand {
    /// Get backlinks pointing to the specified URL
    Backlinks {
        /// The URL to check for backlinks
        url: Url,
    },

    /// Get outlinks from the specified URL
    Outlinks {
        /// The URL to check for outlinks
        url: Url,
    },
}

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

        delay *= 2;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let cli = Cli::parse();
    debugv!(&cli);

    let settings = match GoogolConfig::default() {
        Err(e) => {
            error!("{:#?}", e);

            ClientConfig::default()?
        }

        Ok(config) => config.client,
    };
    debugv!(settings);

    let retries = cli.retries.unwrap_or(settings.max_retries);
    let address = cli.address.unwrap_or(settings.gateway);

    match &cli.command {
        Commands::Enqueue { url } => {
            connect_with_backoff(retries, address, async move |_, mut client| {
                let url = url.to_string();

                let request = Request::new(EnqueueRequest { url });

                let response = client.enqueue_url(request).await?;
                println!("Response: {:#?}", response.into_inner());

                Ok(())
            })
            .await?;
        }
        Commands::Search { words } => {
            connect_with_backoff(retries, address, async move |_, mut client| {
                let words = words.iter().filter(|w| !w.is_empty()).cloned().collect();

                let request = Request::new(SearchRequest { words });

                let response = client.search(request).await?;

                println!("Response: {:#?}", response.into_inner());

                Ok(())
            })
            .await?;
        }
        Commands::Consult { consult_command } => match consult_command {
            ConsultCommand::Backlinks { url } => {
                connect_with_backoff(retries, address, async move |_, mut client| {
                    let url = url.clone();

                    let request = Request::new(BacklinksRequest {
                        url: url.to_string(),
                    });

                    let response = client.consult_backlinks(request).await?.into_inner();

                    println!("Backlinks of {}: {:#?}", url, response);

                    Ok(())
                })
                .await?;
            }
            ConsultCommand::Outlinks { url } => {
                connect_with_backoff(retries, address, async move |_, mut client| {
                    let url = url.clone();

                    let request = Request::new(OutlinksRequest {
                        url: url.to_string(),
                    });

                    let response = client.consult_outlinks(request).await?.into_inner();

                    println!("Outlinks of {}: {:#?}", url, response);

                    Ok(())
                })
                .await?;
            }
        },
        Commands::RealTimeStatus => {
            connect_with_backoff(retries, address, async move |_, mut client| {
                loop {
                    let request = Request::new(RealTimeStatusRequest {});
                    let response = client.real_time_status(request).await?;
                    println!("Status: {:#?}", response.into_inner());
                }
            })
            .await?;
        }
        Commands::Health => {
            connect_with_backoff(retries, address, async move |_, mut client| {
                let request = Request::new(HealthRequest {});
                let response = client.health(request).await?;

                println!("Health: {:?}", response.into_inner());

                Ok(())
            })
            .await?;
        }
    };

    Ok(())
}
