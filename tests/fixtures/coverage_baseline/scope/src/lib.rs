//! Baseline fixture source tree.
//!
//! CR-187, PLAT-1077: `covers_ac1`/`covers_ac3_ignored`/`covers_range` below
//! exercise the `coverage_matrix` surface (FR-003 in this fixture — see
//! `spec/FR-003.md`) and the `range-in-trace-tag` finding. Deliberately no
//! explanatory comment sits in any of their own leading annotation blocks:
//! this file is scanned by the generic near-miss detector too, and prose
//! naming a sibling AC id in a bound symbol's own span would land it in
//! `unmatched_tags` as a false near-miss.

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

    #[trace("FR-003-AC-1")]
    #[test]
    fn covers_ac1() {
        let _ = 1;
    }

    #[trace("FR-003-AC-3")]
    #[ignore]
    #[test]
    fn covers_ac3_ignored() {
        let _ = 1;
    }

    #[trace("FR-003-AC-1..FR-003-AC-2")]
    #[test]
    fn covers_range() {
        let _ = 1;
    }
}
