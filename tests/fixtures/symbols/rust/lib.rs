//! Rust symbol-extraction fixture (FR-051).

/// A container: parsed configuration.
pub struct Config {
    pub name: String,
}

pub enum Mode {
    Fast,
    Careful,
}

/// A plain function.
pub fn parse_config(text: &str) -> Config {
    Config {
        name: text.trim().to_string(),
    }
}

impl Config {
    pub fn is_named(&self) -> bool {
        !self.name.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Trace: FR-051-AC-1
    #[trace("TC-741")]
    #[test]
    fn tc741_extracts() {
        assert!(parse_config("x").is_named());
    }

    #[tokio::test]
    async fn tc743_async_test() {
        assert_eq!(1, 1);
    }

    fn helper() -> Config {
        parse_config("helper")
    }
}
