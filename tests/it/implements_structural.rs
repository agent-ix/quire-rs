//! FR-062 (PLAT-866) — Python and TypeScript structural `implements` syntax
//! positions, and FR-051-AC-51's wrapper-chain symbol (TC-2200..TC-2210).
//!
//! Every test goes through the public path: a module manifest is loaded, the
//! source tree is extracted and bound, and the coverage report carries the
//! findings. Nothing here patches a `TraceabilityModel` by hand.

use ix_trace_rs::trace;
use quire_rs::coverage::CoverageReport;
use quire_rs::symbols::trace as bind_trace;
use quire_rs::symbols::trace::SymbolGraph;
use quire_rs::symbols::SymbolExtraction;
use serde_json::{json, Value};
use std::collections::BTreeSet;

const IDS: &str = r#"((?:\s*"[^"]*"\s*,?)+)\s*"#;

fn py_form() -> Value {
    json!({
        "name": "py-implements", "language": "python", "syntax": "python_decorator",
        "pattern": format!(r"\A@(?:ix_trace_py\.)?implements\({IDS}\)"),
        "template": "@implements({ids})",
    })
}

fn ts_decorator_form() -> Value {
    json!({
        "name": "ts-implementing-decorator", "language": "typescript",
        "syntax": "typescript_decorator",
        "pattern": format!(r"\A@(?:ixTrace\.)?implementing\({IDS}\)"),
        "template": "@implementing({ids})",
    })
}

fn ts_wrapper_form() -> Value {
    json!({
        "name": "ts-implementing-wrapper", "language": "typescript",
        "syntax": "typescript_wrapper",
        "pattern": format!(r"\A(?:ixTrace\.)?implementing\({IDS}\)"),
        "template": "implementing({ids})",
    })
}

fn structural_forms() -> Vec<Value> {
    vec![py_form(), ts_decorator_form(), ts_wrapper_form()]
}

fn pytest_marker() -> Value {
    json!({
        "name": "pytest-trace", "language": "python",
        "pattern": r#"@pytest\.mark\.trace\(((?:\s*"[^"]*"\s*,?)+)\)"#,
        "template": "@pytest.mark.trace({ids})",
    })
}

fn load(
    dir: &std::path::Path,
    markers: Vec<Value>,
    implements: Vec<Value>,
) -> Result<quire_rs::Registry, String> {
    let module = dir.join("module");
    std::fs::create_dir_all(&module).unwrap();
    let manifest = json!({
        "name": "structural-fixture", "version": "0.1.0", "artifact_types": [],
        "traceability": { "trace_tags": { "markers": markers, "implements": implements } }
    });
    std::fs::write(
        module.join("manifest.yaml"),
        serde_json::to_string(&manifest).unwrap(),
    )
    .unwrap();
    let registry = quire_rs::Registry::load_module(&module).map_err(|e| e.to_string())?;
    if let Some(failure) = registry.failures().first() {
        return Err(failure.reason.clone());
    }
    Ok(registry)
}

struct Run {
    extraction: SymbolExtraction,
    graph: SymbolGraph,
    report: CoverageReport,
}

impl Run {
    /// `(symbol, id)` of every `implements` relation, sorted.
    fn implements(&self) -> Vec<(String, String)> {
        let mut out: Vec<_> = self
            .graph
            .implements
            .iter()
            .map(|r| (r.symbol.clone(), r.trace_id.clone()))
            .collect();
        out.sort();
        out
    }

    fn findings(&self, reason: &str) -> Vec<&quire_rs::coverage::CoverageDiagnostic> {
        self.report
            .diagnostics
            .iter()
            .filter(|d| d.reason == reason)
            .collect()
    }
}

fn pairs(items: &[(&str, &str)]) -> Vec<(String, String)> {
    let mut out: Vec<_> = items
        .iter()
        .map(|(s, i)| (s.to_string(), i.to_string()))
        .collect();
    out.sort();
    out
}

