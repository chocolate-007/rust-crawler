use rust_crawler::filters::{FilterSet, PageFilter};
use rust_crawler::models::{PageData, TaskStatus};

fn sample_page() -> PageData {
    PageData {
        url: "https://example.com/rust".to_string(),
        depth: 0,
        title: Some("Rust Home".to_string()),
        status_code: Some(200),
        links: vec![],
        content_length: 512,
        status: TaskStatus::Success,
        error_message: None,
    }
}

#[test]
fn validates_filter_keywords() {
    let filter = FilterSet {
        title_keyword: Some("   ".to_string()),
        ..FilterSet::default()
    };

    assert!(filter.validate().is_err());
}

#[test]
fn keeps_matching_page() {
    let filter = FilterSet {
        title_keyword: Some("rust".to_string()),
        url_keyword: Some("example".to_string()),
        min_content_length: Some(100),
        success_only: true,
    };

    assert!(filter.allows(&sample_page()));
}

#[test]
fn rejects_non_matching_page() {
    let filter = FilterSet {
        min_content_length: Some(1024),
        ..FilterSet::default()
    };

    assert!(!filter.allows(&sample_page()));
}
