use crate::config::CrawlerConfig;
use crate::error::AppError;
use crate::fetcher::{Fetcher, PageFetcher};
use crate::models::{CrawlResult, CrawlStats, CrawlTask, PageData, TaskStatus};
use crate::parser::{HtmlParser, LinkExtractor};
use crate::utils::{normalize_url, should_visit};
use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

#[derive(Debug, Clone)]
pub struct Crawler {
    config: CrawlerConfig,
}

#[derive(Debug)]
struct SchedulerState {
    queue: VecDeque<CrawlTask>,
    seen: HashSet<String>,
    active_workers: usize,
    shutdown: bool,
}

impl Crawler {
    pub fn new(config: CrawlerConfig) -> Self {
        Self { config }
    }

    pub fn run(&self) -> Result<CrawlResult, AppError> {
        let fetcher = Fetcher::new(self.config.timeout_secs)?;
        self.run_with(fetcher, HtmlParser)
    }

    pub fn run_with<F, P>(&self, fetcher: F, parser: P) -> Result<CrawlResult, AppError>
    where
        F: PageFetcher + Clone + Send + Sync + 'static,
        P: LinkExtractor + Clone + Send + Sync + 'static,
    {
        let scheduler = Arc::new((
            Mutex::new(initialize_scheduler(&self.config)?),
            Condvar::new(),
        ));
        let pages = Arc::new(Mutex::new(Vec::<PageData>::new()));
        let stats = Arc::new(Mutex::new(CrawlStats::default()));

        let mut handles = Vec::with_capacity(self.config.worker_count);

        for _ in 0..self.config.worker_count {
            let scheduler = Arc::clone(&scheduler);
            let pages = Arc::clone(&pages);
            let stats = Arc::clone(&stats);
            let config = self.config.clone();
            let fetcher = fetcher.clone();
            let parser = parser.clone();

            let handle = thread::spawn(move || {
                worker_loop(config, scheduler, pages, stats, fetcher, parser)
            });
            handles.push(handle);
        }

        for handle in handles {
            match handle.join() {
                Ok(Ok(())) => {}
                Ok(Err(err)) => return Err(err),
                Err(_) => return Err(AppError::Internal("worker thread panicked".to_string())),
            }
        }

        let mut result_pages = lock_mutex(&pages, "pages")?.clone();
        result_pages.sort_by(|left, right| left.url.cmp(&right.url));

        Ok(CrawlResult {
            pages: result_pages,
            stats: lock_mutex(&stats, "stats")?.clone(),
        })
    }
}

fn initialize_scheduler(config: &CrawlerConfig) -> Result<SchedulerState, AppError> {
    let mut queue = VecDeque::new();
    let mut seen = HashSet::new();

    for url in &config.start_urls {
        if !seen.insert(url.clone()) {
            continue;
        }

        queue.push_back(CrawlTask {
            url: url.clone(),
            depth: 0,
        });
    }

    if queue.is_empty() {
        return Err(AppError::Config(
            "start_urls did not contain any unique values".to_string(),
        ));
    }

    Ok(SchedulerState {
        queue,
        seen,
        active_workers: 0,
        shutdown: false,
    })
}

fn worker_loop<F, P>(
    config: CrawlerConfig,
    scheduler: Arc<(Mutex<SchedulerState>, Condvar)>,
    pages: Arc<Mutex<Vec<PageData>>>,
    stats: Arc<Mutex<CrawlStats>>,
    fetcher: F,
    parser: P,
) -> Result<(), AppError>
where
    F: PageFetcher,
    P: LinkExtractor,
{
    loop {
        let task = match take_next_task(&scheduler)? {
            Some(task) => task,
            None => return Ok(()),
        };

        let page = process_task(&task, &fetcher, &parser, &config, &scheduler, &stats)?;

        {
            let mut pages_guard = lock_mutex(&pages, "pages")?;
            pages_guard.push(page);
        }

        finish_task(&scheduler)?;
    }
}

fn take_next_task(
    scheduler: &Arc<(Mutex<SchedulerState>, Condvar)>,
) -> Result<Option<CrawlTask>, AppError> {
    let (state_mutex, state_cv) = &**scheduler;
    let mut state = lock_mutex(state_mutex, "scheduler")?;

    loop {
        if state.shutdown {
            return Ok(None);
        }

        if let Some(task) = state.queue.pop_front() {
            state.active_workers += 1;
            return Ok(Some(task));
        }

        if state.active_workers == 0 {
            state.shutdown = true;
            state_cv.notify_all();
            return Ok(None);
        }

        state = state_cv
            .wait(state)
            .map_err(|_| AppError::Internal("scheduler wait poisoned".to_string()))?;
    }
}

