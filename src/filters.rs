use crate::error::AppError;
use crate::models::PageData;

pub trait PageFilter {
    fn allows(&self, page: &PageData) -> bool;
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilterSet {
    pub title_keyword: Option<String>,
    pub url_keyword: Option<String>,
    pub min_content_length: Option<usize>,
    pub success_only: bool,
}

impl FilterSet {
    pub fn validate(&self) -> Result<(), AppError> {
        if let Some(keyword) = &self.title_keyword
            && keyword.trim().is_empty()
        {
            return Err(AppError::Config(
                "title keyword filter cannot be empty".to_string(),
            ));
        }

        if let Some(keyword) = &self.url_keyword
            && keyword.trim().is_empty()
        {
            return Err(AppError::Config(
                "url keyword filter cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    pub fn apply<'a>(&self, pages: &'a [PageData]) -> Vec<&'a PageData> {
        pages.iter().filter(|page| self.allows(page)).collect()
    }
}

impl PageFilter for FilterSet {
    fn allows(&self, page: &PageData) -> bool {
        if self.success_only && page.status != crate::models::TaskStatus::Success {
            return false;
        }

        if let Some(keyword) = &self.title_keyword {
            let title = page.title.as_deref().unwrap_or_default().to_lowercase();
            if !title.contains(&keyword.to_lowercase()) {
                return false;
            }
        }

        if let Some(keyword) = &self.url_keyword
            && !page.url.to_lowercase().contains(&keyword.to_lowercase())
        {
            return false;
        }

        if let Some(min_content_length) = self.min_content_length
            && page.content_length < min_content_length
        {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::{FilterSet, PageFilter};
    use crate::models::{PageData, TaskStatus};

    fn sample_page() -> PageData {
        PageData {
            url: "https://example.com/docs".to_string(),
            depth: 1,
            title: Some("Rust Guide".to_string()),
            status_code: Some(200),
            links: vec![],
            content_length: 256,
            status: TaskStatus::Success,
            error_message: None,
        }
    }

    #[test]
    fn filters_page_by_title_keyword() {
        let filter = FilterSet {
            title_keyword: Some("rust".to_string()),
            ..FilterSet::default()
        };

        assert!(filter.allows(&sample_page()));
    }

    #[test]
    fn rejects_page_when_url_keyword_does_not_match() {
        let filter = FilterSet {
            url_keyword: Some("blog".to_string()),
            ..FilterSet::default()
        };

        assert!(!filter.allows(&sample_page()));
    }

    #[test]
    fn applies_multiple_filters() {
        let filter = FilterSet {
            title_keyword: Some("rust".to_string()),
            min_content_length: Some(100),
            success_only: true,
            ..FilterSet::default()
        };

        let pages = vec![sample_page()];
        let result = filter.apply(&pages);
        assert_eq!(result.len(), 1);
    }
}
