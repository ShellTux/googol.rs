use crate::{
    GoogolStatus,
    proto::{
        BacklinksRequest, BacklinksResponse, BarrelStatusRequest, BarrelStatusResponse,
        HealthRequest, HealthResponse, IndexRequest, IndexResponse, OutlinksRequest,
        OutlinksResponse, SearchRequest, SearchResponse, barrel_service_server::BarrelService,
    },
    settings::barrel::BarrelConfig,
};

use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::Write,
    net::SocketAddr,
    str::FromStr,
};
use tokio::sync::Mutex as AsyncMutex;
use tonic::{Request, Response, Status};
use url::Url;

trait IndexStore {
    fn into_vec(&self) -> Vec<String>;
    fn store(&mut self, url: &Url, words: &Vec<String>);
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct IndexPages(HashMap<String, HashSet<Url>>);

impl IndexPages {
    fn search(&self, words: &Vec<String>) -> Vec<String> {
        let mut result_set = HashSet::new();

        for word in words {
            let word = word.to_lowercase();

            if let Some(urls) = self.0.get(&word) {
                for url in urls {
                    result_set.insert(url.clone());
                }
            }
        }

        result_set
            .into_iter()
            .map(|url| url.as_str().to_string())
            .collect()
    }

    fn search_order_by_relevance(
        &self,
        words: &Vec<String>,
        backlinks_map: &HashMap<Url, HashSet<Url>>,
    ) -> Vec<String> {
        let mut url_counts: HashMap<Url, usize> = HashMap::new();

        for word in words {
            let word = word.to_lowercase();

            if let Some(urls) = self.0.get(&word) {
                for url in urls {
                    *url_counts.entry(url.clone()).or_insert(0) =
                        backlinks_map.get(&url).iter().len();
                }
            }
        }

        let mut url_count_vec: Vec<(Url, usize)> = url_counts.into_iter().collect();

        url_count_vec.sort_by(|a, b| b.1.cmp(&a.1));

        url_count_vec
            .into_iter()
            .map(|(url, _)| url.as_str().to_string())
            .collect()
    }
}

impl IndexStore for IndexPages {
    fn into_vec(&self) -> Vec<String> {
        self.0.keys().cloned().collect()
    }

