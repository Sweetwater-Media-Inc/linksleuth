use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedDestination {
    pub url: String,
    pub hostname: String,
    pub source_parameter: String,
    pub decoding_passes: usize,
}

pub fn discover_destination(
    value: &str,
    parameter_name: &str,
    decoding_passes: usize,
) -> Option<EmbeddedDestination> {
    let parsed = Url::parse(value).ok()?;

    if !matches!(parsed.scheme(), "http" | "https") {
        return None;
    }

    let hostname = parsed.host_str()?.to_string();

    Some(EmbeddedDestination {
        url: parsed.to_string(),
        hostname,
        source_parameter: parameter_name.to_string(),
        decoding_passes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_https_destination() {
        let result = discover_destination("https://other.example/path", "target", 2).unwrap();

        assert_eq!(result.hostname, "other.example");
        assert_eq!(result.decoding_passes, 2);
    }

    #[test]
    fn ignores_plain_text() {
        assert!(discover_destination("hello world", "message", 0).is_none());
    }

    #[test]
    fn ignores_unsupported_schemes() {
        assert!(discover_destination("javascript:alert(1)", "target", 0).is_none());
    }

    #[test]
    fn ignores_relative_paths() {
        assert!(discover_destination("/some/local/path", "target", 0).is_none());
    }
}
