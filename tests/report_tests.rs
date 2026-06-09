use rust_crawler::models::{CrawlResult, CrawlStats, PageData, TaskStatus};
use rust_crawler::report::{build_report, render_report};

fn sample_result() -> CrawlResult {
    CrawlResult {
        pages: vec![
            PageData {
                url: "https://example.com".to_string(),
                depth: 0,
                title: Some("Home".to_string()),
                status_code: Some(200),
                links: vec![
                    "https://example.com/a".to_string(),
                    "https://example.com/b".to_string(),
                ],
                content_length: 300,
                status: TaskStatus::Success,
                error_message: None,
            },
            PageData {
                url: "https://example.com/a".to_string(),
                depth: 1,
                title: None,
                status_code: None,
                links: vec![],
                content_length: 0,
                status: TaskStatus::Failed,
                error_message: Some("timeout".to_string()),
            },
        ],
        stats: CrawlStats {
            retried_requests: 2,
            ..CrawlStats::default()
        },
    }
}

#[test]
fn summarizes_crawl_result() {
    let summary = build_report(&sample_result());
    assert_eq!(summary.total_pages, 2);
    assert_eq!(summary.success_pages, 1);
    assert_eq!(summary.failed_pages, 1);
    assert_eq!(summary.average_content_length, 150);
    assert_eq!(summary.retried_requests, 2);
}

#[test]
fn renders_summary_text() {
    let rendered = render_report(&build_report(&sample_result()));
    assert!(rendered.contains("Top domains"));
    assert!(rendered.contains("example.com"));
}