fn process_task<F, P>(
    task: &CrawlTask,
    fetcher: &F,
    parser: &P,
    config: &CrawlerConfig,
    scheduler: &Arc<(Mutex<SchedulerState>, Condvar)>,
    stats: &Arc<Mutex<CrawlStats>>,
) -> Result<PageData, AppError>
where
    F: PageFetcher,
    P: LinkExtractor,
{
    increment_visited(stats)?;

    let response = match fetch_with_retries(fetcher, &task.url, config.max_retries, stats) {
        Ok(response) => response,
        Err(err) => {
            let mut stats_guard = lock_mutex(stats, "stats")?;
            stats_guard.failed_pages += 1;

            return Ok(PageData {
                url: task.url.clone(),
                depth: task.depth,
                title: None,
                status_code: None,
                links: Vec::new(),
                content_length: 0,
                status: TaskStatus::Failed,
                error_message: Some(err.to_string()),
            });
        }
    };

    let title = parser.extract_title(&response.body);
    let raw_links = parser.extract_links(&response.body)?;
    let normalized_links = collect_next_links(&task.url, &raw_links, config.same_domain_only)?;

    let accepted_links = enqueue_links(task.depth, &normalized_links, config, scheduler, stats)?;

    {
        let mut stats_guard = lock_mutex(stats, "stats")?;
        stats_guard.success_pages += 1;
        stats_guard.discovered_links += accepted_links.len();
    }

    Ok(PageData {
        url: task.url.clone(),
        depth: task.depth,
        title,
        status_code: Some(response.status_code),
        links: accepted_links,
        content_length: response.body.len(),
        status: TaskStatus::Success,
        error_message: None,
    })
}

fn fetch_with_retries<F>(
    fetcher: &F,
    url: &str,
    max_retries: usize,
    stats: &Arc<Mutex<CrawlStats>>,
) -> Result<crate::fetcher::FetchResponse, AppError>
where
    F: PageFetcher,
{
    let mut last_error = match fetcher.fetch(url) {
        Ok(response) => return Ok(response),
        Err(err) => err,
    };

    for _ in 0..max_retries {
        {
            let mut stats_guard = lock_mutex(stats, "stats")?;
            stats_guard.retried_requests += 1;
        }

        match fetcher.fetch(url) {
            Ok(response) => return Ok(response),
            Err(err) => last_error = err,
        }
    }

    Err(last_error)
}

fn enqueue_links(
    current_depth: usize,
    links: &[String],
    config: &CrawlerConfig,
    scheduler: &Arc<(Mutex<SchedulerState>, Condvar)>,
    stats: &Arc<Mutex<CrawlStats>>,
) -> Result<Vec<String>, AppError> {
    if current_depth >= config.max_depth {
        return Ok(Vec::new());
    }

    let (state_mutex, state_cv) = &**scheduler;
    let mut accepted = Vec::new();
    let mut skipped = 0usize;

    {
        let mut state = lock_mutex(state_mutex, "scheduler")?;
        for link in links {
            if state.seen.len() >= config.max_pages {
                skipped += 1;
                continue;
            }

            if state.seen.insert(link.clone()) {
                state.queue.push_back(CrawlTask {
                    url: link.clone(),
                    depth: current_depth + 1,
                });
                accepted.push(link.clone());
            } else {
                skipped += 1;
            }
        }
    }

    if !accepted.is_empty() || skipped > 0 {
        let mut stats_guard = lock_mutex(stats, "stats")?;
        stats_guard.skipped_links += skipped;
    }

    state_cv.notify_all();
    Ok(accepted)
}

fn finish_task(scheduler: &Arc<(Mutex<SchedulerState>, Condvar)>) -> Result<(), AppError> {
    let (state_mutex, state_cv) = &**scheduler;
    let mut state = lock_mutex(state_mutex, "scheduler")?;

    if state.active_workers > 0 {
        state.active_workers -= 1;
    }

    if state.queue.is_empty() && state.active_workers == 0 {
        state.shutdown = true;
    }

    state_cv.notify_all();
    Ok(())
}

fn increment_visited(stats: &Arc<Mutex<CrawlStats>>) -> Result<(), AppError> {
    let mut stats_guard = lock_mutex(stats, "stats")?;
    stats_guard.visited_pages += 1;
    Ok(())
}

fn collect_next_links(
    base_url: &str,
    links: &[String],
    same_domain_only: bool,
) -> Result<Vec<String>, AppError> {
    let mut next = Vec::new();

    for link in links {
        if let Ok(normalized) = normalize_url(base_url, link)
            && should_visit(base_url, &normalized, same_domain_only)?
        {
            next.push(normalized);
        }
    }

    Ok(next)
}

fn lock_mutex<'a, T>(
    mutex: &'a Mutex<T>,
    label: &str,
) -> Result<std::sync::MutexGuard<'a, T>, AppError> {
    mutex
        .lock()
        .map_err(|_| AppError::Internal(format!("{label} lock poisoned")))
}

