use crate::models::CrawlResult;
use std::fs::{self, File};
use std::path::Path;

use crate::error::AppError;

#[derive(serde::Serialize)]
struct CsvPageRecord<'a> {
    url: &'a str,
    title: &'a Option<String>,
    status_code: u16,
    links: String,
    content_length: usize,
}

pub trait Storage {
    fn save(&self, path: &Path, result: &CrawlResult) -> Result<(), AppError>;
}

#[derive(Debug, Default, Clone)]
pub struct JsonStorage;

impl Storage for JsonStorage {
    fn save(&self, path: &Path, result: &CrawlResult) -> Result<(), AppError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, result)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct CsvStorage;

impl Storage for CsvStorage {
    fn save(&self, path: &Path, result: &CrawlResult) -> Result<(), AppError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = File::create(path)?;
        let mut writer = csv::Writer::from_writer(file);
        for page in &result.pages {
            let record = CsvPageRecord {
                url: &page.url,
                title: &page.title,
                status_code: page.status_code.unwrap_or_default(),
                links: page.links.join(" | "),
                content_length: page.content_length,
            };
            writer.serialize(record)?;
        }
        writer.flush()?;
        Ok(())
    }
}
