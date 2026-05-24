use crate::cli::{Cli, PopArgs};
use crate::frontier::{FrontierRepo, unique_suffix};
use crate::run_with_root;
use std::env;
use std::fs;
use std::path::PathBuf;

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

    assert_eq!(repo.pop_url(None).unwrap(), "https://example.com/a");
    assert_eq!(repo.pop_url(None).unwrap(), "https://example.com/b");
}

#[test]
fn popping_url_keeps_it_in_library() {
    let root = temp_test_dir("library-pop");
    let repo = FrontierRepo::open(root.clone()).unwrap();

    repo.create_frontier("reading", true).unwrap();
    repo.add_url("https://example.com/a").unwrap();
    repo.add_url("https://example.com/b").unwrap();
    repo.pop_url(None).unwrap();

    let library = fs::read_to_string(root.join("frontiers/reading/library.txt")).unwrap();
    assert_eq!(library, "https://example.com/a\nhttps://example.com/b\n");
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

    assert_eq!(repo.pop_url(None).unwrap(), "https://example.com/b");
}

#[test]
fn delete_url_removes_entry_from_library() {
    let root = temp_test_dir("delete-library");
    let repo = FrontierRepo::open(root.clone()).unwrap();

    repo.create_frontier("queue", true).unwrap();
    repo.add_url("https://example.com/a").unwrap();
    repo.add_url("https://example.com/b").unwrap();
    repo.delete_url("https://example.com/a").unwrap();

    let frontier = fs::read_to_string(root.join("frontiers/queue/frontier.txt")).unwrap();
    let library = fs::read_to_string(root.join("frontiers/queue/library.txt")).unwrap();
    assert_eq!(frontier, "https://example.com/b\n");
    assert_eq!(library, "https://example.com/b\n");
}

#[test]
fn cli_parses_pop_open_flag() {
    let cli = Cli::parse(vec!["pop".into(), "--open".into()]).unwrap();
    assert_eq!(
        cli,
        Cli::Pop(PopArgs {
            index: None,
            open: true
        })
    );
}

#[test]
fn cli_parses_pop_index() {
    let cli = Cli::parse(vec!["pop".into(), "2".into(), "--open".into()]).unwrap();
    assert_eq!(
        cli,
        Cli::Pop(PopArgs {
            index: Some(2),
            open: true
        })
    );
}

#[test]
fn pop_supports_indexed_removal() {
    let root = temp_test_dir("pop-index");
    let repo = FrontierRepo::open(root).unwrap();

    repo.create_frontier("queue", true).unwrap();
    repo.add_url("https://example.com/a").unwrap();
    repo.add_url("https://example.com/b").unwrap();
    repo.add_url("https://example.com/c").unwrap();

    assert_eq!(repo.pop_url(Some(1)).unwrap(), "https://example.com/b");
    assert_eq!(repo.pop_url(None).unwrap(), "https://example.com/a");
    assert_eq!(repo.pop_url(None).unwrap(), "https://example.com/c");
}

#[test]
fn pop_rejects_out_of_bounds_index() {
    let root = temp_test_dir("pop-oob");
    let repo = FrontierRepo::open(root).unwrap();

    repo.create_frontier("queue", true).unwrap();
    repo.add_url("https://example.com/a").unwrap();

    let err = repo.pop_url(Some(1)).unwrap_err();
    assert_eq!(
        err.to_string(),
        "index 1 is out of bounds for frontier 'queue'"
    );
}

#[test]
fn size_reports_number_of_links() {
    let root = temp_test_dir("size-output");
    let repo = FrontierRepo::open(root.clone()).unwrap();
    repo.create_frontier("queue", true).unwrap();
    repo.add_url("https://example.com/a").unwrap();
    repo.add_url("https://example.com/b").unwrap();

    let mut stdout = Vec::new();
    run_with_root(vec!["size".into()], &mut stdout, root).unwrap();

    assert_eq!(String::from_utf8(stdout).unwrap(), "2\n");
}
