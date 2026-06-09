use crate::models::{CrawlResult, PageData, TaskStatus};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportSummary {
    pub total_pages: usize,
    pub success_pages: usize,
    pub failed_pages: usize,
    pub skipped_pages: usize,
    pub pages_with_title: usize,
    pub average_content_length: usize,
    pub retried_requests: usize,
    pub top_domains: Vec<DomainCount>,
    pub status_breakdown: Vec<StatusCount>,
    pub top_linked_pages: Vec<LinkCount>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainCount {
    pub domain: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusCount {
    pub status: TaskStatus,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkCount {
    pub url: String,
    pub outgoing_links: usize,
}

pub fn build_report(result: &CrawlResult) -> ReportSummary {
    let total_pages = result.pages.len();
    let success_pages = result
        .pages
        .iter()
        .filter(|page| page.status == TaskStatus::Success)
        .count();
    let failed_pages = result
        .pages
        .iter()
        .filter(|page| page.status == TaskStatus::Failed)
        .count();
    let skipped_pages = result
        .pages
        .iter()
        .filter(|page| page.status == TaskStatus::Skipped)
        .count();
    let pages_with_title = result
        .pages
        .iter()
        .filter(|page| page.title.as_deref().is_some_and(|title| !title.is_empty()))
        .count();

    let average_content_length = if total_pages == 0 {
        0
    } else {
        result
            .pages
            .iter()
            .map(|page| page.content_length)
            .sum::<usize>()
            / total_pages
    };

    ReportSummary {
        total_pages,
        success_pages,
        failed_pages,
        skipped_pages,
        pages_with_title,
        average_content_length,
        retried_requests: result.stats.retried_requests,
        top_domains: collect_top_domains(&result.pages, 5),
        status_breakdown: collect_status_breakdown(&result.pages),
        top_linked_pages: collect_top_linked_pages(&result.pages, 5),
    }
}

pub fn render_report(summary: &ReportSummary) -> String {
    let mut output = String::new();

    output.push_str("Crawl Report\n");
    output.push_str("====================\n");
    output.push_str(&format!("Total pages: {}\n", summary.total_pages));
    output.push_str(&format!("Successful pages: {}\n", summary.success_pages));
    output.push_str(&format!("Failed pages: {}\n", summary.failed_pages));
    output.push_str(&format!("Skipped pages: {}\n", summary.skipped_pages));
    output.push_str(&format!("Pages with title: {}\n", summary.pages_with_title));
    output.push_str(&format!(
        "Average content length: {}\n",
        summary.average_content_length
    ));
    output.push_str(&format!("Retried requests: {}\n", summary.retried_requests));

    output.push_str("\nStatus breakdown:\n");
    for entry in &summary.status_breakdown {
        output.push_str(&format!("- {:?}: {}\n", entry.status, entry.count));
    }

    output.push_str("\nTop domains:\n");
    for entry in &summary.top_domains {
        output.push_str(&format!("- {} ({})\n", entry.domain, entry.count));
    }

    output.push_str("\nTop linked pages:\n");
    for entry in &summary.top_linked_pages {
        output.push_str(&format!("- {} ({})\n", entry.url, entry.outgoing_links));
    }

    output
}

fn collect_top_domains(pages: &[PageData], limit: usize) -> Vec<DomainCount> {
    let mut counts = BTreeMap::<String, usize>::new();

    for page in pages {
        if let Some(domain) = extract_domain(&page.url) {
            *counts.entry(domain).or_default() += 1;
        }
    }

    let mut entries = counts
        .into_iter()
        .map(|(domain, count)| DomainCount { domain, count })
        .collect::<Vec<_>>();

    entries.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.domain.cmp(&right.domain))
    });
    entries.truncate(limit);
    entries
}

fn collect_status_breakdown(pages: &[PageData]) -> Vec<StatusCount> {
    let mut counts = BTreeMap::<TaskStatus, usize>::new();

    for page in pages {
        *counts.entry(page.status.clone()).or_default() += 1;
    }

    counts
        .into_iter()
        .map(|(status, count)| StatusCount { status, count })
        .collect()
}

fn collect_top_linked_pages(pages: &[PageData], limit: usize) -> Vec<LinkCount> {
    let mut entries = pages
        .iter()
        .map(|page| LinkCount {
            url: page.url.clone(),
            outgoing_links: page.links.len(),
        })
        .collect::<Vec<_>>();

    entries.sort_by(|left, right| {
        right
            .outgoing_links
            .cmp(&left.outgoing_links)
            .then_with(|| left.url.cmp(&right.url))
    });
    entries.truncate(limit);
    entries
}

fn extract_domain(url: &str) -> Option<String> {
    url::Url::parse(url)
        .ok()
        .and_then(|parsed| parsed.domain().map(ToString::to_string))
}

#[cfg(test)]
mod tests {
    use super::{build_report, render_report};
    use crate::models::{CrawlResult, CrawlStats, PageData, TaskStatus};

    #[test]
    fn builds_report_summary() {
        let result = CrawlResult {
            pages: vec![
                PageData {
                    url: "https://example.com".to_string(),
                    depth: 0,
                    title: Some("Home".to_string()),
                    status_code: Some(200),
                    links: vec!["https://example.com/a".to_string()],
                    content_length: 100,
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
            stats: CrawlStats::default(),
        };

        let summary = build_report(&result);
        assert_eq!(summary.total_pages, 2);
        assert_eq!(summary.success_pages, 1);
        assert_eq!(summary.failed_pages, 1);
        assert_eq!(summary.pages_with_title, 1);
        assert_eq!(summary.retried_requests, 0);
        assert_eq!(summary.top_domains[0].domain, "example.com");
    }

    #[test]
    fn renders_human_readable_report() {
        let result = CrawlResult {
            pages: vec![PageData {
                url: "https://example.com".to_string(),
                depth: 0,
                title: Some("Home".to_string()),
                status_code: Some(200),
                links: vec!["https://example.com/a".to_string()],
                content_length: 100,
                status: TaskStatus::Success,
                error_message: None,
            }],
            stats: CrawlStats::default(),
        };

        let report = render_report(&build_report(&result));
        assert!(report.contains("Crawl Report"));
        assert!(report.contains("example.com"));
    }
}
