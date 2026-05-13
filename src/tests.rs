use std::env;
use std::fs;
use std::path::PathBuf;
use crate::cli::Cli;
use crate::frontier::{FrontierRepo, unique_suffix};

pub fn temp_test_dir(name: &str) -> PathBuf {
    let path = env::temp_dir().join(format!("neo-test-{name}-{}", unique_suffix()));
    if path.exists() {
        fs::remove_dir_all(&path).unwrap();
    }
    path
}

#[test]
fn frontier_round_trip_is_fifo() {
    let root = temp_test_dir("fifo");
    let repo = FrontierRepo::open(root).unwrap();

    repo.create_frontier("reading", true).unwrap();
    repo.add_url("https://example.com/a").unwrap();
    repo.add_url("https://example.com/b").unwrap();

    assert_eq!(repo.pop_url().unwrap(), "https://example.com/a");
    assert_eq!(repo.pop_url().unwrap(), "https://example.com/b");
}

#[test]
fn renaming_current_frontier_updates_selection() {
    let root = temp_test_dir("rename");
    let repo = FrontierRepo::open(root).unwrap();

    repo.create_frontier("first", true).unwrap();
    repo.rename_frontier("first", "second").unwrap();

    assert_eq!(repo.current_frontier().unwrap(), Some("second".into()));
    assert_eq!(repo.list_frontiers().unwrap(), vec!["second".to_string()]);
}

#[test]
fn deleting_current_frontier_clears_selection() {
    let root = temp_test_dir("delete-frontier");
    let repo = FrontierRepo::open(root).unwrap();

    repo.create_frontier("active", true).unwrap();
    repo.delete_frontier("active").unwrap();

    assert_eq!(repo.current_frontier().unwrap(), None);
}

#[test]
fn delete_url_removes_matching_entries() {
    let root = temp_test_dir("delete-url");
    let repo = FrontierRepo::open(root).unwrap();

    repo.create_frontier("queue", true).unwrap();
    repo.add_url("https://example.com/a").unwrap();
    repo.add_url("https://example.com/b").unwrap();
    repo.delete_url("https://example.com/a").unwrap();

    assert_eq!(repo.pop_url().unwrap(), "https://example.com/b");
}

#[test]
fn cli_parses_pop_open_flag() {
    let cli = Cli::parse(vec!["pop".into(), "--open".into()]).unwrap();
    assert_eq!(cli, Cli::Pop { open: true });
}
