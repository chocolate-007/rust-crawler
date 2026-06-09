use rust_crawler::parser::{HtmlParser, LinkExtractor};

#[test]
fn extracts_title_and_links() {
    let html = r#"
    <html>
      <head><title>Example Page</title></head>
      <body>
        <a href="/about">About</a>
        <a href="https://example.com/contact">Contact</a>
      </body>
    </html>
    "#;

    let parser = HtmlParser;
    let title = parser.extract_title(html);
    let links = parser.extract_links(html).unwrap();

    assert_eq!(title.as_deref(), Some("Example Page"));
    assert_eq!(links.len(), 2);
}
