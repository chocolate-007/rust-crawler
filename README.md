# RustCrawler

基于 Rust 编写的多线程网页爬虫课程项目，支持并发抓取、链接去重、结果导出、过滤分析和文本报告输出。

## 项目简介

本项目面向 Rust 要求设计，重点体现以下能力：

- Rust 基础语法与类型系统
- 所有权、借用与共享状态管理
- 模块化设计
- `Result` 风格错误处理
- 多线程并发
- 测试、文档与工程规范

程序从一个或多个起始 URL 出发，递归抓取网页，提取页面标题和链接，限制抓取深度与页面数量，并将结果导出为 JSON 或 CSV。程序还支持根据标题、URL、内容长度等条件过滤页面，并输出文本统计报告。

## 已实现功能

- 支持多个起始 URL
- 支持最大深度、最大页面数、线程数、超时设置
- 支持请求失败后的有限重试
- 支持同域名限制
- 支持多线程任务调度
- 支持链接去重与任务边界控制
- 支持抓取结果保存为 JSON / CSV
- 支持标题关键词、URL 关键词、最小内容长度过滤
- 支持生成终端报告和文本报告文件
- 包含单元测试与关键功能测试

## 技术选型

- `clap`：命令行参数解析
- `reqwest`：HTTP 请求
- `scraper`：HTML 解析
- `serde` / `serde_json`：结果序列化
- `csv`：CSV 导出
- `thiserror`：统一错误处理
- `url`：URL 解析与规范化

## 项目结构

```text
src/
├─ main.rs
├─ lib.rs
├─ cli.rs
├─ config.rs
├─ crawler.rs
├─ fetcher.rs
├─ filters.rs
├─ parser.rs
├─ report.rs
├─ storage.rs
├─ models.rs
├─ error.rs
└─ utils.rs
tests/
├─ config_tests.rs
├─ filters_tests.rs
├─ parser_tests.rs
├─ report_tests.rs
├─ storage_tests.rs
└─ utils_tests.rs
```

## 编译与运行

### 1. 编译

```bash
cargo build
```

### 2. 基本运行

```bash
cargo run -- \
  --start-urls https://example.com \
  --max-depth 1 \
  --max-pages 10 \
  --worker-count 4 \
  --output output/result.json
```

### 3. 生成报告

```bash
cargo run -- \
  --start-urls https://example.com \
  --max-depth 1 \
  --max-pages 10 \
  --worker-count 4 \
  --output output/result.json \
  --report \
  --report-output output/report.txt
```

### 4. 使用过滤条件

```bash
cargo run -- \
  --start-urls https://example.com \
  --output output/result.csv \
  --format csv \
  --title-keyword rust \
  --min-content-length 100 \
  --success-only
```

## 命令行参数说明

- `--start-urls`：起始 URL，支持多个，使用逗号分隔
- `--max-depth`：最大爬取深度
- `--max-pages`：最大抓取页面数
- `--worker-count`：工作线程数
- `--max-retries`：单页面最大重试次数
- `--output`：结果输出路径
- `--format`：输出格式，可选 `json` 或 `csv`
- `--same-domain-only`：是否仅抓取同域名页面
- `--timeout-secs`：单个请求超时时间
- `--report`：是否输出分析报告
- `--report-output`：报告文件输出路径
- `--title-keyword`：按标题关键词过滤
- `--url-keyword`：按 URL 关键词过滤
- `--min-content-length`：按最小内容长度过滤
- `--success-only`：仅保留抓取成功页面

## 输出结果说明

### JSON 输出

JSON 输出包含两部分：

- `pages`：每个页面的抓取结果
- `stats`：本次抓取的统计信息

页面结果中包含：

- 页面 URL
- 抓取深度
- 标题
- 状态码
- 子链接列表
- 页面内容长度
- 抓取状态
- 错误信息

### CSV 输出

CSV 输出主要用于快速查看页面摘要，包括：

- URL
- 标题
- 状态码
- 出链列表
- 页面内容长度

## 测试与规范

提交前已执行：

```bash
cargo fmt --all
cargo clippy -- -D warnings
cargo test
```

## 课程要求对应说明

- 模块化设计：已拆分为多个 `src` 模块
- 错误处理：统一使用 `Result` 和 `AppError`
- Rust 核心特性：包含 `struct`、`enum`、`trait`、泛型、借用与共享所有权
- 并发：使用 `thread`、`Arc`、`Mutex`、`Condvar`
- 测试：包含单元测试和关键流程测试
- 文档：提供完整 README 与规划文档


## 后续优化方向

- 增加重试机制
- 增加日志分级
- 增加本地 HTML 样本驱动的集成演示
- 增加断点续爬或异步版本对比
