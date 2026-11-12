//! Reference-cell resolution shared by the FR-076 relationship targets and
//! the FR-075 systems-model tables: a bare artifact `id` or an
//! `ix://<org>/<repo>/<id>` identity, resolved against the extracted
//! artifact, the bundle's artifacts, and the module's imports. FR-075
//! declaration references ([`resolve_reference`]) add the name form: an item
//! matched by `id`, `title` or `name`, filtered by the admitted object types.

use std::collections::BTreeMap;

use super::context::SemanticContext;
use super::relations::RelationVocabulary;
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

/// The extracted artifact's own `id`, object type, and declared operation
/// names: always a bundle artifact (FR-104). `operations` is the artifact's
/// own `## Operations` names — empty where the caller's role never admits
/// `<id>/<member>` (every relationship `Target`), so a self-reference
/// `<own-id>/<member>` (an allocation `Source` naming the artifact itself)
/// checks `<member>` against real operations rather than always refusing.
pub(crate) struct OwnArtifact<'a> {
    pub(crate) id: Option<&'a str>,
    pub(crate) object: Option<&'a str>,
    pub(crate) operations: &'a [String],
}

/// A resolved reference cell.
pub(crate) enum Target<'a> {
    /// An artifact of the bundle, with its object type and the operation
    /// names it declares under `## Operations` (FR-075 Inputs).
    Bundle {
        identity: String,
        object: Option<&'a str>,
        operations: &'a [String],
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
    // Package names compare ASCII case-insensitively (FR-076-AC-23), so an
    // uppercase spelling of the bundle's own package must parse to be matched.
    let package_ok = |s: &str| {
        s.starts_with(|c: char| c.is_ascii_alphanumeric())
            && s.chars()
                .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
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
        if !parsed.package().eq_ignore_ascii_case(package) {
            return if ctx.module.imports.contains_key(parsed.package()) {
                Ok(Target::Imported {
                    identity: cell.to_string(),
                })
            } else {
                Err(Unresolved::UnimportedPackage)
            };
        }
        (parsed.path(), format!("ix://{package}/{}", parsed.path()))
    } else if alphabet(cell) {
        (cell, format!("ix://{package}/{cell}"))
    } else {
        return Err(Unresolved::Malformed);
    };
    let bundle_match: Option<(Option<&str>, &[String])> = if own.id == Some(id) {
        Some((own.object, own.operations))
    } else {
        ctx.bundle
            .artifacts
            .iter()
            .find(|a| a.id == id)
            .map(|a| (a.object.as_deref(), a.operations.as_slice()))
    };
    match bundle_match {
        Some((object, operations)) => Ok(Target::Bundle {
            identity,
            object,
            operations,
        }),
        None if ctx.bundle.artifacts.is_empty() => Ok(Target::Unchecked { identity }),
        None => Err(Unresolved::UnknownId),
    }
}

/// The extracted artifact's own frontmatter `id`, `title`, `name` and
/// `object`: always a declaration of the bundle (FR-075 Declaration
/// references).
#[derive(Debug, Clone, Default)]
pub(crate) struct OwnDeclaration {
    pub(crate) id: Option<String>,
    pub(crate) title: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) object: Option<String>,
}

impl OwnDeclaration {
    /// The declaration the document's frontmatter states.
    pub(crate) fn from_raw(raw: &str) -> Self {
        let fm = crate::parser::frontmatter::extract_frontmatter_ref(raw);
        let field = |key: &str| {
            fm.frontmatter
                .as_ref()
                .and_then(|m| m.get(key))
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        };
        Self {
            id: field("id"),
            title: field("title"),
            name: field("name"),
            object: field("object"),
        }
    }
}

/// One declaration of the bundle: its `id`, every name it is matched by
/// (`id`, `title`, `name`), and its object type.
struct Declaration {
    id: String,
    names: Vec<String>,
    object: Option<String>,
}

/// The object types a reference cell admits, from the first module source
/// that declares the cell (FR-075 Admission).
#[derive(Clone, Copy)]
pub(crate) enum Admission<'a> {
    /// The construct `references` entry for the cell's member: role tokens.
    Construct {
        roles: &'a [String],
        vocabulary: &'a RelationVocabulary,
    },
    /// The `allowed_links` entry for the column's verb: `*`, object type or
    /// role tokens.
    AllowedLinks {
        tokens: &'a [String],
        vocabulary: &'a RelationVocabulary,
    },
    /// Neither source declares the cell: every declaration is admitted.
    Any,
}

