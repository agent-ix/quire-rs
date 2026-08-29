//! Baseline fixture source tree.

#[cfg(test)]
mod tests {
    #[trace("TC-001")]
    // malformed marker names TC-404
    #[test]
    fn covers_the_round_trip() {
        let _ = 1;
    }

    #[trace("TC-999")]
    #[test]
    fn covers_nothing_declared() {
        let _ = 1;
    }
}
