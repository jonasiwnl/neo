use dashmap::DashSet;
use governor::{Quota, RateLimiter};
use reqwest::Client;
use scraper::{Html, Selector};
use std::num::NonZeroU32;
use std::sync::Arc;
use tokio::sync::mpsc;
use url::Url;

use crate::NeoError;

// Shared state across all crawler tasks
struct CrawlerState {
    client: Client,
    visited: DashSet<String>,
    limiter: RateLimiter<
        governor::state::NotKeyed,
        governor::state::InMemoryState,
        governor::clock::DefaultClock,
    >,
}

impl CrawlerState {
    fn new(requests_per_second: u32) -> Self {
        let client = Client::builder()
            .user_agent("JonasNeo/1.0")
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("failed to build HTTP client");

        // Token bucket: refill `requests_per_second` tokens every second
        let quota = Quota::per_second(NonZeroU32::new(requests_per_second).unwrap());
        let limiter = RateLimiter::direct(quota);

        Self {
            client,
            visited: DashSet::new(),
            limiter,
        }
    }
}

async fn crawl_page(state: Arc<CrawlerState>, url: Url) -> Result<(), NeoError> {
    state.limiter.until_ready().await;
    let response = state.client.get(url).send().await?;

    Ok(())
}
