use crate::error::AppError;
use url::Url;

pub fn normalize_url(base: &str, candidate: &str) -> Result<String, AppError> {
    let base_url = Url::parse(base)?;
    let joined = base_url.join(candidate)?;
    Ok(joined.to_string())
}

pub fn is_same_domain(base: &str, candidate: &str) -> Result<bool, AppError> {
    let base_url = Url::parse(base)?;
    let candidate_url = Url::parse(candidate)?;
    Ok(base_url.domain() == candidate_url.domain())
}

pub fn should_visit(
    base_url: &str,
    candidate_url: &str,
    same_domain_only: bool,
) -> Result<bool, AppError> {
    if !is_supported_scheme(candidate_url) {
        return Ok(false);
    }

    if !same_domain_only {
        return Ok(true);
    }

    is_same_domain(base_url, candidate_url)
}

pub fn is_supported_scheme(url: &str) -> bool {
    matches!(Url::parse(url), Ok(parsed) if matches!(parsed.scheme(), "http" | "https"))
}
