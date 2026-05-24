//! Strict MiniJinja environment construction (FR-004).
//!
//! Built once per `Registry::load_from` call and shared across all
//! render operations. Configuration:
//!
//! - `UndefinedBehavior::Strict` so missing template fields raise
//!   instead of silently substituting empty (FR-004-AC-1).
//! - No loader registered → MiniJinja's `{% include %}` and
//!   `{% extends %}` directives produce template errors at render
//!   time. Authoring tools that try to ship a template with includes
//!   can additionally be caught at template-add time via
//!   [`reject_includes`] (FR-004-AC-4).

use minijinja::{Environment, UndefinedBehavior};

/// Build the strict, isolated MiniJinja environment used by the
/// loader and by `render::render`.
pub fn build_strict_env() -> Environment<'static> {
    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Strict);
    env
}

/// Sniff `source` for an `{% include %}` or `{% extends %}` directive
/// and refuse it at template-registration time (FR-004-AC-4).
///
/// MiniJinja already errors at render time when a loader-less env
/// encounters these directives, but the spec wants the failure
/// surfaced at load time so authoring tools get immediate feedback.
pub fn reject_includes(source: &str) -> Result<(), String> {
    let mut i: usize = 0;
    let bytes = source.as_bytes();
    while i + 2 < bytes.len() {
        if &bytes[i..i + 2] == b"{%" {
            let close_off = source[i..].find("%}");
            let close = match close_off {
                Some(c) => i + c + 2,
                None => return Ok(()),
            };
            // Strip whitespace and an optional leading `-` (MiniJinja's
            // `{%- ... -%}` whitespace-control prefix) before reading
            // the tag keyword.
            let tag = source[i + 2..close - 2]
                .trim()
                .trim_start_matches('-')
                .trim();
            let leading = tag.split_whitespace().next().unwrap_or("");
            if leading == "include" || leading == "extends" {
                return Err(format!("{{% {leading} %}} is not supported at v1 (FR-004)"));
            }
            i = close;
        } else {
            i += 1;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_strict_env_renders_a_simple_template() {
        let mut env = build_strict_env();
        env.add_template("ok", "hello {{ name }}").unwrap();
        let out = env
            .get_template("ok")
            .unwrap()
            .render(minijinja::context!(name => "world"))
            .unwrap();
        assert_eq!(out, "hello world");
    }

    // FR-004-AC-1: strict mode surfaces undefined variables as errors.
    #[test]
    fn strict_env_errors_on_undefined_variable() {
        let mut env = build_strict_env();
        env.add_template("strict", "hello {{ name }}").unwrap();
        let err = env
            .get_template("strict")
            .unwrap()
            .render(minijinja::context!())
            .expect_err("strict undefined error");
        let msg = err.to_string();
        assert!(msg.contains("name") || msg.contains("undefined"), "{msg}");
    }

    // FR-004-AC-4: {% include %} is rejected at load time.
    #[test]
    fn reject_includes_catches_include_directive() {
        let err = reject_includes("hello\n{% include \"other.j2\" %}").err();
        assert!(err.is_some());
        let msg = err.unwrap();
        assert!(msg.contains("include"));
    }

    #[test]
    fn reject_includes_catches_extends_directive() {
        let err = reject_includes("{% extends 'base' %}").err();
        assert!(err.is_some());
    }

    #[test]
    fn reject_includes_accepts_normal_template() {
        reject_includes("hello {{ name }}\n{% if x %}{{ x }}{% endif %}").unwrap();
    }

    #[test]
    fn reject_includes_accepts_for_loops_and_set() {
        reject_includes("{% for x in xs %}{{ x }}{% endfor %}{% set y = 1 %}").unwrap();
    }

    #[test]
    fn reject_includes_handles_whitespace_control_dash_prefix() {
        let err = reject_includes("{%- include 'x' -%}").err();
        assert!(err.is_some(), "should still catch with - prefix");
    }
}
