use crate::cli::{Cli, OutputFormatArg};
use crate::error::AppError;
use crate::filters::FilterSet;
use crate::models::OutputFormat;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CrawlerConfig {
    pub start_urls: Vec<String>,
    pub max_depth: usize,
    pub max_pages: usize,
    pub worker_count: usize,
    pub max_retries: usize,
    pub output_path: PathBuf,
    pub output_format: OutputFormat,
    pub report_enabled: bool,
    pub report_output_path: Option<PathBuf>,
    pub filters: FilterSet,
    pub same_domain_only: bool,
    pub timeout_secs: u64,
}

impl CrawlerConfig {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.start_urls.is_empty() {
            return Err(AppError::Config(
                "at least one start url is required".to_string(),
            ));
        }

        if self.max_pages == 0 {
            return Err(AppError::Config(
                "max_pages must be greater than 0".to_string(),
            ));
        }

        if self.worker_count == 0 {
            return Err(AppError::Config(
                "worker_count must be greater than 0".to_string(),
            ));
        }

        if self.max_retries > 10 {
            return Err(AppError::Config(
                "max_retries should not be greater than 10".to_string(),
            ));
        }

        if self.timeout_secs == 0 {
            return Err(AppError::Config(
                "timeout_secs must be greater than 0".to_string(),
            ));
        }

        if self.max_depth > 32 {
            return Err(AppError::Config(
                "max_depth is too large for a course project demo".to_string(),
            ));
        }

        self.filters.validate()?;

        Ok(())
    }
}

impl TryFrom<Cli> for CrawlerConfig {
    type Error = AppError;

    fn try_from(value: Cli) -> Result<Self, Self::Error> {
        let output_format = match value.format {
            Some(format) => match format {
                OutputFormatArg::Json => OutputFormat::Json,
                OutputFormatArg::Csv => OutputFormat::Csv,
            },
            None => infer_output_format(&value.output)?,
        };

        let filters = FilterSet {
            title_keyword: value.title_keyword,
            url_keyword: value.url_keyword,
            min_content_length: value.min_content_length,
            success_only: value.success_only,
        };

        let config = Self {
            start_urls: value.start_urls,
            max_depth: value.max_depth,
            max_pages: value.max_pages,
            worker_count: value.worker_count,
            max_retries: value.max_retries,
            output_path: value.output,
            output_format,
            report_enabled: value.report,
            report_output_path: value.report_output,
            filters,
            same_domain_only: value.same_domain_only,
            timeout_secs: value.timeout_secs,
        };

        config.validate()?;
        Ok(config)
    }
}

fn infer_output_format(path: &Path) -> Result<OutputFormat, AppError> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| AppError::Config("output file must have an extension".to_string()))?;

    OutputFormat::from_extension(extension).ok_or_else(|| {
        AppError::Config("output format must be json or csv, or pass --format".to_string())
    })
}
