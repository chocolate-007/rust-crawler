use crate::error::AppError;
use reqwest::blocking::Client;
use reqwest::header::USER_AGENT;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct FetchResponse {
    pub status_code: u16,
    pub body: String,
}

pub trait PageFetcher {
    fn fetch(&self, url: &str) -> Result<FetchResponse, AppError>;
}

#[derive(Debug, Clone)]
pub struct Fetcher {
    client: Client,
}

impl Fetcher {
    pub fn new(timeout_secs: u64) -> Result<Self, AppError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()?;

        Ok(Self { client })
    }
}

impl PageFetcher for Fetcher {
    fn fetch(&self, url: &str) -> Result<FetchResponse, AppError> {
        let response = self
            .client
            .get(url)
            .header(USER_AGENT, "rust-crawler-course-project/0.1")
            .send()?;

        let status_code = response.status().as_u16();
        let body = response.text()?;

        Ok(FetchResponse { status_code, body })
    }
}
