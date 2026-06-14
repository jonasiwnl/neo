use std::env;
use std::fs;
use std::path::PathBuf;

use scraper::{Html, Selector};

use crate::cli::{Cli, PopArgs};
use crate::frontier::{FrontierRepo, unique_suffix};
use crate::run_with_root;
use crate::index::parse_words;
use crate::util::humansize;

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

#[tokio::test]
async fn size_reports_number_of_links() {
    let root = temp_test_dir("size-output");
    let repo = FrontierRepo::open(root.clone()).unwrap();
    repo.create_frontier("queue", true).unwrap();
    repo.add_url("https://example.com/a").unwrap();
    repo.add_url("https://example.com/b").unwrap();

    let mut stdout = Vec::new();
    run_with_root(vec!["size".into()], &mut stdout, root).await.unwrap();

    assert_eq!(String::from_utf8(stdout).unwrap(), "2\n");
}

#[test]
fn parse_words_returns_alphanumeric_words_paragraph_tag() {
    let fragment = Html::parse_fragment(r#"<p>rust creAte; hello World!</p>"#);
    let selector = Selector::parse("p").unwrap();
    assert_eq!(vec!["rust", "create", "hello", "world"], parse_words(&fragment, &selector));

    let fragment = Html::parse_fragment(r#"<h1>rust creAte; hello World!</p>"#);
    assert_eq!(Vec::<String>::new(), parse_words(&fragment, &selector));
    let fragment = Html::parse_fragment(r#"<title>rust creAte; hello World!</title>"#);
    assert_eq!(Vec::<String>::new(), parse_words(&fragment, &selector));

    let selector = Selector::parse("title").unwrap();
    assert_eq!(vec!["rust", "create", "hello", "world"], parse_words(&fragment, &selector));
}

#[test]
fn humansize_formats_bytes_with_binary_units() {
    let cases = [
        (0, "0B"),
        (1, "1B"),
        (1023, "1023B"),
        (1024, "1.0KB"),
        (1536, "1.5KB"),
        ((1024 << 10) - 1, "1024.0KB"),
        (1024 << 10, "1.0MB"),
        (5 * (1024 << 10) + 512 * 1024, "5.5MB"),
        ((1024 << 20) - 1, "1024.0MB"),
        (1024 << 20, "1.0GB"),
        (3 * (1024 << 20) + 300 * (1024 << 10), "3.3GB"),
        ((1024 << 30) - 1, "1024.0GB"),
        (1024 << 30, "1.0TB"),
        (2 * (1024 << 30) + 512 * (1024 << 20), "2.5TB"),
    ];

    for (bytes, expected) in cases {
        assert_eq!(humansize(bytes), expected, "bytes={bytes}");
    }
}