#[cfg(test)]
mod tests {
    use super::Crawler;
    use crate::config::CrawlerConfig;
    use crate::error::AppError;
    use crate::fetcher::{FetchResponse, PageFetcher};
    use crate::filters::FilterSet;
    use crate::models::{OutputFormat, TaskStatus};
    use crate::parser::{HtmlParser, LinkExtractor};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[derive(Clone)]
    struct MockFetcher {
        responses: Arc<HashMap<String, Result<FetchResponse, AppError>>>,
    }

    impl PageFetcher for MockFetcher {
        fn fetch(&self, url: &str) -> Result<FetchResponse, AppError> {
            match self.responses.get(url) {
                Some(Ok(response)) => Ok(response.clone()),
                Some(Err(err)) => Err(AppError::Network(err.to_string())),
                None => Err(AppError::Network(format!("missing mock response: {url}"))),
            }
        }
    }

    #[derive(Clone)]
    struct StaticParser;

    impl LinkExtractor for StaticParser {
        fn extract_title(&self, html: &str) -> Option<String> {
            HtmlParser.extract_title(html)
        }

        fn extract_links(&self, html: &str) -> Result<Vec<String>, AppError> {
            HtmlParser.extract_links(html)
        }
    }

    fn test_config() -> CrawlerConfig {
        CrawlerConfig {
            start_urls: vec!["https://example.com".to_string()],
            max_depth: 2,
            max_pages: 10,
            worker_count: 3,
            max_retries: 1,
            output_path: PathBuf::from("output/test.json"),
            output_format: OutputFormat::Json,
            report_enabled: false,
            report_output_path: None,
            filters: FilterSet::default(),
            same_domain_only: true,
            timeout_secs: 5,
        }
    }

    #[test]
    fn crawls_multiple_pages_without_duplicates() {
        let mut responses = HashMap::new();
        responses.insert(
            "https://example.com".to_string(),
            Ok(FetchResponse {
                status_code: 200,
                body: r#"
                    <html><head><title>Home</title></head><body>
                    <a href="/a">A</a>
                    <a href="/b">B</a>
                    <a href="/a">A2</a>
                    </body></html>
                "#
                .to_string(),
            }),
        );
        responses.insert(
            "https://example.com/a".to_string(),
            Ok(FetchResponse {
                status_code: 200,
                body: r#"
                    <html><head><title>Page A</title></head><body>
                    <a href="/b">B</a>
                    </body></html>
                "#
                .to_string(),
            }),
        );
        responses.insert(
            "https://example.com/b".to_string(),
            Ok(FetchResponse {
                status_code: 200,
                body: r#"<html><head><title>Page B</title></head><body></body></html>"#.to_string(),
            }),
        );

        let crawler = Crawler::new(test_config());
        let result = crawler
            .run_with(
                MockFetcher {
                    responses: Arc::new(responses),
                },
                StaticParser,
            )
            .unwrap();

        assert_eq!(result.stats.visited_pages, 3);
        assert_eq!(result.stats.success_pages, 3);
        assert_eq!(result.stats.failed_pages, 0);
        assert_eq!(result.pages.len(), 3);
        assert!(result.stats.skipped_links >= 1);
    }

    #[test]
    fn records_failed_pages_in_result() {
        let mut responses = HashMap::new();
        responses.insert(
            "https://example.com".to_string(),
            Err(AppError::Network("timeout".to_string())),
        );

        let crawler = Crawler::new(test_config());
        let result = crawler
            .run_with(
                MockFetcher {
                    responses: Arc::new(responses),
                },
                StaticParser,
            )
            .unwrap();

        assert_eq!(result.stats.visited_pages, 1);
        assert_eq!(result.stats.failed_pages, 1);
        assert_eq!(result.pages[0].status, TaskStatus::Failed);
        assert!(result.pages[0].error_message.is_some());
    }

    #[test]
    fn retries_failed_request_once_before_success() {
        #[derive(Clone)]
        struct FlakyFetcher {
            calls: Arc<std::sync::Mutex<usize>>,
        }

        impl PageFetcher for FlakyFetcher {
            fn fetch(&self, _url: &str) -> Result<FetchResponse, AppError> {
                let mut calls = self.calls.lock().unwrap();
                *calls += 1;
                if *calls == 1 {
                    Err(AppError::Network("temporary failure".to_string()))
                } else {
                    Ok(FetchResponse {
                        status_code: 200,
                        body: "<html><head><title>Recovered</title></head><body></body></html>"
                            .to_string(),
                    })
                }
            }
        }

        let crawler = Crawler::new(test_config());
        let result = crawler
            .run_with(
                FlakyFetcher {
                    calls: Arc::new(std::sync::Mutex::new(0)),
                },
                StaticParser,
            )
            .unwrap();

        assert_eq!(result.stats.success_pages, 1);
        assert_eq!(result.stats.failed_pages, 0);
        assert_eq!(result.stats.retried_requests, 1);
    }
}
