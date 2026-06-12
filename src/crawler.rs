// Dedupe frontier (HashSet) -> url frontier -> workers request, retry, politeness, etc (RIS?) -> index
// Also dedupe page content
// Frontier: one queue per host, priority queue with timer to select next, give to worker

use std::{collections::{BinaryHeap, VecDeque}, sync::Arc, time::SystemTime};
use scraper::{Html, Selector};
use tokio::{sync::{Mutex, mpsc::{Receiver, Sender}}, task::JoinHandle};
use crate::NeoError;

const MAX_RETRIES: u8 = 3;
const WORKER_THREADS: usize = 16;

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

    pub async fn run(mut self, tx: Sender<String>) {
        // In the future, we should have one queue per host, with a priority q with timers to select the next queue to read from
        // Ok, I guess this can infinite loop
        // Also, can we avoid the string copy
        while let Some(url) = self.urls.pop_front() {
            if let Err(_) = tx.send(url.clone()).await {
                self.urls.push_back(url);
            }
        }
        drop(tx);
    }
}

async fn crawl_single_page(url: &String) -> Result<(), NeoError> {
    println!("crawling {}", url);
    Ok(())
}

async fn worker(rx: Arc<Mutex<Receiver<String>>>) {
    loop {
        let url = rx.lock().await.recv().await;
        match url {
            Some(url) => {
                // Log error?
                let mut retries = 0;
                while let Err(_) = crawl_single_page(&url).await && retries < MAX_RETRIES {
                    retries += 1;
                    // Else... log that we dropped a url
                }
            },
            None => break,
        }
    }
}

pub async fn crawl(urls: Vec<String>) -> Result<CrawlSummary, NeoError> {
    let len = urls.len();
    let frontier = Frontier::new(urls);
    let (tx, rx) = tokio::sync::mpsc::channel::<String>(len);
    let rx = Arc::new(Mutex::new(rx));
    let mut handles = Vec::<JoinHandle<()>>::with_capacity(WORKER_THREADS);
    tokio::spawn(frontier.run(tx.clone()));
    for _ in 0..WORKER_THREADS {
        handles.push(tokio::spawn(worker(Arc::clone(&rx))));
    }
    drop(tx);
    // Wait for all workers to finish
    for handle in handles {
        handle.await.unwrap();
    }
    return Ok(CrawlSummary{ urls_crawled: len });
}
