use googol::{
    proto::{
        self, DequeueRequest, Index, IndexRequest, gateway_service_client::GatewayServiceClient,
    },
    settings::{GoogolConfig, Load},
};
use log::{debug, error, info, warn};
use scraper::{Html, Selector};
use std::{collections::HashSet, time::Duration};
use tokio::{task::JoinSet, time::sleep};
use tonic::Request;
use url::Url;

const MIN_BACKOFF: Duration = Duration::from_secs(1);
const MAX_BACKOFF: Duration = Duration::from_secs(60);

#[derive(Debug, Clone)]
struct HtmlInfo {
    url: Url,
    words: HashSet<String>,
    outlinks: HashSet<Url>,
    title: Option<String>,
    icon: Option<String>,
}

impl HtmlInfo {
    pub async fn new(url_str: &str, stop_words: &HashSet<String>) -> Result<Self, HtmlError> {
        // Parse the URL
        let url = Url::parse(url_str).map_err(|_| HtmlError::InvalidUrl)?;

        // Fetch the webpage asynchronously
        let response = reqwest::get(url.as_str()).await?;
        let body = response.text().await?;

        // Parse HTML
        let document = Html::parse_document(&body);

        // Extract title
        let title_selector = Selector::parse("title").unwrap();
        let title = document
            .select(&title_selector)
            .next()
            .map(|t| t.inner_html());

        // Extract all words
        let body_selector = Selector::parse("body").unwrap();
        let words: HashSet<String> = match document.select(&body_selector).next() {
            Some(body) => body
                .text()
                .collect::<Vec<_>>()
                .join(" ")
                .split_whitespace()
                .map(|w| w.to_lowercase())
                .filter(|w| !w.is_empty())
                .filter(|w| !stop_words.contains(w.as_str()))
                .filter(|w| w.chars().all(|c| c.is_alphanumeric()))
                .collect(),
            None => HashSet::new(),
        };

        // Extract all outlinks
        let link_selector = Selector::parse("a").unwrap();
        let outlinks: HashSet<Url> = document
            .select(&link_selector)
            .filter_map(|element| element.value().attr("href"))
            .filter_map(|href| match url.join(href) {
                Ok(outlink) => Some(outlink),
                Err(e) => {
                    error!("Error invalid join url: {}/{}: {}", url, href, e);

                    None
                }
            })
            .collect();

        // Extract favicon URL
        let favicon_selector =
            Selector::parse(r#"link[rel="icon"], link[rel="shortcut icon"]"#).unwrap();
        let favicon_url = document
            .select(&favicon_selector)
            .next()
            .and_then(|link| link.value().attr("href"))
            .and_then(|href| url.join(href).ok());
        debug!("favicon_url = {:#?}", favicon_url);

        let icon = None;
        //let icon = match favicon_url {
        //    // Fetch favicon bytes
        //    Some(favicon_url) => match reqwest::get(favicon_url.as_str()).await {
        //        Ok(resp) => match resp.bytes().await {
        //            Ok(bytes) => Some(general_purpose::STANDARD.encode(bytes)),
        //            Err(_) => None,
        //        },
        //        Err(_) => None,
        //    },
        //    None => None,
        //};

        Ok(Self {
            url,
            words,
            outlinks,
            title,
            icon,
        })
    }
}

impl Into<proto::Page> for HtmlInfo {
    fn into(self) -> proto::Page {
        proto::Page {
            url: self.url.to_string(),
            title: self.title.unwrap_or(String::from("")),
            summary: String::from(""),
            icon: self.icon.unwrap_or(String::from("")),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
enum HtmlError {
    InvalidUrl,
    ReqwestError(reqwest::Error),
    UrlParseError(url::ParseError),
    MissingTitle,
}

impl From<reqwest::Error> for HtmlError {
    fn from(err: reqwest::Error) -> Self {
        HtmlError::ReqwestError(err)
    }
}

impl From<url::ParseError> for HtmlError {
    fn from(err: url::ParseError) -> Self {
        HtmlError::UrlParseError(err)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let settings = GoogolConfig::default()?.downloader;
    info!("{:?}", settings);

    let gateway_address = format!("http://{}", settings.gateway);

    info!("Connecting to gateway: {}...", &gateway_address);

    let mut join_set = JoinSet::new();

    for task_id in 1..=settings.threads {
        let address = gateway_address.clone();
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

                                match HtmlInfo::new(&response.url, &stop_words).await {
                                    Ok(html_info) => {
                                        debug!("html_info = {:#?}", html_info);

                                        let page = Some(html_info.clone().into());

                                        let words: Vec<String> = html_info.words.iter().cloned().collect();
                                        let outlinks: Vec<String> = html_info.outlinks.iter().cloned().map(|outlink| outlink.to_string()).collect();

                                        let index = Some(Index { page, words, outlinks });
                                        debug!("index = {:#?}", index);

                                        client
                                            .index(Request::new(IndexRequest { index }))
                                            .await
                                            .unwrap();

                                        true
                                    },
                                    Err(_) => todo!(),
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