fn run_with(markers: Vec<Value>, implements: Vec<Value>, files: &[(&str, &str)]) -> Run {
    let dir = tempfile::tempdir().unwrap();
    let registry = load(dir.path(), markers, implements).unwrap();
    let src = dir.path().join("src");
    std::fs::create_dir(&src).unwrap();
    for (name, text) in files {
        std::fs::write(src.join(name), text).unwrap();
    }
    let extraction = quire_rs::symbols::extract_tree(&src);
    let graph = bind_trace::bind(&extraction, registry.traceability().unwrap());
    let spec = quire_rs::Spec::from_path(&src);
    let report = quire_rs::coverage::compute(&spec, &registry, &graph, &src).unwrap();
    Run {
        extraction,
        graph,
        report,
    }
}

fn run(files: &[(&str, &str)]) -> Run {
    run_with(vec![], structural_forms(), files)
}

#[trace("TC-2200", "FR-062-AC-18")]
#[test]
fn the_three_syntax_values_round_trip_and_bad_declarations_are_refused_at_load() {
    let dir = tempfile::tempdir().unwrap();
    let registry = load(dir.path(), vec![], structural_forms()).unwrap();
    let model = registry.traceability().unwrap();
    let syntaxes: Vec<_> = model
        .trace_tags
        .implements
        .iter()
        .map(|form| serde_json::to_value(form).unwrap()["syntax"].clone())
        .collect();
    assert_eq!(
        syntaxes,
        vec![
            json!("python_decorator"),
            json!("typescript_decorator"),
            json!("typescript_wrapper")
        ]
    );

    // Omission keeps the declared-text behaviour.
    let mut text = py_form();
    text.as_object_mut().unwrap().remove("syntax");
    text["pattern"] = r"# Implements: (FR-\d+)".into();
    let dir = tempfile::tempdir().unwrap();
    let registry = load(dir.path(), vec![], vec![text]).unwrap();
    assert!(registry.traceability().unwrap().trace_tags.implements[0]
        .syntax
        .is_none());

    for (syntax, language, section) in [
        ("unknown", "python", "implements"),
        ("python_decorator", "typescript", "implements"),
        ("typescript_decorator", "python", "implements"),
        ("typescript_wrapper", "python", "implements"),
        ("python_decorator", "python", "markers"),
        ("typescript_decorator", "typescript", "markers"),
        ("typescript_wrapper", "typescript", "markers"),
    ] {
        let form = json!({
            "name": "bad-form", "language": language, "syntax": syntax,
            "pattern": r#"\A@x\(("[^"]+")\)"#,
        });
        let (markers, implements) = if section == "markers" {
            (vec![form], vec![])
        } else {
            (vec![], vec![form])
        };
        let dir = tempfile::tempdir().unwrap();
        let error = load(dir.path(), markers, implements).unwrap_err();
        assert!(
            error.contains("bad-form"),
            "{syntax}/{language}/{section}: {error}"
        );
    }
}

const PY_BINDING: &str = r#"import ix_trace_py as t

@implements("FR-601", "FR-602")
def single():
    pass

@implements(
    "FR-603",
)
def wrapped():
    pass

@staticmethod
@implements("FR-604")
@dataclass
def stacked():
    pass

@implements("FR-605")
async def asynchronous():
    pass

@ix_trace_py.implements("FR-606")
def qualified():
    pass

@t.implements("FR-607")
def renamed():
    pass

@implements("FR-608")
class Klass:
    def method(self):
        pass
"#;

#[trace("TC-2201", "FR-062-AC-19")]
#[test]
fn a_python_decorator_binds_its_declaration_in_every_spelling() {
    let run = run(&[("lib.py", PY_BINDING)]);
    assert_eq!(
        run.implements(),
        pairs(&[
            ("single", "FR-601"),
            ("single", "FR-602"),
            ("wrapped", "FR-603"),
            ("stacked", "FR-604"),
            ("asynchronous", "FR-605"),
            ("qualified", "FR-606"),
            ("Klass", "FR-608"),
        ])
    );
    assert!(run.graph.verifies.is_empty());
    // The renamed import matches no pattern, so it is no relation and no finding.
    assert!(run.graph.partial_syntax_matches.is_empty());
    assert!(run.findings("partial-implements-syntax-match").is_empty());
}

