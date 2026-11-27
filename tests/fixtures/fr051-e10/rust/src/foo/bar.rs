#[cfg(test)]
mod tests {
    use ix_trace_rs::trace;

    #[trace("FR-051-AC-32")]
    #[test]
    fn t() {
        assert_eq!(3, 3);
    }
}
