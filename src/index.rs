use crate::NeoError;

pub struct _IndexSummary {
    total_urls: u32,
    urls_indexed: u32,
    index_size: u32,
}

pub fn index(_urls: Vec<String>) -> Result<(), NeoError> {
    Ok(())
}
