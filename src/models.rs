use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrawlTask {
    pub url: String,
    pub depth: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PageData {
    pub url: String,
    pub depth: usize,
    pub title: Option<String>,
    pub status_code: Option<u16>,
    pub links: Vec<String>,
    pub content_length: usize,
    pub status: TaskStatus,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrawlStats {
    pub visited_pages: usize,
    pub success_pages: usize,
    pub failed_pages: usize,
    pub discovered_links: usize,
    pub skipped_links: usize,
    pub retried_requests: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Csv,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskStatus {
    Success,
    Failed,
    Skipped,
}

impl OutputFormat {
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension {
            "json" => Some(Self::Json),
            "csv" => Some(Self::Csv),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrawlResult {
    pub pages: Vec<PageData>,
    pub stats: CrawlStats,
}
