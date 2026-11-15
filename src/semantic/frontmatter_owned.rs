//! Frontmatter-owned record keys (FR-069 "Frontmatter-owned record keys",
//! FR-081).
//!
//! An object type that declares a `frontmatter_field` locator under key `K` in
//! `yield_pattern.match`, where `K` is a property of its `data_schema`, owns `K`
//! in frontmatter. The record carries what the locator resolves to, by the
//! FR-011 locator semantics and no others. The engine holds no key name: the
//! declaration carries it.

use std::collections::BTreeSet;

use serde_json::Value;

use crate::ast::QuireDocument;
use crate::extract::dsl::ExtractionDsl;
use crate::extract::locator::{eval_located, is_whole_value_mustache, Locator, LocatorPrimitive};

/// The `frontmatter_field` locators of an object type's `yield_pattern.match`.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct FrontmatterOwned(Vec<Owned>);

#[derive(Debug, Clone, PartialEq)]
struct Owned {
    key: String,
    path: Vec<String>,
    locator: LocatorPrimitive,
}

/// What a frontmatter-owned key resolved to in one document.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Projected {
    pub(crate) key: String,
    /// The frontmatter path the value was read from.
    pub(crate) path: Vec<String>,
    pub(crate) value: Value,
    /// 1-based document line of the resolved leaf key.
    pub(crate) line: usize,
}

impl FrontmatterOwned {
    pub(crate) fn from_dsl(dsl: &ExtractionDsl) -> Self {
        let owned =
            dsl.yield_pattern
                .r#match
                .iter()
                .flatten()
                .filter_map(|(key, locator)| match locator {
                    Locator::Primitive(
                        primitive @ LocatorPrimitive::FrontmatterField { path, .. },
                    ) if !path.is_empty() => Some(Owned {
                        key: key.clone(),
                        path: path.clone(),
                        locator: primitive.clone(),
                    }),
                    _ => None,
                })
                .collect();
        Self(owned)
    }

    pub(crate) fn owns(&self, key: &str) -> bool {
        self.0.iter().any(|owned| owned.key == key)
    }

    /// The frontmatter path that fills `key`, when `key` is frontmatter-owned.
    pub(crate) fn path_of(&self, key: &str) -> Option<&[String]> {
        self.0
            .iter()
            .find(|owned| owned.key == key)
            .map(|owned| owned.path.as_slice())
    }

    /// The authored frontmatter keys that fill an owned key `is_property`
    /// accepts. They are mapped, so they are not also record properties of
    /// their own, unless the key is itself a different record property.
    pub(crate) fn mapped_roots(&self, is_property: impl Fn(&str) -> bool) -> BTreeSet<&str> {
        self.0
            .iter()
            .filter(|owned| is_property(&owned.key))
            .filter_map(|owned| {
                let root = owned.path.first()?.as_str();
                (root == owned.key || !is_property(root)).then_some(root)
            })
            .collect()
    }

    /// The value each owned key resolves to in `raw`, for the keys `is_property`
    /// accepts. A key whose path is absent, or whose `regex` does not match,
    /// is not returned; an explicit `null` is.
    pub(crate) fn project(&self, raw: &str, is_property: impl Fn(&str) -> bool) -> Vec<Projected> {
        let owned: Vec<&Owned> = self.0.iter().filter(|o| is_property(&o.key)).collect();
        if owned.is_empty() {
            return Vec::new();
        }
        let doc = crate::parse_document(raw);
        owned
            .into_iter()
            .filter_map(|owned| owned.resolve(raw, &doc))
            .collect()
    }
}

impl Owned {
    fn resolve(&self, raw: &str, doc: &QuireDocument) -> Option<Projected> {
        let multiple = self.locator.multiple();
        let (values, line) = if leaf_is_null(doc, &self.path) {
            let line = crate::extract::frontmatter_key_line(raw, &self.path)?;
            (vec![Value::Null], line)
        } else {
            let located = eval_located(doc, &self.locator);
            let line = located.iter().find_map(|l| l.line)?;
            let values = located
                .into_iter()
                .map(|l| l.value)
                .filter(|v| !matches!(v, Value::String(s) if is_whole_value_mustache(s)))
                .collect::<Vec<_>>();
            (values, line)
        };
        let value = if multiple {
            (!values.is_empty()).then_some(Value::Array(values))?
        } else {
            values.into_iter().next()?
        };
        Some(Projected {
            key: self.key.clone(),
            path: self.path.clone(),
            value,
            line,
        })
    }
}

/// Whether `path` resolves in the frontmatter to an explicit `null`.
fn leaf_is_null(doc: &QuireDocument, path: &[String]) -> bool {
    let Some(frontmatter) = &doc.frontmatter else {
        return false;
    };
    let mut keys = path.iter();
    let Some(first) = keys.next() else {
        return false;
    };
    let mut current = frontmatter.get(first);
    for key in keys {
        current = current.and_then(Value::as_object).and_then(|m| m.get(key));
    }
    current.is_some_and(Value::is_null)
}
