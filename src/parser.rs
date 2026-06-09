use crate::error::AppError;
use scraper::{Html, Selector};

pub trait LinkExtractor {
    fn extract_title(&self, html: &str) -> Option<String>;
    fn extract_links(&self, html: &str) -> Result<Vec<String>, AppError>;
}

#[derive(Debug, Default, Clone)]
pub struct HtmlParser;

impl LinkExtractor for HtmlParser {
    fn extract_title(&self, html: &str) -> Option<String> {
        let document = Html::parse_document(html);
        let selector = Selector::parse("title").ok()?;
        document
            .select(&selector)
            .next()
            .map(|node| node.text().collect::<String>().trim().to_string())
            .filter(|title| !title.is_empty())
    }

    fn extract_links(&self, html: &str) -> Result<Vec<String>, AppError> {
        let document = Html::parse_document(html);
        let selector = Selector::parse("a")
            .map_err(|err| AppError::Parse(format!("failed to parse selector: {err}")))?;

        let mut links = Vec::new();
        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                let trimmed = href.trim();
                if !trimmed.is_empty() {
                    links.push(trimmed.to_string());
                }
            }
        }

        Ok(links)
    }
}
