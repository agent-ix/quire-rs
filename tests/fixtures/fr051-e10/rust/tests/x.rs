use ix_trace_rs::trace;

#[trace("FR-051-AC-32")]
#[test]
fn top_level() {
    assert_eq!(2 + 2, 4);
}

mod inner {
    use super::trace;

    #[trace("FR-051-AC-32", "FR-051-AC-33")]
    #[test]
    fn nested() {
        if std::env::var_os("IX_TRACE_E2E_FAIL").is_some() {
            panic!("intentional E-10 failure");
        }
        assert!(true);
    }
}

#[trace("FR-051-AC-32")]
#[rstest::rstest]
fn param(#[values(1)] value: u8) {
    assert!(value > 0);
}
