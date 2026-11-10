//! Unit and dimension token resolution (FR-070 "Unit and dimension tokens").
//!
//! A token is a bare `Identifier` or a qualified `<org>/<repo>::<Identifier>`.
//! It resolves from the `BundleIndex` alone: the bundle's own artifacts of the
//! required object type first, then the `importedArtifacts` of every imported
//! package. A match never crosses an owner; the identity names the package
//! that owns the artifact.

use super::context::SemanticContext;
use super::decl::is_identifier;

/// A parsed unit or dimension token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnitToken<'a> {
    Bare(&'a str),
    Qualified { package: &'a str, name: &'a str },
}

impl<'a> UnitToken<'a> {
    /// `Identifier` or `<org>/<repo>::Identifier`; anything else is `None`.
    pub(crate) fn parse(text: &'a str) -> Option<Self> {
        match text.split_once("::") {
            None => is_identifier(text).then_some(Self::Bare(text)),
            Some((package, name)) => (is_package(package) && is_identifier(name))
                .then_some(Self::Qualified { package, name }),
        }
    }

    /// The `Identifier` the token names.
    pub(crate) fn name(&self) -> &'a str {
        match self {
            Self::Bare(name) | Self::Qualified { name, .. } => name,
        }
    }
}

/// `<org>/<repo>`: two non-empty segments of the package alphabet.
fn is_package(text: &str) -> bool {
    let segment = |s: &str| {
        s.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
            && s.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "._-".contains(c))
    };
    text.split_once('/')
        .is_some_and(|(org, repo)| segment(org) && segment(repo))
}

/// The object type a token's artifact must carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TokenKind {
    Unit,
    /// A dimension token, read by the FR-075 dimension tables.
    #[cfg_attr(not(test), allow(dead_code))]
    Dimension,
}

impl TokenKind {
    fn object(self) -> &'static str {
        match self {
            Self::Unit => "unit",
            Self::Dimension => "dimension",
        }
    }
}

/// Why a token resolved to no artifact: the `reason` of
/// `semantic.unresolved-type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnresolvedReason {
    NoBundleIndex,
    UnknownToken,
    ImportUnresolved,
}

impl UnresolvedReason {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::NoBundleIndex => "no-bundle-index",
            Self::UnknownToken => "unknown-token",
            Self::ImportUnresolved => "import-unresolved",
        }
    }
}

/// What a token resolved to.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Resolution {
    /// The `ix://<package>/type/<id>` identity of the matched artifact.
    Resolved(String),
    /// More than one artifact matched; the message names each.
    Ambiguous(String),
    /// No artifact of the required object type matched; `reason` is the one
    /// `semantic.unresolved-type` reason.
    Unresolved { reason: UnresolvedReason },
}

/// One artifact a token may match.
struct Candidate<'a> {
    package: &'a str,
    id: &'a str,
    names: Vec<&'a str>,
}

/// The own bundle's artifacts of `object`, each with the names the bundle's
/// `objects` list for its `id`.
fn own_candidates<'a>(ctx: &'a SemanticContext, object: &str) -> Vec<Candidate<'a>> {
    let package = ctx.identity_package();
    let mut out: Vec<Candidate<'a>> = Vec::new();
    for artifact in &ctx.bundle.artifacts {
        if artifact.object.as_deref() != Some(object) {
            continue;
        }
        // One declaration authored in two forms shares an `id`: one artifact.
        if out.iter().any(|c| c.id == artifact.id) {
            continue;
        }
        let names = ctx
            .bundle
            .objects
            .iter()
            .filter(|o| o.id == artifact.id)
            .flat_map(|o| o.names.iter().map(String::as_str))
            .chain(std::iter::once(artifact.id.as_str()))
            .collect();
        out.push(Candidate {
            package,
            id: &artifact.id,
            names,
        });
    }
    out
}

/// The imported artifacts of `object`, optionally of one `package`.
fn imported_candidates<'a>(
    ctx: &'a SemanticContext,
    object: &str,
    package: Option<&str>,
) -> Vec<Candidate<'a>> {
    ctx.bundle
        .imported_artifacts
        .iter()
        .filter(|a| a.object.as_deref() == Some(object))
        .filter(|a| package.is_none_or(|p| a.package == p))
        .map(|a| Candidate {
            package: &a.package,
            id: &a.id,
            names: a
                .names
                .iter()
                .map(String::as_str)
                .chain(std::iter::once(a.id.as_str()))
                .collect(),
        })
        .collect()
}

