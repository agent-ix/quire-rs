// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

use std::path::{Path, PathBuf};

use ix_trace_rs::trace;

fn rust_sources(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(path) = pending.pop() {
        for entry in std::fs::read_dir(path)
            .expect("read source directory")
            .flatten()
        {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

fn is_non_exhaustive(lines: &[&str], item_line: usize) -> bool {
    for line in lines[..item_line].iter().rev() {
        let line = line.trim();
        if line == "#[non_exhaustive]" {
            return true;
        }
        if line.is_empty() || line.starts_with("#[") || line.starts_with("///") {
            continue;
        }
        break;
    }
    false
}

#[trace("NFR-010-AC-2")]
#[test]
fn every_reexported_public_enum_is_non_exhaustive() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib = std::fs::read_to_string(root.join("src/lib.rs")).expect("read crate root");
    let sources = rust_sources(&root.join("src"));
    let source_text: Vec<_> = sources
        .iter()
        .map(|path| {
            (
                path,
                std::fs::read_to_string(path).expect("read Rust source"),
            )
        })
        .collect();
    let mut checked = 0;
    let mut violations = Vec::new();

    for statement in lib
        .split(';')
        .filter(|statement| statement.trim().starts_with("pub use "))
    {
        let Some((_, names)) = statement.split_once('{') else {
            continue;
        };
        let Some((names, _)) = names.split_once('}') else {
            continue;
        };
        let names = names
            .lines()
            .map(|line| line.split("//").next().unwrap_or_default())
            .collect::<Vec<_>>()
            .join(" ");
        for imported in names.split(',') {
            let imported = imported.trim();
            let Some(name) = imported
                .split_whitespace()
                .next()
                .filter(|name| !name.is_empty())
            else {
                continue;
            };
            let name = name.rsplit("::").next().unwrap_or(name);
            for (path, source) in &source_text {
                let lines: Vec<_> = source.lines().collect();
                for (index, line) in lines.iter().enumerate() {
                    let declaration = line.trim();
                    let declared_name = declaration.strip_prefix("pub enum ").and_then(|rest| {
                        rest.split(|character: char| {
                            !character.is_ascii_alphanumeric() && character != '_'
                        })
                        .next()
                    });
                    if declared_name == Some(name) {
                        checked += 1;
                        if !is_non_exhaustive(&lines, index) {
                            violations.push(format!(
                                "{}:{}: public enum {name} lacks #[non_exhaustive]",
                                path.display(),
                                index + 1
                            ));
                        }
                    }
                }
            }
        }
    }

    assert!(
        checked > 0,
        "no public enums were found in lib.rs re-exports"
    );
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}
