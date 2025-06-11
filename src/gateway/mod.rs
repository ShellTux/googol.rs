//! # Gateway Module
//!
//! This module defines the `Gateway` struct and its associated implementation, providing
//! a gRPC service for managing web crawling and indexing operations. It interacts with
//! load balancers, queues, and maintains status to facilitate distributed crawling.

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
    wait_for_enter,
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

/// Represents notifications used for signaling status changes and queue updates.
#[derive(Debug, Default)]
/// Notification signals for the Gateway.
pub struct Notification {
    /// Notifies when the status has changed.
    pub status: Notify,
    /// Notifies when the queue has new items.
    pub queue: Notify,
}

#[derive(Debug, Default)]
/// The main Gateway struct implementing the gRPC service for crawling operations.
/// Gateway handles crawling, indexing, and status reporting.
pub struct Gateway {
    /// The address of this gateway instance.
    pub address: Address,
    /// Queue managing URLs to crawl.
    pub queue: AsyncMutex<Queue>,
    /// Load balancer managing connections to barrels.
    pub load_balancer: AsyncMutex<LoadBalancer>,
    /// Current status of the gateway.
    pub status: AsyncMutex<GatewayStatus>,
    /// Notifications for status and queue updates.
    pub notification: Notification,
    /// Toggle interactive mode to wait for user input
    pub interactive: bool,
    // TODO: Add caching mechanisms.
}

impl Gateway {
    /// Creates a new Gateway instance with default values.
    ///
    /// # Returns
    /// A new `Gateway`.
    ///
    /// # Examples
    ///
    /// ```
    /// use googol::gateway::Gateway;
    ///
    /// let gw = Gateway::create();
    /// ```
    pub fn create() -> Self {
        Self::default()
    }

    /// Sets the address for the Gateway.
    ///
    /// # Arguments
    /// * `address` - The `Address` to assign.
    ///
    /// # Returns
    /// The updated `Gateway` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use googol::{address:: Address, gateway::Gateway};
    ///
    /// let addr = Address::new("127.0.0.1:8080".parse().unwrap());
    /// let gw = Gateway::create().with_address(addr);
    /// ```
    pub fn with_address(mut self, address: Address) -> Self {
        self.address = address;
        self
    }

    /// Sets the load balancer for the Gateway asynchronously.
    ///
    /// # Arguments
    /// * `lb` - The `LoadBalancer` instance.
    ///
    /// # Returns
    /// The updated `Gateway`.
    ///
    /// # Examples
    ///
    /// ```
    /// use googol::gateway::{Gateway, load_balancer::LoadBalancer};
    /// use std::collections::HashSet;
    ///
    /// let lb = LoadBalancer::new(&["127.0.0.1:50052"].iter().map(|a| a.parse().unwrap()).collect());
    /// let gw = Gateway::create().with_load_balancer(lb);
    /// ```
    pub async fn with_load_balancer(self, lb: LoadBalancer) -> Self {
        *self.load_balancer.lock().await = lb;
        self
    }

    /// Sets the URL queue for the Gateway asynchronously.
    ///
    /// # Arguments
    /// * `queue` - The `Queue` instance.
    ///
    /// # Returns
    /// The updated `Gateway`.
    ///
    /// # Examples
    ///
    /// ```
    /// use googol::gateway::{Gateway, queue::Queue};
    ///
    /// let queue = Queue::create();
    /// let gw = Gateway::create().with_queue(queue);
    /// ```
    pub async fn with_queue(self, queue: Queue) -> Self {
        *self.queue.lock().await = queue;
        self
    }