impl Admission<'_> {
    /// Whether a declaration of object type `object` is admitted. A
    /// declaration with no object type satisfies only `*` or an unrestricted
    /// cell.
    fn admits(self, object: Option<&str>) -> bool {
        match self {
            Self::Construct { roles, vocabulary } => roles.iter().any(|role| {
                role == "*" || object.is_some_and(|o| vocabulary.roles_of(o).contains(role))
            }),
            Self::AllowedLinks { tokens, vocabulary } => tokens
                .iter()
                .any(|token| vocabulary.satisfies(token, object)),
            Self::Any => true,
        }
    }
}

/// Why a reference item did not resolve to exactly one declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReferenceRefusal {
    /// The item names no declaration at all.
    UnknownName,
    /// The item names declarations, none of an admitted object type.
    KindMismatch,
    /// The item names more than one admitted declaration: their ids.
    Ambiguous(Vec<String>),
}

/// The bundle's declarations: the extracted artifact itself, then every
/// `BundleIndex.artifacts` entry joined to its `BundleIndex.objects` names by
/// `id`. One declaration authored in two forms under one `id` is one
/// declaration with the union of its names.
pub(crate) struct Declarations(BTreeMap<String, Declaration>);

impl Declarations {
    pub(crate) fn new(ctx: &SemanticContext, own: &OwnDeclaration) -> Self {
        let mut names_by_id: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for entry in &ctx.bundle.objects {
            names_by_id
                .entry(entry.id.as_str())
                .or_default()
                .extend(entry.names.iter().map(String::as_str));
        }
        let mut out = Self(BTreeMap::new());
        if let Some(id) = own.id.as_deref() {
            let names = [own.title.as_deref(), own.name.as_deref()]
                .into_iter()
                .flatten();
            out.add(id, names, own.object.as_deref());
        }
        for artifact in &ctx.bundle.artifacts {
            let names = names_by_id
                .get(artifact.id.as_str())
                .into_iter()
                .flatten()
                .copied();
            out.add(&artifact.id, names, artifact.object.as_deref());
        }
        out
    }

    fn add<'n>(&mut self, id: &str, names: impl Iterator<Item = &'n str>, object: Option<&str>) {
        let entry = self.0.entry(id.to_string()).or_insert_with(|| Declaration {
            id: id.to_string(),
            names: vec![id.to_string()],
            object: None,
        });
        for name in names {
            if !entry.names.iter().any(|n| n == name) {
                entry.names.push(name.to_string());
            }
        }
        if entry.object.is_none() {
            entry.object = object.map(str::to_string);
        }
    }
}

/// Resolve one FR-075 reference item under `package` to the `SemanticId` of
/// the one admitted declaration it names. The caller has established that
/// the bundle is indexed. An `ix://<org>/<repo>/<id>` item resolves as an
/// FR-076 `Target` does, never by name; an identity in an imported package
/// is admitted unchecked.
pub(crate) fn resolve_reference(
    ctx: &SemanticContext,
    package: &str,
    own: &OwnDeclaration,
    declarations: &Declarations,
    item: &str,
    admission: Admission<'_>,
) -> Result<String, ReferenceRefusal> {
    if item.starts_with("ix://") {
        let own_artifact = OwnArtifact {
            id: own.id.as_deref(),
            object: own.object.as_deref(),
            operations: &[],
        };
        return match resolve_target(ctx, package, &own_artifact, item, is_id_segment) {
            Ok(Target::Bundle {
                identity, object, ..
            }) => {
                if admission.admits(object) {
                    Ok(identity)
                } else {
                    Err(ReferenceRefusal::KindMismatch)
                }
            }
            Ok(Target::Imported { identity } | Target::Unchecked { identity }) => Ok(identity),
            Err(_) => Err(ReferenceRefusal::UnknownName),
        };
    }
    let candidates: Vec<&Declaration> = declarations
        .0
        .values()
        .filter(|d| d.names.iter().any(|n| n == item))
        .collect();
    if candidates.is_empty() {
        return Err(ReferenceRefusal::UnknownName);
    }
    let admitted: Vec<&Declaration> = candidates
        .into_iter()
        .filter(|d| admission.admits(d.object.as_deref()))
        .collect();
    match admitted.as_slice() {
        [] => Err(ReferenceRefusal::KindMismatch),
        [one] => Ok(format!("ix://{package}/{}", one.id)),
        many => Err(ReferenceRefusal::Ambiguous(
            many.iter().map(|d| d.id.clone()).collect(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use ix_trace_rs::trace;

    use super::{is_id_segment, is_object_id};

    #[trace("FR-075-AC-12")]
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
