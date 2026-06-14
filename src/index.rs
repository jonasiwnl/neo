use std::path::PathBuf;

use scraper::{Html, Selector};

use crate::NeoError;

pub struct IndexSummary {
    total_urls: u32,
    urls_indexed: u32,
    index_size: u32,
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

pub fn index(crawl_file: PathBuf) -> Result<(), NeoError> {
    Ok(())
}
