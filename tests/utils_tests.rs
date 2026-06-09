use rust_crawler::utils::{is_same_domain, normalize_url, should_visit};

#[test]
fn normalizes_relative_url() {
    let result = normalize_url("https://example.com/docs/index.html", "../about").unwrap();
    assert_eq!(result, "https://example.com/about");
}

#[test]
fn checks_same_domain() {
    let same = is_same_domain("https://example.com/a", "https://example.com/b").unwrap();
    let different = is_same_domain("https://example.com", "https://rust-lang.org").unwrap();

    assert!(same);
    assert!(!different);
}

#[test]
fn respects_same_domain_rule() {
    let allowed = should_visit("https://example.com", "https://example.com/about", true).unwrap();
    let blocked = should_visit("https://example.com", "https://rust-lang.org", true).unwrap();

    assert!(allowed);
    assert!(!blocked);
}
