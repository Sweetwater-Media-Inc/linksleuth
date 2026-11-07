use url::{ParseError, Url};

#[derive(Debug, PartialEq)]
pub struct QueryParameter {
    pub name: String,
    pub raw_name: String,
    pub value: String,
    pub raw_value: String,
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
        .query()
        .unwrap_or("")
        .split('&')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let (raw_name, raw_value) = part.split_once('=').unwrap_or((part, ""));

            let decoded_name = url::form_urlencoded::parse(part.as_bytes())
                .next()
                .map(|(name, _)| name.into_owned())
                .unwrap_or_default();

            let decoded_value = url::form_urlencoded::parse(part.as_bytes())
                .next()
                .map(|(_, value)| value.into_owned())
                .unwrap_or_default();

            QueryParameter {
                name: decoded_name,
                raw_name: raw_name.to_string(),
                value: decoded_value,
                raw_value: raw_value.to_string(),
            }
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
    #[test]
    fn preserves_raw_query_values() {
        let result =
            parse_url("https://example.com/?target=https%253A%252F%252Fother.example").unwrap();

        let parameter = &result.query_parameters[0];

        assert_eq!(parameter.name, "target");
        assert_eq!(parameter.raw_value, "https%253A%252F%252Fother.example");
        assert_eq!(parameter.value, "https%3A%2F%2Fother.example");
    }
}