#[trace("TC-2201", "FR-062-AC-19")]
#[test]
fn a_pattern_matching_only_a_substring_of_a_python_decorator_binds_nothing() {
    let mut form = py_form();
    form["pattern"] = format!(r"implements\({IDS}\)").into();
    let run = run_with(
        vec![],
        vec![form],
        &[(
            "lib.py",
            "@implements(\"FR-601\")\ndef a():\n    pass\n\n@t.implements(\"FR-602\")\ndef b():\n    pass\n",
        )],
    );
    assert!(run.implements().is_empty());
    let found = run.findings("partial-implements-syntax-match");
    assert_eq!(found.len(), 2);
    assert_eq!(found[0].declaration, "py-implements");
    assert_eq!(found[0].path.as_deref(), Some("lib.py"));
    assert_eq!(found[0].line, Some(1));
    assert_eq!(found[0].value.as_deref(), Some("@implements(\"FR-601\")"));
    assert!(found[0].message.contains(" a "), "{}", found[0].message);
    assert_eq!(found[1].line, Some(5));
}

const PY_SCOPE: &str = r##"# @implements("FR-611")
def commented():
    """@implements("FR-612")"""
    text = '@implements("FR-613")'
    return text

@implements("FR-614")
def control():
    @implements("FR-615")
    def nested():
        pass

    def sibling():
        pass

@implements("FR-616")
class Klass:
    def method(self):
        pass

@implements("FR-617")
def test_something():
    pass

@pytest.fixture
@implements("FR-618")
def a_fixture():
    pass
"##;

#[trace("TC-2202", "FR-062-AC-20")]
#[test]
fn python_decorator_text_elsewhere_binds_nothing_and_ownership_is_exact() {
    let run = run(&[("lib.py", PY_SCOPE)]);
    assert_eq!(
        run.implements(),
        pairs(&[
            ("control", "FR-614"),
            ("nested", "FR-615"),
            ("Klass", "FR-616"),
            ("a_fixture", "FR-618"),
        ])
    );
    assert!(run.graph.verifies.is_empty());
}

const TS_DECORATORS: &str = r#"@implementing("FR-701")
export class Before {
  @implementing("FR-702") static async m() {}
  @implementing(
    "FR-703",
  )
  n() {}
  @implementing("FR-704") get g() { return 1; }
  @implementing("FR-705") prop = 1;
  q(@implementing("FR-706") p: string) {}
  plain() {}
}
export @implementing("FR-707") class After {}
const Expr = @implementing("FR-708") class {};
@ixTrace.implementing("FR-709") class Plain {}
"#;

#[trace("TC-2203", "FR-062-AC-21")]
#[test]
fn a_typescript_decorator_binds_a_class_or_method_and_nothing_else() {
    let run = run(&[("lib.ts", TS_DECORATORS)]);
    assert_eq!(
        run.implements(),
        pairs(&[
            ("Before", "FR-701"),
            ("Before.m", "FR-702"),
            ("Before.n", "FR-703"),
            ("Before.g", "FR-704"),
            ("After", "FR-707"),
            ("Plain", "FR-709"),
        ])
    );
    assert!(run.graph.verifies.is_empty());
    assert!(run.findings("implements-syntax-unattached").is_empty());
    assert!(run.findings("partial-implements-syntax-match").is_empty());
}

