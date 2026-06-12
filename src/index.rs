
use crate::NeoError;

pub struct _IndexSummary {
    total_urls: u32,
    urls_indexed: u32,
    index_size: u32,
}

pub fn _index_document(_words: Vec<String>) -> Result<(), NeoError> {
    Ok(())
}
