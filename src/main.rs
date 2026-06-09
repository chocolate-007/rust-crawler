use rust_crawler::cli::Cli;
use rust_crawler::config::CrawlerConfig;
use rust_crawler::crawler::Crawler;
use rust_crawler::error::AppError;
use rust_crawler::models::OutputFormat;
use rust_crawler::report::{build_report, render_report};
use rust_crawler::storage::{CsvStorage, JsonStorage, Storage};
use std::fs;

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), AppError> {
    let cli = Cli::parse();
    let config = CrawlerConfig::try_from(cli)?;

    let crawler = Crawler::new(config.clone());
    let mut result = crawler.run()?;

    if !config.filters.eq(&Default::default()) {
        result.pages = config
            .filters
            .apply(&result.pages)
            .into_iter()
            .cloned()
            .collect();
    }

    match config.output_format {
        OutputFormat::Json => JsonStorage.save(&config.output_path, &result)?,
        OutputFormat::Csv => CsvStorage.save(&config.output_path, &result)?,
    }

    if config.report_enabled {
        let report = render_report(&build_report(&result));
        println!("\n{report}");

        if let Some(path) = &config.report_output_path {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, report)?;
        }
    }

    println!(
        "Crawl finished. visited={}, success={}, failed={}, discovered_links={}, skipped_links={}, retries={}, saved={}",
        result.stats.visited_pages,
        result.stats.success_pages,
        result.stats.failed_pages,
        result.stats.discovered_links,
        result.stats.skipped_links,
        result.stats.retried_requests,
        config.output_path.display()
    );

    Ok(())
}
