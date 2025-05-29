use std::{collections::HashSet, time::Duration};

use googol::{
    proto::{DequeueRequest, Index, IndexRequest, gateway_service_client::GatewayServiceClient},
    settings::{GoogolConfig, Load},
};
use log::{error, info, warn};
use scraper::{Html, Selector};
use tokio::{task::JoinSet, time::sleep};
use tonic::Request;

const MIN_BACKOFF: Duration = Duration::from_secs(1);
const MAX_BACKOFF: Duration = Duration::from_secs(60);

async fn html_get(
    task_id: usize,
    url: &String,
    stop_words: &HashSet<String>,
) -> Option<(HashSet<String>, HashSet<String>)> {
    match reqwest::get(url).await {
        Ok(http_response) => match http_response.text().await {
            Ok(body) => {
                let (links, keywords) = parse_html(&body, &stop_words);

                let links = links
                    .iter()
                    .map(|link| format!("{}/{}", url, link))
                    .collect();

                info!("[task-{}] links    = {:?}", task_id, links);
                info!("[task-{}] keywords = {:?}", task_id, keywords);

                Some((links, keywords))
            }
            Err(e) => {
                error!("[task-{}] Error parsing text: {}", task_id, e);
                None
            }
        },
        Err(e) => {
            error!(
                "[task-{}] Error downloading html of {}: {}",
                task_id, url, e
            );
            None
        }
    }
}

fn parse_html(body: &str, stop_words: &HashSet<String>) -> (HashSet<String>, HashSet<String>) {
    let document = Html::parse_document(body);

    let link_selector = Selector::parse("a").unwrap();
    let links: HashSet<String> = document
        .select(&link_selector)
        .filter_map(|element| element.value().attr("href"))
        .map(|href| href.to_string())
        .collect();

    let p_selector = Selector::parse("p").unwrap();
    let keywords: HashSet<String> = document
        .select(&p_selector)
        .flat_map(|paragraph| {
            paragraph
                .text()
                .map(|text| text.split_whitespace())
                .flatten()
        })
        .map(|word| {
            word.to_lowercase()
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_string()
        })
        .filter(|word| !word.is_empty() && !stop_words.contains(word.as_str()))
        .collect();

    (links, keywords)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let settings = GoogolConfig::default()?.downloader;
    info!("{:?}", settings);

    let barrel_orchestrator_address = format!("http://{}", settings.gateway);

    info!(
        "Connecting to barrel orchestrator: {}...",
        &barrel_orchestrator_address
    );

    let mut join_set = JoinSet::new();

    for task_id in 1..=settings.threads {
        let address = barrel_orchestrator_address.clone();
        let stop_words = settings.stop_words.clone();

        join_set.spawn(async move {
            let mut interval = MIN_BACKOFF;
            loop {
                let success = match GatewayServiceClient::connect(address.clone()).await {
                    Err(e) => {
                        error!("[task-{}] Error connecting to {}: {}", task_id, address, e);
                        false
                    }
                    Ok(mut client) => {
                        let request = Request::new(DequeueRequest {});

                        match client.dequeue_url(request).await {
                            Err(e) => {
                                error!("[task-{}] Failing dequeuing url: {}", task_id, e);
                                false
                            }
                            Ok(response) => {
                                info!("[task-{}] RESPONSE = {:#?}", task_id, response);

                                let response = response.into_inner();

                                match html_get(task_id, &response.url, &stop_words).await {
                                    None => todo!(),
                                    Some((outlinks, words)) => {
                                        let url = response.url;
                                        let words = words.into_iter().collect();
                                        let outlinks = outlinks.into_iter().collect();

                                        let index = Some(Index {
                                            url,
                                            words,
                                            outlinks,
                                        });

                                        client
                                            .index(Request::new(IndexRequest { index }))
                                            .await
                                            .unwrap();
                                        true
                                    }
                                }
                            }
                        }
                    }
                };

                if success {
                    interval = MIN_BACKOFF;
                } else {
                    interval = (interval * 2).min(MAX_BACKOFF).max(MIN_BACKOFF);
                    warn!(
                        "[task{}] Failing connecting to gateway {}. Trying connecting in {} seconds...",
                        task_id,
                        address,
                        interval.as_secs()
                    );
                    sleep(interval).await;
                }
            }
        });
    }

    join_set.join_all().await;

    Ok(())
}
