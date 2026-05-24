//! Deep JSON merge (FR-002 step 1).
//!
//! Per-key right-wins. Nested objects merge recursively. Arrays and
//! scalars are replaced wholesale (Filament convention — append
//! semantics would surprise authors who use arrays for "the full set
//! after edit"). Pure function over `serde_json::Value`.

use serde_json::Value;

/// Merge `patch` onto `current`, returning a new value.
pub fn deep_merge(current: &Value, patch: &Value) -> Value {
    match (current, patch) {
        (Value::Object(a), Value::Object(b)) => {
            let mut out = a.clone();
            for (k, vb) in b {
                match out.get(k) {
                    Some(va) => {
                        out.insert(k.clone(), deep_merge(va, vb));
                    }
                    None => {
                        out.insert(k.clone(), vb.clone());
                    }
                }
            }
            Value::Object(out)
        }
        // Arrays and scalars are wholesale-replaced by the patch.
        _ => patch.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn merges_disjoint_keys() {
        let a = json!({"a": 1});
        let b = json!({"b": 2});
        assert_eq!(deep_merge(&a, &b), json!({"a": 1, "b": 2}));
    }

    #[test]
    fn patch_wins_for_overlapping_scalar() {
        let a = json!({"title": "old"});
        let b = json!({"title": "new"});
        assert_eq!(deep_merge(&a, &b), json!({"title": "new"}));
    }

    // FR-002-AC-1
    #[test]
    fn preserves_siblings_during_merge() {
        let a = json!({"title": "old", "body": "content"});
        let b = json!({"title": "new"});
        assert_eq!(
            deep_merge(&a, &b),
            json!({"title": "new", "body": "content"})
        );
    }

    #[test]
    fn nested_objects_merge_recursively() {
        let a = json!({"meta": {"author": "alice", "year": 2024}});
        let b = json!({"meta": {"year": 2025, "tag": "fr"}});
        assert_eq!(
            deep_merge(&a, &b),
            json!({"meta": {"author": "alice", "year": 2025, "tag": "fr"}})
        );
    }

    #[test]
    fn arrays_are_replaced_wholesale_not_appended() {
        let a = json!({"tags": ["a", "b", "c"]});
        let b = json!({"tags": ["x"]});
        assert_eq!(deep_merge(&a, &b), json!({"tags": ["x"]}));
    }

    #[test]
    fn null_patch_replaces_object() {
        let a = json!({"k": "v"});
        let b = json!(null);
        assert_eq!(deep_merge(&a, &b), json!(null));
    }
}
