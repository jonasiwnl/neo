use std::{collections::HashSet, fmt, fs::{File, remove_file}, io::{BufRead, BufReader}, path::PathBuf};

use scraper::{Html, Selector};

use crate::{NeoError, crawler::CrawledPage, util::humansize};

pub struct IndexSummary {
    dictionary_size: u64,
    posting_list_size: u64,
    docstore_size: u64,
}

impl fmt::Display for IndexSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "index file size: {}\n\tdictionary: {}\n\tposting list: {}\n\tdocstore: {}",
            humansize(self.dictionary_size + self.posting_list_size + self.docstore_size),
            humansize(self.dictionary_size),
            humansize(self.posting_list_size),
            humansize(self.docstore_size),
        )
    }
}

struct Dictionary {
    dictionary: HashSet<String, u64>, // TODO: varint?
    posting_lists: Vec<PostingList>,
}

struct PostingList {
    posts: Vec<Post>,
}

struct Post {
    docid_or_delta: u64, // TODO: varint
    positions: Vec<u64>, // TODO: varint
}

struct DocStore {
    // docid -> meta
    docstore: HashSet<u64, DocMeta>,
}

struct DocMeta {
    url: String,
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

pub async fn index(crawl_file: PathBuf, consume: bool) -> Result<IndexSummary, NeoError> {
    // First, let's iterate over the crawled documents
    let file = File::open(&crawl_file)?;
    let reader = BufReader::new(file);

    let mut docid = 0;
    for line in reader.lines() {
        let line = line?;
        let page: CrawledPage = serde_json::from_str(&line)?;
        let document = Html::parse_document(&page.html);
        for tag in [ "title", "h1", "p" ] {
            let selector =  Selector::parse(tag).unwrap();
            let words = parse_words(&document, &selector);
        }
        eprintln!("indexing {}", page.url);

        docid += 1;
    }

    if consume {
        remove_file(&crawl_file)?;
        eprintln!("deleted {}", crawl_file.into_os_string().into_string().unwrap());
    }

    Ok(IndexSummary { dictionary_size: 0, posting_list_size: 0, docstore_size: 0 })
}

// Use ML to predict page quality (?)