#[trace("TC-2203", "FR-062-AC-21")]
#[test]
fn an_abstract_signature_decorator_rejects_the_file_with_the_per_file_diagnostic() {
    let run = run(&[
        (
            "bad.ts",
            "abstract class D {\n  @implementing(\"FR-720\") abstract f(): void;\n}\n",
        ),
        ("good.ts", "@implementing(\"FR-721\")\nclass E {}\n"),
    ]);
    assert_eq!(run.implements(), pairs(&[("E", "FR-721")]));
    let rejected: Vec<_> = run
        .extraction
        .diagnostics
        .iter()
        .filter(|d| d.path == "bad.ts")
        .collect();
    assert_eq!(rejected.len(), 1);
    assert!(rejected[0]
        .reason
        .contains("unresolvable declaration structure"));
    assert!(run.extraction.symbols.iter().all(|s| s.path != "bad.ts"));
}

const TS_SCOPE: &str = r##"// @implementing("FR-731") class InComment {}
const text = '@implementing("FR-732")';
const template = `@implementing("FR-733")`;

describe("suite", () => {
  @implementing("FR-734")
  class InSuite {
    @implementing("FR-735")
    method() {}
    other() {}
  }
});

@implementing("FR-736")
class Owner {
  a() {}
  @implementing("FR-737")
  b() {}
}
"##;

#[trace("TC-2204", "FR-062-AC-22")]
#[test]
fn typescript_decorator_text_elsewhere_binds_nothing_and_ownership_is_exact() {
    let run = run(&[("lib.ts", TS_SCOPE)]);
    assert_eq!(
        run.implements(),
        pairs(&[
            ("InSuite", "FR-734"),
            ("InSuite.method", "FR-735"),
            ("Owner", "FR-736"),
            ("Owner.b", "FR-737"),
        ])
    );
}

const TS_WRAPPERS: &str = r#"const plain = implementing("FR-741")((x: number) => x);
export const exported = implementing("FR-742")(() => 1);
let a = implementing("FR-743")(function () {}), b = ixTrace.implementing("FR-744")(async () => 2);
const nested = implementing("FR-745")(implementing("FR-746")(() => 3));
const memo = memoize(() => 4);
function declared() {}
"#;

#[trace("TC-2205", "FR-062-AC-23")]
#[trace("TC-2210", "FR-051-AC-51")]
#[test]
fn a_wrapper_chain_mints_a_function_symbol_and_binds_each_links_callee_call() {
    let run = run(&[("lib.ts", TS_WRAPPERS)]);
    assert_eq!(
        run.implements(),
        pairs(&[
            ("plain", "FR-741"),
            ("exported", "FR-742"),
            ("a", "FR-743"),
            ("b", "FR-744"),
            ("nested", "FR-745"),
            ("nested", "FR-746"),
        ])
    );
    let minted: BTreeSet<_> = run
        .extraction
        .symbols
        .iter()
        .filter(|s| {
            s.position == Some(quire_rs::traceability::TraceMarkerSyntax::TypescriptWrapper)
        })
        .map(|s| s.qualified_name.as_str())
        .collect();
    assert_eq!(
        minted,
        BTreeSet::from(["plain", "exported", "a", "b", "nested"])
    );
    for name in ["memo", "declared"] {
        let symbol = run
            .extraction
            .symbols
            .iter()
            .find(|s| s.qualified_name == name);
        assert!(
            symbol.is_none_or(|s| s.position.is_none()),
            "{name} must not be a wrapper symbol"
        );
    }
    assert!(run.findings("implements-syntax-unattached").is_empty());
    assert!(run.findings("partial-implements-syntax-match").is_empty());
}

#[trace("TC-2210", "FR-051-AC-51")]
#[test]
fn a_call_that_is_not_a_curried_wrapper_chain_mints_nothing() {
    let run = run(&[(
        "lib.ts",
        "const a = memoize(() => 1);\nconst b = callee(1)(memoize(() => 2));\nconst c = function () {};\nconst d = callee(1)(() => 3);\n",
    )]);
    let names: Vec<_> = run
        .extraction
        .symbols
        .iter()
        .map(|s| s.qualified_name.as_str())
        .collect();
    assert_eq!(names, vec!["lib", "d"]);
}

