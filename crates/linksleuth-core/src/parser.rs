use url::{ParseError, Url};

#[derive(Debug, PartialEq)]
pub struct QueryParameter {
    pub name: String,
    pub value: String,
}

#[derive(Debug)]
pub struct ParsedUrl {
    pub original: String,
    pub normalized: String,
    pub scheme: String,
    pub hostname: String,
    pub port: Option<u16>,
    pub path: String,
    pub query_parameters: Vec<QueryParameter>,
    pub fragment: Option<String>,
}

#[derive(Debug)]
pub enum UrlError {
    TooLong,
    Invalid(ParseError),
    UnsupportedScheme,
    MissingHostname,
}

pub fn parse_url(input: &str) -> Result<ParsedUrl, UrlError> {
    // Limit input size to 16 KiB.
    if input.len() > 16 * 1024 {
        return Err(UrlError::TooLong);
    }

    let url = Url::parse(input).map_err(UrlError::Invalid)?;

    if !matches!(url.scheme(), "http" | "https") {
        return Err(UrlError::UnsupportedScheme);
    }

    let hostname = url.host_str().ok_or(UrlError::MissingHostname)?.to_string();

    let query_parameters = url
        .query_pairs()
        .map(|(name, value)| QueryParameter {
            name: name.into_owned(),
            value: value.into_owned(),
        })
        .collect();

    Ok(ParsedUrl {
        original: input.to_string(),
        normalized: url.to_string(),
        scheme: url.scheme().to_string(),
        hostname,
        port: url.port(),
        path: url.path().to_string(),
        query_parameters,
        fragment: url.fragment().map(str::to_string),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_https_url() {
        let result = parse_url("https://example.com/test").unwrap();

        assert_eq!(result.scheme, "https");
        assert_eq!(result.hostname, "example.com");
        assert_eq!(result.path, "/test");
    }

    #[test]
    fn decodes_query_parameters() {
        let result = parse_url("https://example.com/?target=https%3A%2F%2Fother.example").unwrap();

        assert_eq!(result.query_parameters[0].value, "https://other.example");
    }

    #[test]
    fn rejects_unsupported_schemes() {
        let result = parse_url("file:///etc/passwd");

        assert!(matches!(result, Err(UrlError::UnsupportedScheme)));
    }

    #[test]
    fn rejects_oversized_input() {
        let input = format!("https://example.com/{}", "a".repeat(20_000));

        assert!(matches!(parse_url(&input), Err(UrlError::TooLong)));
    }
}
