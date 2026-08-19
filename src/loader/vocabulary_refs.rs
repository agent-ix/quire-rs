//! Dereference named vocabularies in body-extraction asserts (FR-060).
//!
//! A `TestMatrix` contract that writes out
//! `column_choices: {Type: [Unit, Integration, ...]}` holds a second copy of a
//! list the traceability model already declares as
//! `vocabularies.test_type`. Two copies of one list stay in agreement only by
//! someone remembering — in this ecosystem, by one test in one module
//! (`spec-artifacts-process tests/test_manifest.py`). Since FR-054 added
//! `verification_method` and `verification_class` there are three lists worth
//! referencing, so the pressure grows rather than holds steady.
//!
//! `from_vocabulary` and `column_vocabularies` name the vocabulary instead.
//!
//! **Resolution happens here, at registry construction, and not in the
//! evaluator.** Two reasons, and the second is the one that fixes the shape:
//!
//! 1. The evaluator's signature stays as it is. Threading a `Registry` into
//!    `evaluate_assert` would change a public API every consumer of the
//!    per-document validation path inherits, to serve a lookup that is
//!    constant for the life of the registry.
//! 2. **The vocabulary a contract names may be declared by a different module
//!    than the archetype naming it**, so resolution cannot happen at module
//!    compile time — only after the cross-module merge. This is the point in
//!    the pipeline where the merged vocabularies and the compiled archetypes
//!    are both in hand.
//!
//! An archetype declaring neither key is returned untouched and never cloned,
//! so a module that has not adopted this pays nothing.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::extract::dsl::ExtractionDsl;
use crate::extract::locator::{Locator, LocatorAssert};
use crate::loader::compile::CompiledArchetype;

/// Implements: FR-060
/// Resolve every named vocabulary in `archetypes`, keyed by whatever the
/// caller keys them by.
///
/// `lookup` returns the values for a vocabulary name, or an empty slice when
/// nothing declares it. An unknown name resolves to an **empty** choice set,
/// which is deliberately not the same as "no constraint": a contract that
/// names a vocabulary nothing declares has asked for something the module set
/// cannot provide, and silently dropping the constraint would let a typo widen
/// the contract instead of failing it.
pub(crate) fn resolve_vocabularies<K: Ord + Clone>(
    archetypes: BTreeMap<K, Arc<CompiledArchetype>>,
    lookup: &impl Fn(&str) -> Vec<String>,
) -> BTreeMap<K, Arc<CompiledArchetype>> {
    archetypes
        .into_iter()
        .map(|(key, archetype)| {
            let resolved = match archetype.body_extraction.as_ref() {
                Some(dsl) if dsl_names_a_vocabulary(dsl) => {
                    let mut next = (*archetype).clone();
                    if let Some(dsl) = next.body_extraction.as_mut() {
                        resolve_dsl(dsl, lookup);
                    }
                    Arc::new(next)
                }
                // Untouched, and not cloned: a module that declares no named
                // vocabulary must be byte-identical to one built before this
                // FR existed.
                _ => archetype,
            };
            (key, resolved)
        })
        .collect()
}

/// Whether any assert in `dsl` names a vocabulary — the cheap pre-pass that
/// keeps the no-op case allocation-free.
fn dsl_names_a_vocabulary(dsl: &ExtractionDsl) -> bool {
    locators(dsl).any(|locator| {
        primitives(locator)
            .any(|assert| assert.from_vocabulary.is_some() || assert.column_vocabularies.is_some())
    })
}

fn locators(dsl: &ExtractionDsl) -> impl Iterator<Item = &Locator> {
    dsl.yield_pattern
        .r#match
        .iter()
        .flat_map(|m| m.values())
        .chain(dsl.yield_pattern.per_match.iter().flat_map(|m| m.values()))
}

fn primitives(locator: &Locator) -> impl Iterator<Item = &LocatorAssert> {
    let all: Vec<&LocatorAssert> = match locator {
        Locator::Primitive(p) => p.assert().into_iter().collect(),
        Locator::Fallback(chain) => chain.iter().filter_map(|p| p.assert()).collect(),
    };
    all.into_iter()
}

fn resolve_dsl(dsl: &mut ExtractionDsl, lookup: &impl Fn(&str) -> Vec<String>) {
    let mut all: Vec<&mut Locator> = Vec::new();
    if let Some(m) = dsl.yield_pattern.r#match.as_mut() {
        all.extend(m.values_mut());
    }
    if let Some(m) = dsl.yield_pattern.per_match.as_mut() {
        all.extend(m.values_mut());
    }
    for locator in all {
        match locator {
            Locator::Primitive(p) => {
                if let Some(assert) = p.assert_mut() {
                    resolve_assert(assert, lookup);
                }
            }
            Locator::Fallback(chain) => {
                for p in chain.iter_mut() {
                    if let Some(assert) = p.assert_mut() {
                        resolve_assert(assert, lookup);
                    }
                }
            }
        }
    }
}

/// Dereference one assert's named vocabularies into literal choices.
///
/// A literal `choices` / `column_choices` beside the reference **wins**, and
/// the reference is dropped rather than merged. Two sources for one constraint
/// is the duplication this FR removes; silently unioning them would recreate it
/// inside a single assert, where it would be even harder to see.
fn resolve_assert(assert: &mut LocatorAssert, lookup: &impl Fn(&str) -> Vec<String>) {
    if let Some(name) = assert.from_vocabulary.take() {
        if assert.choices.is_none() {
            assert.choices = Some(lookup(&name));
        }
    }
    if let Some(named) = assert.column_vocabularies.take() {
        let mut columns = assert.column_choices.take().unwrap_or_default();
        for (header, name) in named {
            columns.entry(header).or_insert_with(|| lookup(&name));
        }
        assert.column_choices = Some(columns);
    }
}