const TS_UNATTACHED: &str = r#"implementing("FR-751")(() => 1);
const arg = memoize(implementing("FR-752")(() => 2));
export default implementing("FR-753")(() => 3);
const inner = implementing("FR-754")(memoize(() => 4));
const control = implementing("FR-755")(() => 5);
@implementing("FR-756")
class Decorated {}
"#;

#[trace("TC-2206", "FR-062-AC-24")]
#[test]
fn a_wrapper_attached_to_nothing_binds_nothing_and_is_reported_once() {
    let run = run(&[("lib.ts", TS_UNATTACHED)]);
    assert_eq!(
        run.implements(),
        pairs(&[("control", "FR-755"), ("Decorated", "FR-756")])
    );
    let found = run.findings("implements-syntax-unattached");
    let at: Vec<_> = found
        .iter()
        .map(|d| (d.line.unwrap(), d.value.clone().unwrap()))
        .collect();
    assert_eq!(
        at,
        vec![
            (1, "implementing(\"FR-751\")".to_string()),
            (2, "implementing(\"FR-752\")".to_string()),
            (3, "implementing(\"FR-753\")".to_string()),
            (4, "implementing(\"FR-754\")".to_string()),
        ]
    );
    assert!(
        found
            .iter()
            .all(|d| d.declaration == "ts-implementing-wrapper"
                && d.path.as_deref() == Some("lib.ts"))
    );
}

