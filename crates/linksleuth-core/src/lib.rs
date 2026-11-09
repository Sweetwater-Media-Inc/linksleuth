pub mod analysis;
pub mod consumer;
pub mod decoder;
pub mod destinations;
pub mod encodings;
pub mod normalization;
pub mod parser;
pub mod reporting;

/// Returns the current LinkSleuth engine version.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_not_empty() {
        assert!(!version().is_empty());
    }
}
