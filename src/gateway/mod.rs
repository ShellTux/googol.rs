use crate::{
    GoogolStatus,
    address::Address,
    gateway::load_balancer::LoadBalancer,
    proto::{
        BacklinksRequest, BacklinksResponse, BroadcastIndexRequest, BroadcastIndexResponse,
        DequeueRequest, DequeueResponse, EnqueueRequest, EnqueueResponse, GatewayStatusRequest,
        GatewayStatusResponse, HealthRequest, HealthResponse, IndexRequest, IndexResponse,
        OutlinksRequest, OutlinksResponse, RealTimeStatusRequest, RealTimeStatusResponse,
        RequestIndexRequest, RequestIndexResponse, SearchRequest, SearchResponse,
        gateway_service_server::GatewayService,
    },
    settings::gateway::GatewayConfig,
};
use load_balancer::LBResult;
use log::{debug, error};
use queue::Queue;
use status::GatewayStatus;
use tokio::sync::{Mutex as AsyncMutex, Notify};
use tonic::{Request, Response, Status};
use url::Url;

pub mod load_balancer;
pub mod queue;
pub mod status;

#[derive(Debug, Default)]
/// Notification
pub struct Notification {
    pub status: Notify,
    pub queue: Notify,
}

#[derive(Debug, Default)]
pub struct Gateway {
    pub address: Address,
    pub queue: AsyncMutex<Queue>,
    pub load_balancer: AsyncMutex<LoadBalancer>,
    pub status: AsyncMutex<GatewayStatus>,
    pub notification: Notification,
    // TODO: Add caching
}

impl Gateway {
    pub fn create() -> Self {
        Self::default()
    }

    pub fn with_address(mut self, address: Address) -> Self {
        self.address = address;
        self
    }

    pub async fn with_load_balancer(self, lb: LoadBalancer) -> Self {
        *self.load_balancer.lock().await = lb;
        self
    }

    pub async fn with_queue(self, queue: Queue) -> Self {
        *self.queue.lock().await = queue;
        self
    }

    pub async fn from(config: &GatewayConfig) -> Self {
        Self::create()
            .with_address(Address::new(config.address))
            .with_load_balancer(LoadBalancer::new(&config.barrels))
            .await
            .with_queue(Queue::create().with_domains_filter(&config.domains_filter))
            .await
    }
}

#[tonic::async_trait]
impl GatewayService for Gateway {
    async fn broadcast_index(
        &self,
        request: Request<BroadcastIndexRequest>,
    ) -> Result<Response<BroadcastIndexResponse>, Status> {
        debug!("{:#?}", request);

        unimplemented!()
    }

    async fn consult_backlinks(
        &self,
        request: Request<BacklinksRequest>,
    ) -> Result<Response<BacklinksResponse>, Status> {
        debug!("{:#?}", request);

        let request = request.into_inner();

        let (status, backlinks) = match self
            .load_balancer
            .lock()
            .await
            .send_until(|mut client| {
                let request = request.clone();
                Box::pin(async move { client.consult_backlinks(request).await })
            })
            .await
        {
            LBResult::Ok(response, _, _) => (response.status, response.backlinks),
            LBResult::Offline(_) => (GoogolStatus::UnavailableBarrels as i32, vec![]),
        };

        Ok(Response::new(BacklinksResponse { status, backlinks }))
    }

    async fn consult_outlinks(
        &self,
        request: Request<OutlinksRequest>,
    ) -> Result<Response<OutlinksResponse>, Status> {
        debug!("{:#?}", request);

        let request = request.into_inner();

        let (status, outlinks) = match self
            .load_balancer
            .lock()
            .await
            .send_until(|mut client| {
                let request = request.clone();
                Box::pin(async move { client.consult_outlinks(request).await })
            })
            .await
        {
            LBResult::Ok(response, _, _) => (response.status, response.outlinks),
            LBResult::Offline(_) => (GoogolStatus::UnavailableBarrels as i32, vec![]),
        };

        Ok(Response::new(OutlinksResponse { status, outlinks }))
    }