#[trace("TC-2207", "FR-062-AC-25")]
#[test]
fn a_substring_match_of_a_typescript_candidate_is_a_partial_finding_and_a_renamed_one_is_silent() {
    let mut decorator = ts_decorator_form();
    decorator["pattern"] = format!(r"implementing\({IDS}\)").into();
    let mut wrapper = ts_wrapper_form();
    wrapper["pattern"] = format!(r"implementing\({IDS}\)").into();
    let source = r#"@implementing("FR-761")
class A {}
const w = t.implementing("FR-762")(() => 1);
@Injectable()
class B {}
"#;
    let run = run_with(vec![], vec![decorator, wrapper], &[("lib.ts", source)]);
    assert!(run.implements().is_empty());
    let found = run.findings("partial-implements-syntax-match");
    let at: Vec<_> = found
        .iter()
        .map(|d| {
            (
                d.declaration.as_str(),
                d.line.unwrap(),
                d.value.clone().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        at,
        vec![
            (
                "ts-implementing-decorator",
                1,
                "@implementing(\"FR-761\")".to_string()
            ),
            (
                "ts-implementing-wrapper",
                3,
                "t.implementing(\"FR-762\")".to_string()
            ),
        ]
    );

    // The start-anchored shipped patterns yield neither relation nor finding.
    let run = self::run(&[("lib.ts", "const w = t.implementing(\"FR-762\")(() => 1);\n@t.implementing(\"FR-763\")\nclass C {}\n@Injectable()\nclass D {}\n")]);
    assert!(run.implements().is_empty());
    assert!(run.graph.partial_syntax_matches.is_empty());
    assert!(run.findings("implements-syntax-unattached").is_empty());
}

fn advisory_run(files: &[(&str, &str)], implements: Vec<Value>) -> Run {
    let ts_marker = json!({
        "name": "ts-trace", "language": "typescript",
        "pattern": r#"\btrace\(((?:\s*"[^"]*"\s*,?)+)\)"#,
    });
    run_with(vec![pytest_marker(), ts_marker], implements, files)
}

#[trace("TC-2208", "FR-062-AC-26")]
#[test]
fn comment_forms_bind_nothing_under_structural_forms_and_the_advisory_names_the_structure() {
    let py = "# Implements: FR-007-AC-2\ndef production():\n    pass\n";
    let ts = "// Implements: FR-007-AC-2\nexport class Cls {\n  // Implements: FR-007-AC-2\n  method() {}\n}\n// Implements: FR-007-AC-2\nconst fn = () => 1;\n// Implements: FR-007-AC-2\nfunction decl() {}\n";
    let run = advisory_run(&[("lib.py", py), ("lib.ts", ts)], structural_forms());
    assert!(run.implements().is_empty());
    let values: Vec<_> = run
        .findings("trace-id-outside-marker")
        .iter()
        .map(|d| {
            (
                d.path.clone().unwrap(),
                d.line.unwrap(),
                d.value.clone().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        values,
        vec![
            (
                "lib.py".to_string(),
                1,
                "@implements(\"FR-007-AC-2\")".to_string()
            ),
            (
                "lib.ts".to_string(),
                1,
                "@implementing(\"FR-007-AC-2\")".to_string()
            ),
            (
                "lib.ts".to_string(),
                3,
                "@implementing(\"FR-007-AC-2\")".to_string()
            ),
            (
                "lib.ts".to_string(),
                6,
                "implementing(\"FR-007-AC-2\")".to_string()
            ),
            ("lib.ts".to_string(), 8, "function".to_string()),
        ]
    );

    // A module still declaring a textual form for the language binds as before.
    let textual = json!({
        "name": "py-text", "language": "python", "pattern": r"# Implements: (FR-\d+-AC-\d+)",
    });
    let mut forms = structural_forms();
    forms.push(textual);
    let run = advisory_run(&[("lib.py", py)], forms);
    // The synthetic module container reads the same file text too.
    assert_eq!(
        run.implements(),
        pairs(&[("lib", "FR-007-AC-2"), ("production", "FR-007-AC-2")])
    );
}

#[trace("TC-2209", "FR-062-AC-27")]
#[test]
fn structural_forms_preserve_identity_locations_ranges_isolation_and_determinism() {
    let plain =
        "def production():\n    pass\n\nclass Klass:\n    def method(self):\n        pass\n";
    let decorated = "# unrelated\n@implements(\"FR-801\")\ndef production():\n    pass\n\n@implements(\"FR-802\")\nclass Klass:\n    @implements(\"FR-803\")\n    def method(self):\n        pass\n";
    let identity = |run: &Run| -> Vec<_> {
        run.extraction
            .symbols
            .iter()
            .map(|s| (s.id.clone(), s.path.clone(), s.kind))
            .collect()
    };
    let before = run(&[("lib.py", plain)]);
    let after = run(&[("lib.py", decorated)]);
    assert_eq!(identity(&before), identity(&after));
    assert_eq!(
        after.implements(),
        pairs(&[
            ("production", "FR-801"),
            ("Klass", "FR-802"),
            ("Klass.method", "FR-803")
        ])
    );

    // Locations: the candidate's own line, after unrelated source and a
    // trailing comment.
    let ts = "const early = 1; // trailing\nimport x from 'y';\n\nconst w = implementing(\n  \"FR-804\",\n)(() => 1);\n";
    let located = run(&[("lib.ts", ts)]);
    let symbol = located
        .extraction
        .symbols
        .iter()
        .find(|s| s.qualified_name == "w")
        .unwrap();
    assert_eq!(symbol.attributes[0].line, 4);
    assert_eq!(symbol.attributes[0].text, "implementing(\n  \"FR-804\",\n)");

    // Range refusal: a range inside a decorator and a wrapper binds nothing.
    let ranged = run(&[
        (
            "r.py",
            "@implements(\"FR-810..FR-812\")\ndef p():\n    pass\n",
        ),
        (
            "r.ts",
            "const w = implementing(\"FR-820..FR-822\")(() => 1);\n",
        ),
    ]);
    assert!(ranged.implements().is_empty());
    let mut ranges: Vec<_> = ranged
        .graph
        .range_diagnostics
        .iter()
        .map(|r| (r.path.clone(), r.line, r.range_text.clone()))
        .collect();
    ranges.sort();
    assert_eq!(
        ranges,
        vec![
            ("r.py".to_string(), 1, "FR-810..FR-812".to_string()),
            ("r.ts".to_string(), 1, "FR-820..FR-822".to_string()),
        ]
    );

    // File isolation: a rejected file yields no relation; the valid file is unchanged.
    let isolated = run(&[
        ("bad.py", "@implements(\"FR-830\")\ndef broken(:\n"),
        ("good.py", decorated),
    ]);
    assert_eq!(isolated.implements(), after.implements());
    assert_eq!(isolated.extraction.diagnostics.len(), 1);
    assert_eq!(isolated.extraction.diagnostics[0].path, "bad.py");

    // Determinism: complete ordered graph and diagnostics repeat.
    let again = run(&[("lib.py", decorated)]);
    assert_eq!(after.graph, again.graph);
    assert_eq!(after.report.diagnostics, again.report.diagnostics);
}

#[trace("TC-2209", "FR-062-AC-27")]
#[test]
fn structural_scope_moves_no_verifies_edge_and_no_coverage_backing() {
    let dir = tempfile::tempdir().unwrap();
    let registry = load(dir.path(), vec![pytest_marker()], structural_forms()).unwrap();
    // Declare the criterion table so backing is computed.
    let path = dir.path().join("module/manifest.yaml");
    let mut manifest: Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    manifest["artifact_types"] = json!([{"name":"FR"}]);
    manifest["traceability"]["trace_targets"] = json!([{"name":"criterion","archetype":"FR","section":"Acceptance Criteria","id_column":"ID"}]);
    std::fs::write(&path, serde_json::to_string(&manifest).unwrap()).unwrap();
    drop(registry);
    let registry = quire_rs::Registry::load_module(&dir.path().join("module")).unwrap();
    assert!(registry.failures().is_empty(), "{:?}", registry.failures());

    let scope = dir.path().join("scope");
    let src = scope.join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::create_dir(scope.join("spec")).unwrap();
    std::fs::write(scope.join("spec/FR-901.md"), "---\nid: FR-901\ntype: FR\n---\n\n## Acceptance Criteria\n\n| ID | Criteria |\n|---|---|\n| FR-901-AC-1 | Backed control. |\n").unwrap();
    let evidence = "@pytest.mark.trace(\"FR-901-AC-1\")\ndef test_evidence():\n    pass\n\n@pytest.mark.trace(\"FR-999-AC-1\")\ndef test_untracked():\n    pass\n";
    std::fs::write(
        src.join("lib.py"),
        format!("{evidence}\ndef production():\n    pass\n"),
    )
    .unwrap();
    let spec = quire_rs::Spec::from_path(&scope);
    let model = registry.traceability().unwrap();
    let before_extraction = quire_rs::symbols::extract_tree(&src);
    let before = bind_trace::bind(&before_extraction, model);
    let before_report = quire_rs::coverage::compute(&spec, &registry, &before, &scope).unwrap();

    std::fs::write(
        src.join("lib.py"),
        format!("{evidence}\n@implements(\"FR-901-AC-1\")\ndef production():\n    pass\n\n@implements(\"FR-902\")\n@pytest.mark.trace(\"FR-903\")\ndef another():\n    pass\n"),
    )
    .unwrap();
    let after_extraction = quire_rs::symbols::extract_tree(&src);
    let after = bind_trace::bind(&after_extraction, model);
    let after_report = quire_rs::coverage::compute(&spec, &registry, &after, &scope).unwrap();

    assert_eq!(after.implements.len(), 2);
    assert_eq!(after.verifies, before.verifies);
    assert_eq!(after.backed_trace_ids(), before.backed_trace_ids());
    assert_eq!(after_report.totals, before_report.totals);
    assert_eq!(
        after_report.untracked_symbols,
        before_report.untracked_symbols
    );
    assert_eq!(before_report.totals.backed, 1);
}
