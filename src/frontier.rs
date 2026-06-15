use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::NeoError;
use crate::crawler::{CrawlSummary, crawl};
use crate::index::{IndexSummary, index};

pub struct FrontierRepo {
    root: PathBuf,
}

impl FrontierRepo {
    pub fn open(root: PathBuf) -> Result<Self, NeoError> {
        fs::create_dir_all(root.join("frontiers"))?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn create_frontier(&self, name: &str, switch_to_it: bool) -> Result<(), NeoError> {
        validate_name(name)?;
        let dir = self.frontier_dir(name);
        if dir.exists() {
            return Err(NeoError::Message(format!(
                "frontier '{name}' already exists"
            )));
        }

        fs::create_dir_all(&dir)?;
        self.write_atomic(dir.join("frontier.txt"), b"")?;
        self.write_atomic(dir.join("library.txt"), b"")?;
        self.write_atomic(dir.join("meta.txt"), format!("name={name}\n").as_bytes())?;
        self.write_atomic(dir.join("crawl.jsonl"), b"")?;

        if switch_to_it {
            self.set_current_frontier(Some(name))?;
        }

        Ok(())
    }

    pub fn switch_frontier(&self, name: &str) -> Result<(), NeoError> {
        validate_name(name)?;
        if !self.frontier_dir(name).is_dir() {
            return Err(NeoError::Message(format!(
                "frontier '{name}' does not exist"
            )));
        }
        self.set_current_frontier(Some(name))
    }

    pub fn rename_frontier(&self, name: &str, new_name: &str) -> Result<(), NeoError> {
        validate_name(name)?;
        validate_name(new_name)?;

        let source = self.frontier_dir(name);
        let target = self.frontier_dir(new_name);
        if !source.is_dir() {
            return Err(NeoError::Message(format!(
                "frontier '{name}' does not exist"
            )));
        }
        if target.exists() {
            return Err(NeoError::Message(format!(
                "frontier '{new_name}' already exists"
            )));
        }

        fs::rename(&source, &target)?;
        self.write_atomic(
            target.join("meta.txt"),
            format!("name={new_name}\n").as_bytes(),
        )?;

        if self.current_frontier()?.as_deref() == Some(name) {
            self.set_current_frontier(Some(new_name))?;
        }

        Ok(())
    }

    pub fn list_frontiers(&self) -> Result<Vec<String>, NeoError> {
        let mut frontiers = Vec::new();
        for entry in fs::read_dir(self.root.join("frontiers"))? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                frontiers.push(entry.file_name().to_string_lossy().into_owned());
            }
        }
        frontiers.sort();
        Ok(frontiers)
    }

    pub fn delete_frontier(&self, name: &str) -> Result<(), NeoError> {
        validate_name(name)?;
        let dir = self.frontier_dir(name);
        if !dir.is_dir() {
            return Err(NeoError::Message(format!(
                "frontier '{name}' does not exist"
            )));
        }
        fs::remove_dir_all(dir)?;
        if self.current_frontier()?.as_deref() == Some(name) {
            self.set_current_frontier(None)?;
        }
        Ok(())
    }

    pub fn add_url(&self, url: &str) -> Result<(), NeoError> {
        validate_url(url)?;
        let frontier = self.require_current_frontier()?;
        let mut frontier_urls = self.read_frontier(self.frontier_file(&frontier))?;
        frontier_urls.push(url.to_string());
        let mut library_urls = self.read_frontier(self.library_file(&frontier))?;
        library_urls.push(url.to_string());
        self.write_frontier(self.frontier_file(&frontier), &frontier_urls)?;
        self.write_frontier(self.library_file(&frontier), &library_urls)
    }

    pub fn size(&self) -> Result<usize, NeoError> {
        let frontier = self.require_current_frontier()?;
        Ok(self.read_frontier(self.frontier_file(&frontier))?.len())
    }

    pub fn pop_url(&self, index: Option<usize>) -> Result<String, NeoError> {
        let frontier = self.require_current_frontier()?;
        let mut urls = self.read_frontier(self.frontier_file(&frontier))?;
        let index = index.unwrap_or(0);
        if urls.is_empty() {
            return Err(NeoError::Message(format!("frontier '{frontier}' is empty")));
        }
        if index >= urls.len() {
            return Err(NeoError::Message(format!(
                "index {index} is out of bounds for frontier '{frontier}'"
            )));
        }
        let url = urls.remove(index);
        self.write_frontier(self.frontier_file(&frontier), &urls)?;
        Ok(url)
    }

    pub fn delete_url(&self, url: &str) -> Result<(), NeoError> {
        validate_url(url)?;
        let frontier = self.require_current_frontier()?;
        let library_urls = self.read_frontier(self.library_file(&frontier))?;
        let original_len = library_urls.len();
        let filtered_library_urls: Vec<_> = library_urls.into_iter().filter(|entry| entry != url).collect();
        if filtered_library_urls.len() == original_len {
            return Err(NeoError::Message(format!(
                "url not found in frontier '{frontier}'"
            )));
        }

        let frontier_urls = self.read_frontier(self.frontier_file(&frontier))?;
        let filtered_frontier_urls: Vec<_> = frontier_urls.into_iter().filter(|entry| entry != url).collect();

        self.write_frontier(self.frontier_file(&frontier), &filtered_frontier_urls)?;
        self.write_frontier(self.library_file(&frontier), &filtered_library_urls)
    }

    pub fn current_frontier(&self) -> Result<Option<String>, NeoError> {
        let path = self.current_path();
        if !path.exists() {
            return Ok(None);
        }

        let name = fs::read_to_string(path)?.trim().to_string();
        if name.is_empty() {
            return Ok(None);
        }
        Ok(Some(name))
    }

    pub async fn crawl_repo(&self, library: bool) -> Result<CrawlSummary, NeoError> {
        let frontier = self.require_current_frontier()?;
        let urls = if library { self.read_frontier(self.library_file(&frontier)) } else { self.read_frontier(self.frontier_file(&frontier)) }?;
        crawl(urls, self.crawl_file(&frontier)).await
    }

    pub async fn index_repo(&self, consume: bool) -> Result<IndexSummary, NeoError> {
        let frontier = self.require_current_frontier()?;
        index(self.crawl_file(&frontier), consume).await
    }

    pub fn search_command(&self, query: &str) -> Result<Vec<String>, NeoError> {
        // TODO
        println!("{query}");
        Ok(vec![])
    }

    pub fn require_current_frontier(&self) -> Result<String, NeoError> {
        self.current_frontier()?.ok_or_else(|| {
            NeoError::Message("no active frontier; run `neo frontier start <name>`".into())
        })
    }

    fn set_current_frontier(&self, frontier: Option<&str>) -> Result<(), NeoError> {
        match frontier {
            Some(name) => self.write_atomic(self.current_path(), format!("{name}\n").as_bytes()),
            None => {
                let path = self.current_path();
                if path.exists() {
                    fs::remove_file(path)?;
                }
                Ok(())
            }
        }
    }

    fn read_frontier(&self, path: PathBuf) -> Result<Vec<String>, NeoError> {
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(path)?;
        Ok(content
            .lines()
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
            .collect())
    }

    fn write_frontier(&self, path: PathBuf, urls: &[String]) -> Result<(), NeoError> {
        let mut data = urls.join("\n");
        if !data.is_empty() {
            data.push('\n');
        }
        self.write_atomic(path, data.as_bytes())
    }

    fn write_atomic(&self, path: PathBuf, bytes: &[u8]) -> Result<(), NeoError> {
        let temp = path.with_extension(format!("tmp-{}", unique_suffix()));
        fs::write(&temp, bytes)?;
        fs::rename(temp, path)?;
        Ok(())
    }

    fn frontier_dir(&self, name: &str) -> PathBuf {
        self.root.join("frontiers").join(name)
    }

    pub fn frontier_file(&self, name: &str) -> PathBuf {
        self.frontier_dir(name).join("frontier.txt")
    }

    pub fn library_file(&self, name: &str) -> PathBuf {
        self.frontier_dir(name).join("library.txt")
    }

    pub fn crawl_file(&self, name: &str) -> PathBuf {
        self.frontier_dir(name).join("crawl.jsonl")
    }

    fn current_path(&self) -> PathBuf {
        self.root.join("current_frontier")
    }
}

fn validate_name(name: &str) -> Result<(), NeoError> {
    if name.is_empty() {
        return Err(NeoError::Message("frontier name cannot be empty".into()));
    }
    if name.contains('/') || name.contains('\\') {
        return Err(NeoError::Message(
            "frontier name cannot contain path separators".into(),
        ));
    }
    Ok(())
}

fn validate_url(url: &str) -> Result<(), NeoError> {
    if url.starts_with("http://") || url.starts_with("https://") {
        Ok(())
    } else {
        Err(NeoError::Message(
            "url must start with http:// or https://".into(),
        ))
    }
}

pub fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}