    /// Sets interactive flag for the Gateway.
    ///
    /// # Arguments
    /// * `interactive` - The `bool` to assign.
    ///
    /// # Returns
    /// The updated `Gateway` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use googol::gateway::Gateway;
    ///
    /// let gw = Gateway::create().with_interactive(true);
    /// ```
    pub fn with_interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    /// Creates a Gateway from a configuration.
    ///
    /// # Arguments
    /// * `config` - Reference to `GatewayConfig`.
    ///
    /// # Returns
    /// A `Gateway` instance configured accordingly.
    ///
    /// # Examples
    ///
    /// ```
    /// use googol::{settings::gateway::{GatewayConfig, DomainsFilter}, gateway::Gateway, address::Address};
    /// use std::collections::VecDeque;
    ///
    /// let config = GatewayConfig {
    ///     address: "127.0.0.1:8080".parse().unwrap(),
    ///     queue: VecDeque::new(),
    ///     barrels: ["127.0.0.1:50052"].iter().map(|a| a.parse().unwrap()).collect(),
    ///     domains_filter: DomainsFilter::default(),
    /// };
    /// let gw = Gateway::from(&config);
    /// ```
    pub async fn from(config: &GatewayConfig) -> Self {
        Self::create()
            .with_address(Address::new(config.address))
            .with_load_balancer(LoadBalancer::new(&config.barrels))
            .await
            .with_queue(Queue::create().with_domains_filter(&config.domains_filter))
            .await
    }
}

/// Implementation of the gRPC GatewayService trait for the Gateway.
#[tonic::async_trait]
impl GatewayService for Gateway {
    /// Handles broadcasting an index to barrels.
    ///
    /// # Arguments
    /// * `request` - The gRPC request containing `BroadcastIndexRequest`.
    ///
    /// # Returns
    /// A response with `BroadcastIndexResponse`.
    async fn broadcast_index(
        &self,
        request: Request<BroadcastIndexRequest>,
    ) -> Result<Response<BroadcastIndexResponse>, Status> {
        debug!("{:#?}", request);

        unimplemented!()
    }

