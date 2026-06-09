use rust_crawler::models::{CrawlResult, CrawlStats, PageData, TaskStatus};
use rust_crawler::storage::{CsvStorage, JsonStorage, Storage};
use std::fs;
use std::path::PathBuf;

fn test_output_path(file_name: &str) -> PathBuf {
    let path = PathBuf::from("tests_artifacts").join(file_name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    path
}

#[test]
fn saves_json_file() {
    let result = CrawlResult {
        pages: vec![PageData {
            url: "https://example.com".to_string(),
            depth: 0,
            title: Some("Example".to_string()),
            status_code: Some(200),
            links: vec!["https://example.com/about".to_string()],
            content_length: 128,
            status: TaskStatus::Success,
            error_message: None,
        }],
        stats: CrawlStats {
            visited_pages: 1,
            success_pages: 1,
            failed_pages: 0,
            discovered_links: 1,
            skipped_links: 0,
            retried_requests: 0,
        },
    };

    let path = test_output_path("result.json");
    JsonStorage.save(&path, &result).unwrap();
    assert!(path.exists());
}

#[test]
fn saves_csv_file() {
    let result = CrawlResult {
        pages: vec![PageData {
            url: "https://example.com".to_string(),
            depth: 0,
            title: Some("Example".to_string()),
            status_code: Some(200),
            links: vec!["https://example.com/about".to_string()],
            content_length: 128,
            status: TaskStatus::Success,
            error_message: None,
        }],
        stats: CrawlStats {
            visited_pages: 1,
            success_pages: 1,
            failed_pages: 0,
            discovered_links: 1,
            skipped_links: 0,
            retried_requests: 0,
        },
    };

    let path = test_output_path("result.csv");
    CsvStorage.save(&path, &result).unwrap();
    assert!(path.exists());
}
