// A deliberately unparseable fixture (FR-051-AC-9): the file is truncated
// mid-block, so its braces never balance.
pub fn truncated() {
    if true {
        let x = 1;