    /// Consults backlinks from the load balancer.
    ///
    /// # Arguments
    /// * `request` - The gRPC request containing `BacklinksRequest`.
    ///
    /// # Returns
    /// A response with `BacklinksResponse`.
    async fn consult_backlinks(
        &self,
        request: Request<BacklinksRequest>,
    ) -> Result<Response<BacklinksResponse>, Status> {
        debug!("{:#?}", request);

        let request = request.into_inner();

        // Send request to load balancer, retrying until success or offline.
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

    /// Consults outlinks from the load balancer.
    ///
    /// # Arguments
    /// * `request` - The gRPC request containing `OutlinksRequest`.
    ///
    /// # Returns
    /// A response with `OutlinksResponse`.
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

    /// Dequeues a URL from the queue, waiting if necessary.
    ///
    /// # Arguments
    /// * `request` - The gRPC request containing `DequeueRequest`.
    ///
    /// # Returns
    /// A response with `DequeueResponse`.
    async fn dequeue_url(
        &self,
        request: Request<DequeueRequest>,
    ) -> Result<Response<DequeueResponse>, Status> {
        debug!("{:#?}", request);

        // Wait until a URL is available in the queue.
        let url = loop {
            if let Some(url) = self.queue.lock().await.dequeue() {
                break url;
            }

            // Wait for notification that a URL has been enqueued.
            self.notification.queue.notified().await;
        }
        .to_string();

        // Notify status listeners of queue change.
        self.notification.status.notify_waiters();

        Ok(Response::new(DequeueResponse { url }))
    }

    /// Enqueues a URL into the queue.
    ///
    /// # Arguments
    /// * `request` - The gRPC request containing `EnqueueRequest`.
    ///
    /// # Returns
    /// A response with `EnqueueResponse`.
    async fn enqueue_url(
        &self,
        request: Request<EnqueueRequest>,
    ) -> Result<Response<EnqueueResponse>, Status> {
        debug!("{:#?}", request);

        let request = request.into_inner();

        // Parse URL and enqueue if valid.
        let (status, queue) = match Url::parse(&request.url) {
            Err(e) => {
                error!("Invalid url: `{}`: {}", &request.url, e);
                (GoogolStatus::InvalidUrl, vec![])
            }
            Ok(url) => self.queue.lock().await.enqueue(url),
        };

        // Notify status listeners if enqueue succeeded.
        if status == GoogolStatus::Success {
            self.notification.status.notify_waiters();
        }

        let status = status as i32;

        Ok(Response::new(EnqueueResponse { status, queue }))
    }

    /// Checks the health of the gateway.
    ///
    /// # Arguments
    /// * `request` - The gRPC request containing `HealthRequest`.
    ///
    /// # Returns
    /// A response with `HealthResponse`.
    async fn health(
        &self,
        request: Request<HealthRequest>,
    ) -> Result<Response<HealthResponse>, Status> {
        debug!("{:#?}", request);

        let response = HealthResponse {
            status: format!("OK: Online. Listening at {}...", self.address),
        };

        if self.interactive {
            wait_for_enter!("Press Enter to send \x1b[32m{:#?}\x1b[0m...", &response);
        }

        Ok(Response::new(response))
    }

    /// Performs an index operation.
    ///
    /// # Arguments
    /// * `request` - The gRPC request containing `IndexRequest`.
    ///
    /// # Returns
    /// A response with `IndexResponse`.
    async fn index(
        &self,
        request: Request<IndexRequest>,
    ) -> Result<Response<IndexResponse>, Status> {
        debug!("{:#?}", request);

        let request = request.into_inner();

        // If outlinks are provided, enqueue them.
        if let Some(index) = &request.index {
            let mut queue = self.queue.lock().await;

            for url in index.outlinks.iter().map(|url| Url::parse(url).unwrap()) {
                queue.enqueue(url);
            }
        }

        // Broadcast index to barrels.
        let online = match self
            .load_balancer
            .lock()
            .await
            .broadcast(|_, mut client| {
                let request = request.clone();

                Box::pin(async move {
                    // Send index request to each barrel.

                    //if let Ok(response) = response {
                    //    let response = response.into_inner();
                    //
                    //    barrel.index_size_bytes = response.size_bytes as usize;
                    //}

                    // Additional response handling can be added here.
                    client.index(request).await
                })
            })
            .await
        {
            LBResult::Ok(responses, _, _) => responses.len(),
            LBResult::Offline(_) => 0,
        };

        if online == 0 {
            // TODO: Handle caching for later index sending if all barrels are offline.
        }

        Ok(Response::new(IndexResponse { size_bytes: 0 }))
    }

    /// Retrieves real-time status information.
    ///
    /// # Arguments
    /// * `request` - The gRPC request containing `RealTimeStatusRequest`.
    ///
    /// # Returns
    /// A response with `RealTimeStatusResponse`.
    async fn real_time_status(
        &self,
        request: Request<RealTimeStatusRequest>,
    ) -> Result<Response<RealTimeStatusResponse>, Status> {
        debug!("{:#?}", request);

        // Wait for status update notification.
        self.notification.status.notified().await;

        // Gather current system statuses.
        let barrels = self.load_balancer.lock().await.get_barrels_status();
        let queue = self.queue.lock().await.into_vec();
        let status = self.status.lock().await;

        // Compute average response time.
        let avg_response_time_ms = status.response_time.miliseconds;

        // Collect top 10 searches.
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

    /// Requests an index operation.
    ///
    /// # Arguments
    /// * `request` - The gRPC request containing `RequestIndexRequest`.
    ///
    /// # Returns
    /// A response with `RequestIndexResponse`.
    async fn request_index(
        &self,
        request: Request<RequestIndexRequest>,
    ) -> Result<Response<RequestIndexResponse>, Status> {
        debug!("{:#?}", request);

        unimplemented!()
    }

    /// Performs a search operation.
    ///
    /// # Arguments
    /// * `request` - The gRPC request containing `SearchRequest`.
    ///
    /// # Returns
    /// A response with `SearchResponse`.
    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        debug!("{:#?}", request);

        let request = request.into_inner();

        // Send search request to load balancer.
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

                // Update response time and top searches.
                status.response_time.update(&response_time);

                for word in &request.words {
                    status.top_searches.add_search(word);
                }

                // Notify waiting tasks about status update.
                self.notification.status.notify_waiters();

                (response.status, response.pages)
            }
            LBResult::Offline(_) => (GoogolStatus::UnavailableBarrels as i32, vec![]),
        };

        Ok(Response::new(SearchResponse { status, pages }))
    }

    /// Retrieves overall gateway status.
    ///
    /// # Arguments
    /// * `request` - The gRPC request containing `GatewayStatusRequest`.
    ///
    /// # Returns
    /// A response with `GatewayStatusResponse`.
    async fn status(
        &self,
        request: Request<GatewayStatusRequest>,
    ) -> Result<Response<GatewayStatusResponse>, Status> {
        debug!("{:#?}", request);

        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_default_interactive_off() {
        let gateway = Gateway::default();

        assert!(!gateway.interactive);

        let gateway = gateway.with_interactive(true);

        assert!(gateway.interactive);
    }
}
