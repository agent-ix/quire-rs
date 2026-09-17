//! Reference-cell resolution shared by the FR-076 relationship targets and
//! the FR-075 systems-model tables: a bare artifact `id` or an
//! `ix://<org>/<repo>/<id>` identity, resolved against the extracted
//! artifact, the bundle's artifacts, and the module's imports.

use super::context::SemanticContext;
use crate::ix_ref::IxRef;

/// An id in the semantic-core `SemanticId` id alphabet: an ASCII letter or
/// digit, then ASCII letters, digits, `.`, `_`, `~`, `:`, or `-`.
pub(crate) fn is_id_segment(id: &str) -> bool {
    id.starts_with(|c: char| c.is_ascii_alphanumeric())
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._~:-".contains(c))
}

/// An object `id`: an ASCII letter, then ASCII letters, digits, or `_`.
pub(crate) fn is_object_id(id: &str) -> bool {
    id.starts_with(|c: char| c.is_ascii_alphabetic())
        && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// The extracted artifact's own `id` and object type: always a bundle
/// artifact (FR-104).
pub(crate) struct OwnArtifact<'a> {
    pub(crate) id: Option<&'a str>,
    pub(crate) object: Option<&'a str>,
}

/// A resolved reference cell.
pub(crate) enum Target<'a> {
    /// An artifact of the bundle, with its object type.
    Bundle {
        identity: String,
        object: Option<&'a str>,
    },
    /// An identity in an imported package; its object type is unknown.
    Imported { identity: String },
    /// An own-package identity with no bundle index to check it against.
    Unchecked { identity: String },
}

impl Target<'_> {
    pub(crate) fn identity(&self) -> &str {
        match self {
            Self::Bundle { identity, .. }
            | Self::Imported { identity }
            | Self::Unchecked { identity } => identity,
        }
    }
}

/// Why a reference cell did not resolve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Unresolved {
    /// Neither an id of the alphabet nor an `ix://<org>/<repo>/<id>`.
    Malformed,
    /// An `ix://` identity in a package the module does not import.
    UnimportedPackage,
    /// An own-package id no bundle artifact carries.
    UnknownId,
}

/// `ix://<org>/<repo>/<id>` with a `SemanticId` package and an `id` of
/// `alphabet`.
fn semantic_identity(cell: &str, alphabet: fn(&str) -> bool) -> Option<IxRef<'_>> {
    let identity = IxRef::parse(cell)?;
    let package_ok = |s: &str| {
        s.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
            && s.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "._-".contains(c))
    };
    (package_ok(identity.org()) && package_ok(identity.repo()) && alphabet(identity.path()))
        .then_some(identity)
}

/// Resolve `cell` under `package`: a bare id of `alphabet` qualifies under
/// `package`; an `ix://` identity in `package` resolves by its id, and one in
/// an imported package resolves unchecked. An own-package id resolves to the
/// artifact itself or a bundle artifact; with no bundle artifacts it lowers
/// unchecked.
pub(crate) fn resolve_target<'a>(
    ctx: &'a SemanticContext,
    package: &str,
    own: &OwnArtifact<'a>,
    cell: &str,
    alphabet: fn(&str) -> bool,
) -> Result<Target<'a>, Unresolved> {
    let (id, identity) = if cell.starts_with("ix://") {
        let parsed = semantic_identity(cell, alphabet).ok_or(Unresolved::Malformed)?;
        if parsed.package() != package {
            return if ctx.module.imports.contains_key(parsed.package()) {
                Ok(Target::Imported {
                    identity: cell.to_string(),
                })
            } else {
                Err(Unresolved::UnimportedPackage)
            };
        }
        (parsed.path(), cell.to_string())
    } else if alphabet(cell) {
        (cell, format!("ix://{package}/{cell}"))
    } else {
        return Err(Unresolved::Malformed);
    };
    let bundle_object = if own.id == Some(id) {
        Some(own.object)
    } else {
        ctx.bundle
            .artifacts
            .iter()
            .find(|a| a.id == id)
            .map(|a| a.object.as_deref())
    };
    match bundle_object {
        Some(object) => Ok(Target::Bundle { identity, object }),
        None if ctx.bundle.artifacts.is_empty() => Ok(Target::Unchecked { identity }),
        None => Err(Unresolved::UnknownId),
    }
}

#[cfg(test)]
mod tests {
    use ix_trace_rs::trace;

    use super::{is_id_segment, is_object_id};

    #[trace("TC-1872", "FR-075-AC-12")]
    #[test]
    fn object_ids_are_letters_digits_and_underscores() {
        assert!(is_object_id("quant_codec"));
        assert!(is_object_id("A1"));
        assert!(!is_object_id("quant-codec"));
        assert!(!is_object_id("1codec"));
        assert!(!is_object_id("_codec"));
        assert!(!is_object_id(""));
        assert!(is_id_segment("quant-codec"));
    }
}
