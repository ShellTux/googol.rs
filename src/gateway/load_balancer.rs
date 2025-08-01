use super::status::ResponseTime;
use crate::{
    address::Address,
    proto::{BarrelStatus, barrel_service_client::BarrelServiceClient},
};
use futures::future::BoxFuture;
use log::error;
use std::{collections::HashSet, net::SocketAddr};
use tokio::time::Instant;
use tonic::{
    Response, Status,
    transport::{Channel, Error},
};

#[derive(Debug, Default)]
pub struct Barrel {
    pub address: Address,
    pub online: bool,
    pub index_size_bytes: usize,
}

impl Barrel {
    fn new(address: SocketAddr) -> Self {
        Self {
            address: Address::new(address),
            online: false,
            index_size_bytes: 0,
        }
    }

    fn mark_failure(&mut self) {
        self.online = false;
    }

    fn mark_success(&mut self) {
        self.online = true;
    }

    async fn connect(&self) -> Result<BarrelServiceClient<Channel>, Error> {
        let address = format!("http://{}", self.address);
        BarrelServiceClient::connect(address).await
    }

    fn get_status(&self) -> BarrelStatus {
        BarrelStatus {
            address: self.address.to_string(),
            online: self.online,
            index_size_bytes: self.index_size_bytes as u64,
        }
    }
}

#[derive(Debug, Default)]
pub struct LoadBalancer {
    pub barrels: Vec<Barrel>,
}

#[derive(Debug)]
pub enum LBResult<T> {
    Ok(T, usize, ResponseTime),
    Offline(usize),
}

impl<'a> IntoIterator for &'a mut LoadBalancer {
    type Item = &'a mut Barrel;
    type IntoIter = std::slice::IterMut<'a, Barrel>;

    fn into_iter(self) -> Self::IntoIter {
        self.barrels.iter_mut()
    }
}

impl LoadBalancer {
    pub fn new(barrels: &HashSet<SocketAddr>) -> Self {
        assert!(!barrels.is_empty());

        Self {
            barrels: barrels.iter().map(|addr| Barrel::new(*addr)).collect(),
        }
    }

    pub fn get_barrels_status(&self) -> Vec<BarrelStatus> {
        self.barrels
            .iter()
            .map(|barrel| barrel.get_status())
            .collect()
    }

    pub async fn broadcast<F, T>(&mut self, mut f: F) -> LBResult<Vec<T>>
    where
        F: FnMut(
                &mut Barrel,
                BarrelServiceClient<Channel>,
            ) -> BoxFuture<'static, Result<Response<T>, Status>>
            + Send,
    {
        let mut offline = 0;
        let mut responses = vec![];
        let mut avg_response = ResponseTime::default();

        for barrel in self.into_iter() {
            let start_instant = Instant::now();

            match barrel.connect().await {
                Ok(client) => {
                    if let Ok(response) = f(barrel, client).await {
                        barrel.mark_success();
                        avg_response.new_sample(start_instant);
                        responses.push(response.into_inner());
                    } else {
                        barrel.mark_failure();
                        offline += 1;
                    }
                }
                Err(e) => {
                    barrel.mark_failure();
                    error!("Error connecting to {}: {}", barrel.address, e);
                    offline += 1;
                }
            }
        }

        if !responses.is_empty() {
            LBResult::Ok(responses, offline, avg_response)
        } else {
            let offline = self.barrels.len();
            LBResult::Offline(offline)
        }
    }

    pub async fn send_until<T, F>(&mut self, mut f: F) -> LBResult<T>
    where
        F: FnMut(BarrelServiceClient<Channel>) -> BoxFuture<'static, Result<Response<T>, Status>>
            + Send,
        T: Send,
    {
        let mut offline = 0;
        let mut avg_response = ResponseTime::default();

        for barrel in &mut self.barrels.iter_mut() {
            let start_time = Instant::now();

            match barrel.connect().await {
                Ok(client) => {
                    if let Ok(response) = f(client).await {
                        barrel.mark_success();
                        avg_response.new_sample(start_time);
                        return LBResult::Ok(response.into_inner(), offline, avg_response);
                    }
                }
                Err(e) => {
                    barrel.mark_failure();
                    error!("Error connecting to {}: {}", barrel.address, e);
                    offline += 1;
                }
            }
        }

        let offline = self.barrels.len();
        LBResult::Offline(offline)
    }
}