    fn store(&mut self, url: &Url, words: &Vec<String>) {
        for word in words {
            let word = word.to_lowercase();

            self.0
                .entry(word)
                .or_insert_with(HashSet::new)
                .insert(url.clone());
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct InvertedIndexPages(HashMap<Url, HashSet<String>>);

impl IndexStore for InvertedIndexPages {
    fn into_vec(&self) -> Vec<String> {
        self.0.keys().map(|url| url.as_str().to_string()).collect()
    }

    fn store(&mut self, url: &Url, words: &Vec<String>) {
        self.0
            .entry(url.clone())
            .or_insert_with(HashSet::new)
            .extend(words.iter().map(|word| word.to_lowercase()));
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Index {
    pub indexed_pages: IndexPages,
    pub inverted_indexed_pages: InvertedIndexPages,
    pub backlinks: HashMap<Url, HashSet<Url>>,
    pub outlinks: HashMap<Url, HashSet<Url>>,
    #[serde(skip)]
    pub filepath: String,
}

impl Index {
    fn indexed_pages(&self) -> Vec<String> {
        self.inverted_indexed_pages.into_vec()
    }

    fn indexed_words(&self) -> Vec<String> {
        self.indexed_pages.into_vec()
    }

    fn consult_backlinks(&self, url: &Url) -> Vec<String> {
        match self.backlinks.get(url) {
            Some(backlink_set) => backlink_set.iter().map(|url| url.to_string()).collect(),
            None => Vec::new(),
        }
    }

    fn consult_outlinks(&self, url: &Url) -> Vec<String> {
        match self.outlinks.get(url) {
            Some(outlink_set) => outlink_set.iter().map(|url| url.to_string()).collect(),
            None => Vec::new(),
        }
    }

    fn store(&mut self, url: &Url, words: &Vec<String>, outlinks: &Vec<Url>) {
        self.indexed_pages.store(url, words);
        self.inverted_indexed_pages.store(url, words);

        self.outlinks
            .entry(url.clone())
            .or_insert_with(HashSet::new)
            .extend(outlinks.iter().cloned());

        for outlink in outlinks {
            self.backlinks
                .entry(outlink.clone())
                .or_insert_with(HashSet::new)
                .insert(url.clone());
        }

        if let Ok(mut file) = File::create(&self.filepath) {
            match serde_json::to_string(&self) {
                Ok(json_str) => match file.write(json_str.as_bytes()) {
                    Ok(_) => info!("Written index to file succesfully: {}", self.filepath),
                    Err(e) => error!("Failed to write index to file: {}", e),
                },
                Err(e) => {
                    error!("Failed to serialize index: {}", e);
                }
            }
        } else {
            error!("Error creating file: {}", self.filepath);
        }
    }

    pub fn load(filepath: &str) -> Self {
        let mut index = match fs::read_to_string(filepath) {
            Ok(contents) => match serde_json::from_str(&contents) {
                Ok(index) => {
                    info!("Success to load index file: {}", filepath);
                    index
                }
                Err(e) => {
                    error!("Failed to parse index file: {}", e);
                    Index::default()
                }
            },
            Err(e) => {
                error!("Failed to read index file: {}", e);
                Index::default()
            }
        };

        index.filepath = filepath.to_string();
        index
    }
}

#[derive(Debug)]
pub struct Barrel {
    pub address: SocketAddr,
    index: AsyncMutex<Index>,
}

impl Default for Barrel {
    fn default() -> Self {
        Self {
            address: SocketAddr::from_str("[::1]:50053").unwrap(),
            index: AsyncMutex::new(Index::default()),
        }
    }
}

impl Barrel {
    pub async fn from(config: &BarrelConfig) -> Self {
        let mut barrel = Barrel::default();
        barrel.address = config.address;

        *barrel.index.lock().await = Index::load(config.filepath.as_str());

        barrel
    }
}

#[tonic::async_trait]
impl BarrelService for Barrel {
    async fn consult_backlinks(
        &self,
        request: Request<BacklinksRequest>,
    ) -> Result<Response<BacklinksResponse>, Status> {
        debug!("{:#?}", request);

        let request = request.into_inner();

        let url = Url::parse(&request.url).map_err(|e| {
            error!("Invalid URL provided: {}", e);
            Status::invalid_argument(format!("Invalid URL: {}", request.url))
        })?;

        let backlinks = self.index.lock().await.consult_backlinks(&url);
        let status = GoogolStatus::Success as i32;

        Ok(Response::new(BacklinksResponse { status, backlinks }))
    }

    async fn consult_outlinks(
        &self,
        request: Request<OutlinksRequest>,
    ) -> Result<Response<OutlinksResponse>, Status> {
        debug!("{:#?}", request);

        let request = request.into_inner();

        let url = Url::parse(&request.url).map_err(|e| {
            error!("Invalid URL provided: {}", e);
            Status::invalid_argument(format!("Invalid URL: {}", request.url))
        })?;

        let outlinks = self.index.lock().await.consult_outlinks(&url);
        debug!("outlinks = {:?}", outlinks);
        let status = GoogolStatus::Success as i32;

        Ok(Response::new(OutlinksResponse { status, outlinks }))
    }

    async fn health(
        &self,
        request: Request<HealthRequest>,
    ) -> Result<Response<HealthResponse>, Status> {
        debug!("{:#?}", request);

        Ok(Response::new(HealthResponse {
            status: format!("OK: Online. Listening at {}...", self.address),
        }))
    }

    async fn index(
        &self,
        request: Request<IndexRequest>,
    ) -> Result<Response<IndexResponse>, Status> {
        debug!("{:#?}", request);

        let request = request.into_inner();

        let index = request.index.unwrap();

        let url = Url::parse(&index.url)
            .map_err(|e| error!("Invalid url `{}`: {}", index.url, e))
            .unwrap();

        let words = index.words;

        let outlinks = index
            .outlinks
            .iter()
            .filter_map(|url| match Url::parse(url) {
                Ok(url) => Some(url),
                Err(e) => {
                    error!("Invalid url `{}`: {}", url, e);
                    None
                }
            })
            .collect();

        self.index.lock().await.store(&url, &words, &outlinks);

        Ok(Response::new(IndexResponse {}))
    }

    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        debug!("{:#?}", request);

        let request = request.into_inner();

        let index = self.index.lock().await;

        let words = request.words;

        let pages = index
            .indexed_pages
            .search_order_by_relevance(&words, &index.backlinks);

        Ok(Response::new(SearchResponse {
            status: GoogolStatus::Success as i32,
            urls: pages,
        }))
    }

    async fn status(
        &self,
        request: Request<BarrelStatusRequest>,
    ) -> Result<Response<BarrelStatusResponse>, Status> {
        debug!("{:#?}", request);

        let status = String::default();

        Ok(Response::new(BarrelStatusResponse { status }))
    }
}