    async fn dequeue_url(
        &self,
        request: Request<DequeueRequest>,
    ) -> Result<Response<DequeueResponse>, Status> {
        debug!("{:#?}", request);

        let url = loop {
            if let Some(url) = self.queue.lock().await.dequeue() {
                break url;
            }

            self.notification.queue.notified().await;
        }
        .to_string();

        self.notification.status.notify_waiters();

        Ok(Response::new(DequeueResponse { url }))
    }

    async fn enqueue_url(
        &self,
        request: Request<EnqueueRequest>,
    ) -> Result<Response<EnqueueResponse>, Status> {
        debug!("{:#?}", request);

        let request = request.into_inner();

        let (status, queue) = match Url::parse(&request.url) {
            Err(e) => {
                error!("Invalid url: `{}`: {}", &request.url, e);
                (GoogolStatus::InvalidUrl, vec![])
            }
            Ok(url) => self.queue.lock().await.enqueue(url),
        };

        if status == GoogolStatus::Success {
            self.notification.status.notify_waiters();
        }

        let status = status as i32;

        Ok(Response::new(EnqueueResponse { status, queue }))
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

        if let Some(index) = &request.index {
            let mut queue = self.queue.lock().await;

            for url in index.outlinks.iter().map(|url| Url::parse(url).unwrap()) {
                queue.enqueue(url);
            }
        }

        let online = match self
            .load_balancer
            .lock()
            .await
            .broadcast(|_, mut client| {
                let request = request.clone();

                Box::pin(async move {
                    let response = client.index(request).await;

                    //if let Ok(response) = response {
                    //    let response = response.into_inner();
                    //
                    //    barrel.index_size_bytes = response.size_bytes as usize;
                    //}

                    response
                })
            })
            .await
        {
            LBResult::Ok(responses, _, _) => responses.len(),
            LBResult::Offline(_) => 0,
        };

        if online == 0 {
            // TODO: Caching to send index later to barrels
        }

        Ok(Response::new(IndexResponse { size_bytes: 0 }))
    }

    async fn real_time_status(
        &self,
        request: Request<RealTimeStatusRequest>,
    ) -> Result<Response<RealTimeStatusResponse>, Status> {
        debug!("{:#?}", request);

        self.notification.status.notified().await;

        let barrels = self.load_balancer.lock().await.get_barrels_status();

        let queue = self.queue.lock().await.into_vec();

        let status = self.status.lock().await;

        let avg_response_time_ms = status.response_time.miliseconds;

        let top10_searches = status
            .top_searches
            .top_n(10)
            .iter()
            .map(|(word, _)| word)
            .cloned()
            .collect();

        Ok(Response::new(RealTimeStatusResponse {
            top10_searches,
            barrels,
            avg_response_time_ms,
            queue,
        }))
    }

    async fn request_index(
        &self,
        request: Request<RequestIndexRequest>,
    ) -> Result<Response<RequestIndexResponse>, Status> {
        debug!("{:#?}", request);

        todo!()
    }

    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        debug!("{:#?}", request);

        let request = request.into_inner();

        let (status, pages) = match self
            .load_balancer
            .lock()
            .await
            .send_until(|mut client| {
                let request = request.clone();
                Box::pin(async move { client.search(request).await })
            })
            .await
        {
            LBResult::Ok(response, _, response_time) => {
                let mut status = self.status.lock().await;

                status.response_time.update(&response_time);

                self.notification.status.notify_waiters();

                (response.status, response.pages)
            }
            LBResult::Offline(_) => (GoogolStatus::UnavailableBarrels as i32, vec![]),
        };

        Ok(Response::new(SearchResponse { status, pages }))
    }

    async fn status(
        &self,
        request: Request<GatewayStatusRequest>,
    ) -> Result<Response<GatewayStatusResponse>, Status> {
        debug!("{:#?}", request);

        todo!()
    }
}
