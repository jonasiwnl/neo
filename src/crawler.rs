// Dedupe frontier (HashSet) -> url frontier -> workers request, retry, politeness, etc (RIS?) -> index
// Also dedupe page content
// Frontier: one queue per host, priority queue with timer to select next, give to worker

use std::{collections::{BinaryHeap, VecDeque}, sync::Arc, time::SystemTime, io::{Write, BufWriter}, path::PathBuf, fs::File};

use serde::{Deserialize, Serialize};
use tokio::{sync::{Mutex, mpsc::{Receiver, Sender}}, task::JoinHandle};
use chrono::Utc;

use crate::NeoError;

const MAX_RETRIES: u8 = 3;
const WORKER_THREADS: usize = 16;

pub struct CrawlSummary {
    pub urls_crawled: usize,
}

#[derive(Serialize, Deserialize)]
pub struct CrawledPage {
    url: String,
    fetched_at: String,
    html: String,
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

    pub async fn run(mut self, url_tx: Sender<String>) {
        // In the future, we should have one queue per host, with a priority q with timers to select the next queue to read from
        // Ok, I guess this can infinite loop
        // Also, can we avoid the string copy
        while let Some(url) = self.urls.pop_front() {
            if let Err(_) = url_tx.send(url.clone()).await {
                self.urls.push_back(url);
            }
        }
        drop(url_tx);
    }
}

async fn crawl_single_page(url: &String, write_tx: &Sender<CrawledPage>) -> Result<(), NeoError> {
    dbg!("crawling {}", url);

    let fetched_at = Utc::now().to_rfc3339();
    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        return Ok(()); // TODO: return some error status
    }

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !content_type.contains("text/html") {
        return Ok(());
    }

    let html = response.text().await?;
    let page = CrawledPage{url: url.to_string(), fetched_at, html};

    write_tx.send(page).await?;

    dbg!("crawled {}", url);

    Ok(())
}

async fn writer_fn(mut writer: BufWriter<File>, mut write_rx: Receiver<CrawledPage>) -> Result<(), NeoError> {
    loop {
        let page = write_rx.recv().await;
        match page {
            Some(page) => {
                serde_json::to_writer(&mut writer, &page)?;
                writeln!(writer)?;
            },
            None => break,
        }
    }

    dbg!("writer routine finished.");
    Ok(())
}

async fn worker_fn(url_rx: Arc<Mutex<Receiver<String>>>, write_tx: Sender<CrawledPage>) {
    loop {
        let url = url_rx.lock().await.recv().await;
        match url {
            Some(url) => {
                // Log error?
                let mut retries = 0;
                while let Err(_) = crawl_single_page(&url, &write_tx).await && retries < MAX_RETRIES {
                    retries += 1;
                    // Else... log that we dropped a url
                }
            },
            None => break,
        }
    }

    drop(write_tx);
}

pub async fn crawl(urls: Vec<String>, crawl_file: PathBuf) -> Result<CrawlSummary, NeoError> {
    let len = urls.len();
    let frontier = Frontier::new(urls);

    // Crawl output
    let file = File::create(crawl_file)?;
    let writer = BufWriter::new(file);
    let (write_tx, write_rx) = tokio::sync::mpsc::channel::<CrawledPage>(5 * WORKER_THREADS);
    tokio::spawn(writer_fn(writer, write_rx));

    // Url send/receive
    let (url_tx, url_rx) = tokio::sync::mpsc::channel::<String>(len);
    let url_rx = Arc::new(Mutex::new(url_rx));
    let mut handles = Vec::<JoinHandle<()>>::with_capacity(WORKER_THREADS);
    tokio::spawn(frontier.run(url_tx.clone()));
    drop(url_tx);

    for _ in 0..WORKER_THREADS {
        handles.push(tokio::spawn(worker_fn(Arc::clone(&url_rx), write_tx.clone())));
    }
    drop(write_tx);

    // Wait for all workers to finish
    for handle in handles {
        handle.await.unwrap();
    }

    return Ok(CrawlSummary{ urls_crawled: len });
}
