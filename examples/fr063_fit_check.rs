// FR-063 advisory fit check. Repository membership is supplied by
// scripts/plain_language_sweep.py, which imports the corpus authority rather
// than copying its exclusions here.
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use quire_rs::{check_plain_language_at, PlainLanguageProfile};
use serde::Serialize;

#[derive(Serialize)]
struct Summary {
    profile: PlainLanguageProfile,
    repositories: usize,
    documents_examined: usize,
    readable_documents: usize,
    readable_blocks: usize,
    findings_by_rule: BTreeMap<String, usize>,
    documents_with_any_finding: usize,
    documents_with_findings_by_rule: BTreeMap<String, usize>,
    skipped_by_reason: BTreeMap<String, usize>,
    samples_by_rule: BTreeMap<String, Vec<String>>,
}

fn main() {
    let repositories: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    if repositories.is_empty() {
        eprintln!("usage: fr063_fit_check <repository>...");
        std::process::exit(2);
    }
    let profile = measurement_profile();
    let mut summary = Summary {
        profile: profile.clone(),
        repositories: repositories.len(),
        documents_examined: 0,
        readable_documents: 0,
        readable_blocks: 0,
        findings_by_rule: BTreeMap::new(),
        documents_with_any_finding: 0,
        documents_with_findings_by_rule: BTreeMap::new(),
        skipped_by_reason: BTreeMap::new(),
        samples_by_rule: BTreeMap::new(),
    };
    let mut documents_with_any_finding = BTreeSet::new();
    let mut documents_with_findings_by_rule: BTreeMap<String, BTreeSet<PathBuf>> = BTreeMap::new();

    for repository in repositories {
        let report =
            check_plain_language_at(&repository.join("spec"), "measurement-2026-08-21", &profile);
        summary.documents_examined += report.documents_examined;
        summary.readable_documents += report.readable_documents;
        summary.readable_blocks += report.readable_blocks;
        for skipped in report.skipped_inputs {
            *summary.skipped_by_reason.entry(skipped.reason).or_default() += 1;
        }
        for finding in report.findings {
            documents_with_any_finding.insert(finding.path.clone());
            documents_with_findings_by_rule
                .entry(finding.rule.clone())
                .or_default()
                .insert(finding.path.clone());
            *summary
                .findings_by_rule
                .entry(finding.rule.clone())
                .or_default() += 1;
            let samples = summary.samples_by_rule.entry(finding.rule).or_default();
            if samples.len() < 20 {
                samples.push(format!(
                    "{}:{}: {} — {}",
                    finding.path.display(),
                    finding.line,
                    finding.message,
                    finding.excerpt
                ));
            }
        }
    }

    summary.documents_with_any_finding = documents_with_any_finding.len();
    summary.documents_with_findings_by_rule = documents_with_findings_by_rule
        .into_iter()
        .map(|(rule, paths)| (rule, paths.len()))
        .collect();

    println!(
        "{}",
        serde_json::to_string_pretty(&summary).expect("summary is serializable")
    );
}

fn measurement_profile() -> PlainLanguageProfile {
    let definitions = [
        ("AC", "acceptance criterion"),
        ("ADR", "architecture decision record"),
        ("API", "application programming interface"),
        ("AST", "abstract syntax tree"),
        ("CI", "continuous integration"),
        ("CLI", "command-line interface"),
        ("CPU", "central processing unit"),
        ("DSL", "domain-specific language"),
        ("FR", "functional requirement"),
        ("HTML", "hypertext markup language"),
        ("HTTP", "hypertext transfer protocol"),
        ("ID", "identifier"),
        ("IO", "input/output"),
        ("JSON", "JavaScript object notation"),
        ("LLM", "large language model"),
        ("NFR", "non-functional requirement"),
        ("OS", "operating system"),
        ("PR", "pull request"),
        ("RAM", "random-access memory"),
        ("SLO", "service-level objective"),
        ("SQL", "structured query language"),
        ("SR", "specification review"),
        ("TC", "test case"),
        ("UI", "user interface"),
        ("URI", "uniform resource identifier"),
        ("URL", "uniform resource locator"),
        ("US", "user story"),
        ("UTF", "Unicode transformation format"),
        ("UUID", "universally unique identifier"),
        ("YAML", "YAML data serialization language"),
    ];
    PlainLanguageProfile {
        version: "2026-08-21".to_string(),
        document_types: vec![
            "FR".to_string(),
            "NFR".to_string(),
            "StR".to_string(),
            "US".to_string(),
            "ADR".to_string(),
        ],
        sentence_word_limit: 35,
        max_heading_level_step: 1,
        known_acronyms: definitions
            .into_iter()
            .map(|(name, definition)| (name.to_string(), definition.to_string()))
            .collect(),
        ignored_uppercase_terms: [
            "AND", "DELETE", "DRAFT", "ELSE", "ERROR", "FAIL", "FALSE", "GET", "GIVEN", "IF",
            "MAY", "MUST", "NONE", "NOT", "NOTE", "NULL", "OR", "PASS", "PATCH", "POST", "PUT",
            "RETIRED", "SHALL", "SHOULD", "TBD", "THEN", "TODO", "TRUE", "WARNING", "WHEN",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>(),
    }
}
