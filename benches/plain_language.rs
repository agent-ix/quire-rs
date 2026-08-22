use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use quire_rs::{check_plain_language, reader_blocks, PlainLanguageProfile};

const LARGE_DOCUMENT: &str = include_str!("../spec/tests.md");

fn profile() -> PlainLanguageProfile {
    PlainLanguageProfile {
        version: "bench".to_string(),
        document_types: Vec::new(),
        sentence_word_limit: 35,
        max_heading_level_step: 1,
        known_acronyms: BTreeMap::new(),
        ignored_uppercase_terms: BTreeSet::from([
            "MAY".to_string(),
            "MUST".to_string(),
            "SHALL".to_string(),
        ]),
    }
}

fn bench_plain_language(c: &mut Criterion) {
    c.bench_function("reader_blocks/spec_tests", |b| {
        b.iter(|| reader_blocks(black_box(LARGE_DOCUMENT)))
    });
    let profile = profile();
    c.bench_function("plain_language/spec_tests", |b| {
        b.iter(|| {
            check_plain_language(
                black_box(Path::new("spec/tests.md")),
                black_box(LARGE_DOCUMENT),
                black_box(&profile),
            )
        })
    });
}

criterion_group!(benches, bench_plain_language);
criterion_main!(benches);
