// Dedupe frontier (HashSet) -> url frontier -> workers request, retry, politeness, etc (RIS?) -> index
// Also dedupe page content
// Frontier: one queue per host, priority queue with timer to select next, give to worker

use std::{collections::{BinaryHeap, VecDeque}, time::SystemTime};
use scraper::{Html, Selector};
use crate::NeoError;

pub struct CrawlSummary {
    pub urls_crawled: usize,
}

pub fn parse_words(document: &Html, selector: &Selector) -> Vec<String> {
    document.select(&selector)
        .map(|element| element.text()
            .map(|text| text.split(' ')
                .map(|word| word.chars()
                    .filter(|ch| ch.is_alphanumeric())
                    .map(|ch| ch.to_ascii_lowercase())
                    .collect()
                )
            ).flatten()
        ).flatten().collect()
}

struct Frontier {
    urls: VecDeque<String>,
    queue_selector: BinaryHeap<SystemTime>,
}

impl Frontier {
    fn new(mut urls: Vec<String>) -> Self {
        urls.dedup();
        let queue_selector = BinaryHeap::new();
        return Frontier{urls: VecDeque::from(urls), queue_selector};
    }
}

pub async fn crawl(urls: Vec<String>) -> Result<CrawlSummary, NeoError> {
    let len = urls.len();
    let frontier = Frontier::new(urls);
    return Ok(CrawlSummary{ urls_crawled: len });
}
