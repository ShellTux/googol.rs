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
use status::GatewayStatus;
use std::collections::VecDeque;
use tokio::sync::{Mutex as AsyncMutex, Notify};
use tonic::{Request, Response, Status};
use url::Url;

pub mod load_balancer;
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
    pub queue: AsyncMutex<VecDeque<Url>>,
    pub load_balancer: AsyncMutex<LoadBalancer>,
    pub status: AsyncMutex<GatewayStatus>,
    pub notification: Notification,
    // TODO: Add caching
}

impl Gateway {
    pub async fn from(config: &GatewayConfig) -> Self {
        let mut gateway = Gateway::default();
        gateway.address = Address::new(config.address);
        *gateway.load_balancer.lock().await = LoadBalancer::new(&config.barrels);
        gateway
    }

    #[allow(private_interfaces)]
    pub async fn enqueue<I>(&self, urls: I) -> (GoogolStatus, Vec<String>)
    where
        I: IntoIterator<Item = Url>,
    {
        let mut queue = self.queue.lock().await;
        let mut new_urls = 0;

        for url in urls {
            if queue.contains(&url) {
                continue;
            }

            queue.push_back(url.clone());
            new_urls += 1;
        }

        if new_urls > 0 {
            self.notification.queue.notify_one();
            self.notification.status.notify_waiters();
        }

        let queue_vec = queue.iter().map(|url| url.to_string()).collect();

        let status = if new_urls > 0 {
            GoogolStatus::AlreadyIndexedUrl
        } else {
            GoogolStatus::Success
        };

        (status, queue_vec)
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
            LBResult::Ok(response, _) => (response.status, response.backlinks),
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
            LBResult::Ok(response, _) => (response.status, response.outlinks),
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
            if let Some(url) = self.queue.lock().await.pop_front() {
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
            Ok(url) => self.enqueue([url]).await,
        };

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

        if let Some(index) = request.index.clone() {
            self.enqueue(index.outlinks.iter().map(|url| Url::parse(url).unwrap()))
                .await;
        }

        let offline = match self
            .load_balancer
            .lock()
            .await
            .broadcast(|mut client| {
                let request = request.clone();

                Box::pin(async move { client.index(request).await })
            })
            .await
        {
            LBResult::Ok(_, _) => 0,
            LBResult::Offline(offline) => offline,
        };

        if offline > 0 {
            // TODO: Caching to send index later to barrels
        }

        Ok(Response::new(IndexResponse {}))
    }

    async fn real_time_status(
        &self,
        request: Request<RealTimeStatusRequest>,
    ) -> Result<Response<RealTimeStatusResponse>, Status> {
        debug!("{:#?}", request);

        self.notification.status.notified().await;

        let barrels = self.load_balancer.lock().await.get_barrels_status();

        let queue = self
            .queue
            .lock()
            .await
            .iter()
            .map(|url| url.to_string())
            .collect();

        let status = self.status.lock().await;

        let avg_response_time_ms = status.response_time.miliseconds;

        let top10_searches = status
            .top_searches
            .top_n(10)
            .iter()
            .map(move |(word, _)| word.clone())
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

        let (status, urls) = match self
            .load_balancer
            .lock()
            .await
            .send_until(|mut client| {
                let request = request.clone();
                Box::pin(async move { client.search(request).await })
            })
            .await
        {
            LBResult::Ok(response, response_time) => {
                let mut status = self.status.lock().await;

                status.response_time.update(&response_time);
                self.status
                    .lock()
                    .await
                    .response_time
                    .update(&response_time);

                self.notification.status.notify_waiters();

                (response.status, response.urls)
            }
            LBResult::Offline(_) => (GoogolStatus::UnavailableBarrels as i32, vec![]),
        };

        Ok(Response::new(SearchResponse { status, urls }))
    }

    async fn status(
        &self,
        request: Request<GatewayStatusRequest>,
    ) -> Result<Response<GatewayStatusResponse>, Status> {
        debug!("{:#?}", request);

        todo!()
    }
}
