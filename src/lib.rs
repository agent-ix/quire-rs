//! High-performance Rust templating + parsing engine for the Filament/Quire ecosystem.

/// Placeholder entry point.
pub fn hello() -> &'static str {
    "hello from quire_rs"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_returns_greeting() {
        assert!(hello().contains("quire_rs"));
    }
}
