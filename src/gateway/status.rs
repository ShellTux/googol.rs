use std::collections::VecDeque;

use tokio::time::Instant;
use url::Url;

use crate::top_searches::TopSearches;

#[derive(Debug, Default)]
pub struct Queue(VecDeque<Url>);

impl Queue {
    pub fn push_back(&mut self, url: Url) {
        self.0.push_back(url);
    }

    pub fn pop_front(&mut self) -> Option<Url> {
        self.0.pop_front()
    }
}

#[derive(Debug, Default)]
pub struct ResponseTime {
    pub miliseconds: f32,
    pub count: usize,
}

impl ResponseTime {
    pub fn new_sample(&mut self, start_instant: Instant) {
        let duration = start_instant.elapsed().as_secs_f32() * 1000.;
        let count = self.count as f32;

        self.miliseconds = ((self.miliseconds * count) + duration) / (count + 1.);
        self.count += 1;
    }

    pub fn update(&mut self, response_time: &ResponseTime) {
        self.miliseconds = ((self.miliseconds * self.count as f32)
            + (response_time.miliseconds * response_time.count as f32))
            / (self.count + response_time.count) as f32;
    }
}

#[derive(Debug, Default)]
pub struct GatewayStatus {
    pub top_searches: TopSearches,
    pub response_time: ResponseTime,
}
