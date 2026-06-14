use std::{path::PathBuf, fmt};

use scraper::{Html, Selector};

use crate::{NeoError, util::humansize};

pub struct IndexSummary {
    index_size: u64,
}

impl fmt::Display for IndexSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "index file size: {}", humansize(self.index_size))
    }
}

// let document = Html::parse_document(&html);
// for tag in [ "title", "h1", "p" ] {
//     let selector =  Selector::parse(tag).unwrap();
//     let words = parse_words(&document, &selector);
//     index_document(words)?;
// }
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

pub async fn index(crawl_file: PathBuf) -> Result<IndexSummary, NeoError> {
    Ok(IndexSummary { index_size: 0 })
}
