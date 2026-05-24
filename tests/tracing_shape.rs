//! Tracing-shape integration test (NFR-008).
//!
//! When compiled with `--features tracing`, the engine emits spans
//! at every hot entry point. This test installs a buffer-backed
//! `tracing_subscriber::fmt` layer, exercises each entry, and
//! verifies the captured output names the expected spans + carries
//! the spec-required fields (NFR-008-AC-1).
//!
//! When compiled without the feature, the test confirms (a) the
//! crate still builds, (b) `make test` still has at least one test
//! function in this file to report on (NFR-008-AC-3 zero-cost claim).

#[cfg(feature = "tracing")]
mod with_tracing {
    use std::io;
    use std::path::Path;
    use std::sync::{Arc, Mutex};

    use quire_rs::{
        apply_patch, extract, extract_frontmatter, harvest_edges, parse_document, render_by_name,
        ExtractionDsl, IdentityResolver, Registry,
    };
    use serde_json::json;
    use tracing_subscriber::{fmt, fmt::format::FmtSpan, prelude::*, EnvFilter};

    /// `MakeWriter` that funnels every line into a shared
    /// `Mutex<Vec<u8>>`. The test reads it after exercising the
    /// engine and looks for span names + field values.
    #[derive(Clone, Default)]
    struct BufWriter(Arc<Mutex<Vec<u8>>>);

    impl io::Write for BufWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'a> fmt::MakeWriter<'a> for BufWriter {
        type Writer = BufWriter;
        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    fn install_subscriber() -> Arc<Mutex<Vec<u8>>> {
        let buf = Arc::new(Mutex::new(Vec::new()));
        let writer = BufWriter(buf.clone());
        // Default-level DEBUG so the debug_span! calls fire.
        let filter = EnvFilter::new("quire_rs=debug");
        let layer = fmt::layer()
            .with_writer(writer)
            .with_ansi(false)
            .with_target(true)
            // Emit a log line on span enter (and close) so the
            // captured buffer carries the span name + its fields.
            // Without this, `debug_span!(...).entered()` produces no
            // visible output unless an `event!` fires inside.
            .with_span_events(FmtSpan::NEW);
        let _ = tracing_subscriber::registry()
            .with(filter)
            .with(layer)
            .try_init();
        buf
    }

    fn captured(buf: &Arc<Mutex<Vec<u8>>>) -> String {
        String::from_utf8(buf.lock().unwrap().clone()).unwrap_or_default()
    }

    #[test]
    fn tracing_spans_carry_expected_names_and_fields() {
        let buf = install_subscriber();

        // Exercise every instrumented entry point.
        let modules_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("render_parity")
            .join("modules");
        let r = Registry::load_from(&[modules_root.as_path()]).expect("load");
        let _ = parse_document("---\nid: x\n---\n## A\nbody\n");
        let _ = extract_frontmatter("---\nid: x\n---\nbody");
        let arch = r.archetype("demo-item").expect("demo-item");
        let _ = apply_patch(arch, &json!({"id": "DEMO-001"}), &json!({"title": "x"}));
        let _ = render_by_name(&r, "demo-item", &json!({"id": "DEMO-001", "title": "x"}));
        let dsl: ExtractionDsl = serde_yaml::from_str(
            "yield_pattern:\n  match:\n    id:\n      from: frontmatter_field\n      path: [id]\n",
        )
        .unwrap();
        let _ = extract(&parse_document("---\nid: x\n---\n## A\nbody"), &dsl);
        let doc = parse_document("---\nid: FR-001\ndepends_on:\n- FR-002\n---\nbody");
        let _ = harvest_edges(&doc, "ix://o/r/FR-001", None, &IdentityResolver);

        let s = captured(&buf);
        // NFR-008-AC-1: each hot entry emits a named span.
        for name in [
            "quire_rs::load",
            "quire_rs::parse",
            "quire_rs::apply_patch",
            "quire_rs::render",
            "quire_rs::extract",
            "quire_rs::harvest_edges",
        ] {
            assert!(s.contains(name), "missing span {name:?} in:\n{s}");
        }
        // Spec-required fields per NFR-008 — at minimum the
        // load-bearing identifier (archetype name, byte count,
        // source ref) must be in the span fields.
        // fmt layer writes field values without surrounding quotes
        // (e.g. `archetype=demo-item`), so the assertions match that.
        assert!(
            s.contains("archetype=demo-item"),
            "render archetype field missing:\n{s}"
        );
        assert!(
            s.contains("source=ix://o/r/FR-001"),
            "harvest source field missing:\n{s}"
        );
        assert!(s.contains("bytes="), "parse bytes field missing:\n{s}");
        assert!(s.contains("paths=1"), "load paths field missing:\n{s}");
        assert!(
            s.contains("data_bytes="),
            "render data_bytes field missing:\n{s}"
        );
    }
}

#[cfg(not(feature = "tracing"))]
#[test]
fn tracing_disabled_is_zero_cost_at_compile_time() {
    // Engine compiles without the `tracing` crate. This entry exists
    // so `cargo test` has at least one tracing_shape test to report
    // on regardless of the feature flag (NFR-008-AC-3).
}
