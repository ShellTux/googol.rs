use crate::{
    GoogolStatus,
    address::Address,
    index_store::IndexStore,
    page::Page,
    proto::{
        BacklinksRequest, BacklinksResponse, BarrelStatusRequest, BarrelStatusResponse,
        HealthRequest, HealthResponse, IndexRequest, IndexResponse, OutlinksRequest,
        OutlinksResponse, SearchRequest, SearchResponse, barrel_service_server::BarrelService,
    },
    settings::barrel::BarrelConfig,
};
use log::{debug, error};
use tokio::sync::Mutex as AsyncMutex;
use tonic::{Request, Response, Status};
use url::Url;

#[derive(Debug, Default)]
pub struct Barrel {
    pub address: Address,
    index: AsyncMutex<IndexStore>,
}

impl Barrel {
    pub async fn from(config: &BarrelConfig) -> Self {
        let mut barrel = Barrel::default();
        barrel.address = Address::new(config.address);

        *barrel.index.lock().await = IndexStore::load(&config.filepath).unwrap();

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

        let backlinks = self
            .index
            .lock()
            .await
            .consult_backlinks(&url)
            .iter()
            .map(|url| url.to_string())
            .collect();

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

        let outlinks = self
            .index
            .lock()
            .await
            .consult_outlinks(&url)
            .iter()
            .map(|url| url.to_string())
            .collect();
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

        let page = Page::from(index.page.unwrap());

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

        let mut index = self.index.lock().await;

        index.store(&page, &words, &outlinks);
        index.save().unwrap();

        Ok(Response::new(IndexResponse { size_bytes: 0 }))
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
            .search_by_relevance(&words)
            .iter()
            .cloned()
            .map(|page| page.into())
            .collect();

        Ok(Response::new(SearchResponse {
            status: GoogolStatus::Success as i32,
            pages,
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
