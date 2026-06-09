use rust_crawler::cli::Cli;
use rust_crawler::config::CrawlerConfig;
use rust_crawler::models::OutputFormat;
use std::path::PathBuf;

fn sample_cli() -> Cli {
    Cli {
        start_urls: vec!["https://example.com".to_string()],
        max_depth: 2,
        max_pages: 10,
        worker_count: 4,
        max_retries: 2,
        output: PathBuf::from("output/result.json"),
        format: None,
        report: true,
        report_output: Some(PathBuf::from("output/report.txt")),
        title_keyword: Some("rust".to_string()),
        url_keyword: None,
        min_content_length: Some(100),
        success_only: true,
        same_domain_only: true,
        timeout_secs: 10,
    }
}

#[test]
fn infers_json_output_format_from_path() {
    let config = CrawlerConfig::try_from(sample_cli()).unwrap();
    assert_eq!(config.output_format, OutputFormat::Json);
    assert!(config.report_enabled);
    assert_eq!(config.max_retries, 2);
    assert_eq!(config.filters.title_keyword.as_deref(), Some("rust"));
}

#[test]
fn rejects_unsupported_output_extension() {
    let mut cli = sample_cli();
    cli.output = PathBuf::from("output/result.txt");

    let err = CrawlerConfig::try_from(cli).unwrap_err();
    assert!(
        err.to_string()
            .contains("output format must be json or csv")
    );
}

#[test]
fn accepts_explicit_csv_format() {
    let mut cli = sample_cli();
    cli.output = PathBuf::from("output/custom.data");
    cli.format = Some(rust_crawler::cli::OutputFormatArg::Csv);

    let config = CrawlerConfig::try_from(cli).unwrap();
    assert_eq!(config.output_format, OutputFormat::Csv);
}
