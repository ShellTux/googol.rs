use std::time::Duration;

use hello_world::HelloRequest;
use hello_world::greeter_client::GreeterClient;
use tonic::Request;
use tonic::transport::Channel;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

async fn connect_with_backoff(
    address: &str,
    max_attempts: usize,
) -> Result<GreeterClient<Channel>, Box<dyn std::error::Error>> {
    for attempt in 1..=max_attempts {
        match tonic::transport::Channel::from_shared(address.to_string())?
            .connect()
            .await
        {
            Ok(channel) => {
                println!("Connected successfully on attempt {}", attempt);
                return Ok(GreeterClient::new(channel));
            }
            Err(e) => {
                let wait_time = Duration::from_secs(2u64.pow(attempt as u32));
                eprintln!(
                    "Attempt {} failed: {:?}. Retrying in {:?} seconds...",
                    attempt,
                    e,
                    wait_time.as_secs()
                );
                tokio::time::sleep(wait_time).await;
            }
        }
    }
    Err("Failed to connect after maximum attempts".into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let address = "http://[::1]:50069";
    let mut client = match connect_with_backoff(address, 5).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return Ok(());
        }
    };

    let request = Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let response = client.say_hello(request).await?;

    println!("RESPONSE = {:?}", response);

    Ok(())
}