/// The candidates `name` matches: those whose `id` equals it, else those one
/// of whose names equals it.
fn matching<'c, 'a>(candidates: &'c [Candidate<'a>], name: &str) -> Vec<&'c Candidate<'a>> {
    let by_id: Vec<_> = candidates.iter().filter(|c| c.id == name).collect();
    if !by_id.is_empty() {
        return by_id;
    }
    candidates
        .iter()
        .filter(|c| c.names.contains(&name))
        .collect()
}

fn settle(matches: &[&Candidate<'_>], token: &str) -> Option<Resolution> {
    match matches {
        [] => None,
        [one] => Some(Resolution::Resolved(format!(
            "ix://{}/type/{}",
            one.package, one.id
        ))),
        many => {
            let named: Vec<String> = many
                .iter()
                .map(|c| format!("{} of {}", c.id, c.package))
                .collect();
            Some(Resolution::Ambiguous(format!(
                "token {token:?} names {}",
                named.join(" and ")
            )))
        }
    }
}

/// Resolve `token` to the artifact of `object` type (`unit` or `dimension`)
/// it names, reading only the `BundleIndex`.
pub(crate) fn resolve_token(
    ctx: &SemanticContext,
    kind: TokenKind,
    token: UnitToken<'_>,
) -> Resolution {
    let object = kind.object();
    if ctx.bundle.is_empty() && ctx.bundle.artifacts.is_empty() {
        return Resolution::Unresolved {
            reason: UnresolvedReason::NoBundleIndex,
        };
    }
    let own = own_candidates(ctx, object);
    let found = match token {
        UnitToken::Bare(name) => {
            let found = settle(&matching(&own, name), name).or_else(|| {
                let imported = imported_candidates(ctx, object, None);
                settle(&matching(&imported, name), name)
            });
            found
        }
        UnitToken::Qualified { package, name } => {
            if package == ctx.module.package {
                settle(&matching(&own, name), name)
            } else if !ctx.module.imports.contains_key(package) {
                return Resolution::Unresolved {
                    reason: UnresolvedReason::UnknownToken,
                };
            } else if !ctx
                .bundle
                .imported_artifacts
                .iter()
                .any(|a| a.package == package)
            {
                return Resolution::Unresolved {
                    reason: UnresolvedReason::ImportUnresolved,
                };
            } else {
                let imported = imported_candidates(ctx, object, Some(package));
                settle(&matching(&imported, name), name)
            }
        }
    };
    found.unwrap_or(Resolution::Unresolved {
        reason: UnresolvedReason::UnknownToken,
    })
}

#[cfg(test)]
mod tests {
    use ix_trace_rs::trace;
    use serde_json::json;

    use super::*;
    use crate::semantic::context::BundleIndex;
    use crate::semantic::contract::SemanticModule;

    fn module() -> SemanticModule {
        SemanticModule {
            contract_version: "1.0.0".to_string(),
            package: "acme/own".to_string(),
            exports: Vec::new(),
            imports: ["acme/time-units", "acme/other-units", "acme/missing"]
                .into_iter()
                .map(|p| (p.to_string(), "1".to_string()))
                .collect(),
            import_types: Default::default(),
            targets: Vec::new(),
            compatibility_posture: "none".to_string(),
            legacy_forms: "warn".to_string(),
            mappings: Vec::new(),
        }
    }

    fn context(bundle: serde_json::Value) -> SemanticContext {
        let bundle: BundleIndex = serde_json::from_value(bundle).expect("bundle");
        SemanticContext::new(module(), "doc.md", bundle)
    }

    fn imported_bundle() -> serde_json::Value {
        json!({
            "package": "acme/own",
            "artifacts": [{ "id": "s", "object": "unit", "operations": [] }],
            "importedArtifacts": [
                { "package": "acme/time-units", "id": "ms", "names": ["Millisecond"], "object": "unit" },
                { "package": "acme/other-units", "id": "ms", "names": [], "object": "unit" },
                { "package": "acme/time-units", "id": "Time", "names": [], "object": "dimension" }
            ]
        })
    }

    fn resolve(ctx: &SemanticContext, kind: TokenKind, token: &str) -> Resolution {
        resolve_token(ctx, kind, UnitToken::parse(token).expect("token"))
    }

    fn resolved(identity: &str) -> Resolution {
        Resolution::Resolved(identity.to_string())
    }

    fn unresolved(reason: UnresolvedReason) -> Resolution {
        Resolution::Unresolved { reason }
    }

    #[trace("TC-1952", "FR-070-AC-14")]
    #[test]
    fn bare_tokens_prefer_the_own_bundle_then_the_imports() {
        let ctx = context(imported_bundle());
        assert_eq!(
            resolve(&ctx, TokenKind::Unit, "s"),
            resolved("ix://acme/own/type/s")
        );
        assert_eq!(
            resolve(&ctx, TokenKind::Unit, "Millisecond"),
            resolved("ix://acme/time-units/type/ms")
        );
        match resolve(&ctx, TokenKind::Unit, "ms") {
            Resolution::Ambiguous(message) => {
                assert!(
                    message.contains("acme/time-units") && message.contains("acme/other-units")
                );
            }
            other => panic!("expected ambiguity, got {other:?}"),
        }
        let mut with_own = imported_bundle();
        with_own["artifacts"]
            .as_array_mut()
            .expect("artifacts")
            .push(json!({ "id": "ms", "object": "unit", "operations": [] }));
        let ctx = context(with_own);
        assert_eq!(
            resolve(&ctx, TokenKind::Unit, "ms"),
            resolved("ix://acme/own/type/ms")
        );
    }

    #[trace("TC-1952", "FR-070-AC-14")]
    #[test]
    fn qualified_tokens_consult_only_their_package() {
        let ctx = context(imported_bundle());
        assert_eq!(
            resolve(&ctx, TokenKind::Unit, "acme/time-units::ms"),
            resolved("ix://acme/time-units/type/ms")
        );
        assert_eq!(
            resolve(&ctx, TokenKind::Unit, "acme/other-units::ms"),
            resolved("ix://acme/other-units/type/ms")
        );
        assert_eq!(
            resolve(&ctx, TokenKind::Unit, "acme/own::s"),
            resolved("ix://acme/own/type/s")
        );
        assert_eq!(
            resolve(&ctx, TokenKind::Unit, "acme/time-units::Time"),
            unresolved(UnresolvedReason::UnknownToken)
        );
        assert_eq!(
            resolve(&ctx, TokenKind::Unit, "acme/none::ms"),
            unresolved(UnresolvedReason::UnknownToken)
        );
        assert_eq!(
            resolve(&ctx, TokenKind::Unit, "acme/missing::ms"),
            unresolved(UnresolvedReason::ImportUnresolved)
        );
    }

    #[trace("TC-1952", "FR-070-AC-14")]
    #[test]
    fn dimensions_carry_their_owner_in_the_identity() {
        let mut bundle = imported_bundle();
        let ctx = context(bundle.clone());
        assert_eq!(
            resolve(&ctx, TokenKind::Dimension, "Time"),
            resolved("ix://acme/time-units/type/Time")
        );
        bundle["artifacts"]
            .as_array_mut()
            .expect("artifacts")
            .push(json!({ "id": "Time", "object": "dimension", "operations": [] }));
        let ctx = context(bundle);
        assert_eq!(
            resolve(&ctx, TokenKind::Dimension, "Time"),
            resolved("ix://acme/own/type/Time")
        );
        assert_eq!(
            resolve(&ctx, TokenKind::Dimension, "acme/time-units::Time"),
            resolved("ix://acme/time-units/type/Time")
        );
    }

    #[trace("TC-1952", "FR-070-AC-14")]
    #[test]
    fn absent_imported_artifacts_leave_qualified_tokens_import_unresolved() {
        let ctx = context(json!({
            "package": "acme/own",
            "artifacts": [{ "id": "s", "object": "unit", "operations": [] }]
        }));
        assert_eq!(
            resolve(&ctx, TokenKind::Unit, "acme/time-units::ms"),
            unresolved(UnresolvedReason::ImportUnresolved)
        );
        assert_eq!(
            resolve(&ctx, TokenKind::Unit, "ms"),
            unresolved(UnresolvedReason::UnknownToken)
        );
    }

    #[trace("TC-1952", "FR-070-AC-14")]
    #[test]
    fn an_empty_index_has_the_single_reason_no_bundle_index() {
        let ctx = context(json!({}));
        for token in ["acme/time-units::ms", "acme/none::ms", "ms"] {
            assert_eq!(
                resolve(&ctx, TokenKind::Unit, token),
                unresolved(UnresolvedReason::NoBundleIndex)
            );
        }
    }

    #[trace("TC-1947", "FR-070-AC-13")]
    #[test]
    fn only_identifiers_and_org_repo_qualified_identifiers_parse() {
        for bad in [
            "",
            " ms ",
            "m-s",
            "a::b::c",
            "::ms",
            "acme::ms",
            "acme/rpc::",
            "Quantity<ms>",
        ] {
            assert_eq!(UnitToken::parse(bad), None, "{bad:?}");
        }
        assert_eq!(UnitToken::parse("ms"), Some(UnitToken::Bare("ms")));
        assert_eq!(
            UnitToken::parse("acme/rpc::ms"),
            Some(UnitToken::Qualified {
                package: "acme/rpc",
                name: "ms"
            })
        );
    }
}
