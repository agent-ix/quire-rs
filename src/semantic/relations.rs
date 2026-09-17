//! Relationships extraction (FR-076, quoin FR-104): the `## Relationships`
//! `Name | Verb | Target | Multiplicity` table to semantic-core
//! `RelationDecl[]`, gated by the `relationships` mapping token and checked
//! against the FR-040 edge registry the caller supplies.
//!
//! Semantic-core `0.2.0` has no `name` or `sourceSpan` on `RelationDecl`, so
//! each row is extracted as an [`ExtractedRelation`] and split into the
//! `relations` / `relationSources` pair only at the record boundary
//! ([`ExtractedRelation::split`]). When `agent-ix/filament-core-data#155`
//! moves both onto `RelationDecl`, that split is the one place to change.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::clauses::SourceLocus;
use super::context::SemanticContext;
use super::contract::SemanticSeverity;
use super::decl::{is_identifier, Multiplicity};
use super::model::{entry_errors, line_span, Section};
use super::properties::{map_multiplicity, strip_ticks};
use super::scan::{blocks_in, level2_headings, lines, Block, Table};
use super::{KindAvailability, SemanticDiagnostic};
use crate::ix_ref::IxRef;
use crate::loader::compile::CompiledArchetype;
use crate::registry::Registry;
use crate::vocab::{target_satisfies, AllowedLinks, EdgeCategory, EdgeTypeDef, InverseIndex};

/// The `semantic.mappings` token that declares relationships extraction.
pub const RELATIONSHIPS_TOKEN: &str = "relationships";

/// The section heading the table is read under.
const SECTION: &str = "Relationships";

/// The one table header, in order.
const COLUMNS: [&str; 4] = ["Name", "Verb", "Target", "Multiplicity"];

/// The verb whose declaration belongs to the FR-075 `generalization` mapping.
const SPECIALIZES: &str = "specializes";

/// The registry inverse that makes a verb composite.
const COMPOSITE_INVERSE: &str = "part_of";

/// One verb of the merged `edge_types` registry as relationships
/// extraction reads it (FR-040): its category and optional inverse label.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EdgeVerb {
    pub category: EdgeCategory,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inverse: Option<String>,
}

impl From<&EdgeTypeDef> for EdgeVerb {
    fn from(def: &EdgeTypeDef) -> Self {
        Self {
            category: def.category,
            inverse: def.inverse.clone(),
        }
    }
}

/// The registry and object-type facts relationships extraction checks rows
/// against (FR-076 Inputs). A surface that supplies none extracts under
/// `no-relation-vocabulary`; it is never an empty vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(from = "VocabularyWire")]
pub struct RelationVocabulary {
    object_type: Option<String>,
    edge_types: BTreeMap<String, EdgeVerb>,
    roles: BTreeMap<String, Vec<String>>,
    allowed_links: BTreeMap<String, AllowedLinks>,
    /// Inverse label → forward verb, with the FR-041 precedence of
    /// [`Registry::inverse_index`].
    inverse_forwards: BTreeMap<String, String>,
}

/// The JSON form of a [`RelationVocabulary`] (`relationVocabulary`).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct VocabularyWire {
    #[serde(default)]
    object_type: Option<String>,
    #[serde(default)]
    edge_types: BTreeMap<String, EdgeVerb>,
    #[serde(default)]
    roles: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    allowed_links: BTreeMap<String, AllowedLinks>,
}

impl From<VocabularyWire> for RelationVocabulary {
    fn from(wire: VocabularyWire) -> Self {
        Self::new(
            wire.object_type,
            wire.edge_types,
            wire.roles,
            wire.allowed_links,
        )
    }
}

impl RelationVocabulary {
    /// A vocabulary for the object type `object_type`: the merged
    /// `edge_types` registry, each object type's `roles`, and each object
    /// type's `allowed_links`. The inverse index is derived as the loader
    /// derives [`Registry::inverse_index`].
    pub fn new(
        object_type: Option<String>,
        edge_types: BTreeMap<String, EdgeVerb>,
        roles: BTreeMap<String, Vec<String>>,
        allowed_links: BTreeMap<String, AllowedLinks>,
    ) -> Self {
        let inverse_forwards =
            InverseIndex::derive(&edge_types, |entry| entry.inverse.as_deref()).forwards;
        Self {
            object_type,
            edge_types,
            roles,
            allowed_links,
            inverse_forwards,
        }
    }

    /// The vocabulary a loaded registry declares for artifacts of the object
    /// type `object_type`: its merged `edge_types` and inverse index, and the
    /// `roles` and `allowed_links` of every active archetype.
    pub fn from_registry(registry: &Registry, object_type: &CompiledArchetype) -> Self {
        let mut roles = BTreeMap::new();
        let mut allowed_links = BTreeMap::new();
        for arch in registry.active_archetypes() {
            roles.insert(arch.name.clone(), arch.roles().to_vec());
            allowed_links.insert(arch.name.clone(), arch.allowed_links().clone());
        }
        Self {
            object_type: Some(object_type.name.clone()),
            edge_types: registry
                .edge_types()
                .iter()
                .map(|(verb, def)| (verb.clone(), EdgeVerb::from(def)))
                .collect(),
            roles,
            allowed_links,
            inverse_forwards: registry.inverse_index().clone(),
        }
    }

    /// The forward verb whose registry entry declares `label` as its
    /// inverse, when `label` is not itself a forward verb (FR-041).
    fn forward_of_inverse(&self, label: &str) -> Option<&str> {
        self.inverse_forwards.get(label).map(String::as_str)
    }

    /// The target tokens the extracting object type allows for `verb`.
    fn allowed_targets(&self, verb: &str) -> Option<&[String]> {
        let object_type = self.object_type.as_deref()?;
        self.allowed_links
            .get(object_type)?
            .get(verb)
            .map(Vec::as_slice)
    }

    /// FR-040 `target_satisfies`, except that an untyped target satisfies
    /// only `*` (FR-104 Rows).
    fn satisfies(&self, token: &str, target_object: Option<&str>) -> bool {
        match target_object {
            None => token == "*",
            Some(object) => target_satisfies(
                token,
                object,
                self.roles.get(object).map_or(&[], Vec::as_slice),
            ),
        }
    }
}

/// A semantic-core `0.2.0` `RelationDecl`. Field order is the canonical key
/// order; `composite` and `multiplicity` are always present (FR-104).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationDecl {
    pub verb: String,
    pub category: EdgeCategory,
    pub composite: bool,
    pub target: String,
    pub multiplicity: Multiplicity,
}

/// The `name` and `sourceSpan` of one `relations` element, at its index:
/// the semantic-core `0.2.0` carrier until `RelationDecl` holds them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationSource {
    pub name: String,
    pub source_span: SourceLocus,
}

/// One extracted row: the declaration with its name and span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExtractedRelation {
    pub(crate) name: String,
    pub(crate) decl: RelationDecl,
    pub(crate) source_span: SourceLocus,
}

impl ExtractedRelation {
    /// The record-boundary shape: parallel `relations` and `relationSources`.
    pub(crate) fn split(rows: Vec<Self>) -> (Vec<RelationDecl>, Vec<RelationSource>) {
        rows.into_iter()
            .map(|row| {
                (
                    row.decl,
                    RelationSource {
                        name: row.name,
                        source_span: row.source_span,
                    },
                )
            })
            .unzip()
    }
}

/// What relationships extraction produced for one artifact.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct RelationsOutcome {
    /// Present exactly when `availability` is `available`.
    pub(crate) relations: Option<Vec<ExtractedRelation>>,
    /// Absent when the artifact neither names the token nor holds a
    /// relationship-shaped table.
    pub(crate) availability: Option<KindAvailability>,
    pub(crate) diagnostics: Vec<SemanticDiagnostic>,
}

/// Every diagnostic FR-104 names; each variant fixes its code, severity, and
/// `reason`.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Finding {
    /// A relationship table or block the manifest or grammar does not admit.
    NotExtractable,
    SecondTable,
    SecondSection,
    NameNotIdentifier,
    DuplicateName,
    InverseVerb {
        forward: String,
    },
    Generalization,
    UnknownVerb,
    VerbNotAllowed,
    TargetNotId,
    TargetNotAllowed {
        object: Option<String>,
    },
    Multiplicity {
        why: String,
    },
    DeclaredInFrontmatter,
    /// A `## Relationships` section with no block (warning).
    NoBlock,
    /// The surface supplies no relation vocabulary (advisory).
    NoVocabulary,
    /// The surface supplies no bundle package to qualify targets under
    /// (advisory).
    NoBundlePackage,
    /// A row target lowered without a bundle index to check it (advisory).
    UnresolvedTarget,
}

impl Finding {
    fn code(&self) -> &'static str {
        match self {
            Self::NotExtractable => "semantic.feature-not-extractable",
            Self::SecondTable | Self::SecondSection => "semantic.duplicate-section",
            Self::DuplicateName | Self::DeclaredInFrontmatter => "semantic.duplicate-model-entry",
            Self::NameNotIdentifier
            | Self::InverseVerb { .. }
            | Self::Generalization
            | Self::UnknownVerb
            | Self::VerbNotAllowed
            | Self::TargetNotId
            | Self::TargetNotAllowed { .. }
            | Self::Multiplicity { .. } => "semantic.invalid-model-cell",
            Self::NoBlock => "semantic.relationships-no-block",
            Self::NoVocabulary => "semantic.relationships-no-vocabulary",
            Self::NoBundlePackage => "semantic.relationships-no-bundle-package",
            Self::UnresolvedTarget => "semantic.unresolved-target",
        }
    }

    fn severity(&self) -> SemanticSeverity {
        match self {
            Self::NoBlock => SemanticSeverity::Warning,
            Self::NoVocabulary | Self::NoBundlePackage | Self::UnresolvedTarget => {
                SemanticSeverity::Advisory
            }
            Self::NotExtractable
            | Self::SecondTable
            | Self::SecondSection
            | Self::NameNotIdentifier
            | Self::DuplicateName
            | Self::InverseVerb { .. }
            | Self::Generalization
            | Self::UnknownVerb
            | Self::VerbNotAllowed
            | Self::TargetNotId
            | Self::TargetNotAllowed { .. }
            | Self::Multiplicity { .. }
            | Self::DeclaredInFrontmatter => SemanticSeverity::Error,
        }
    }

    fn reason(&self) -> &'static str {
        match self {
            Self::NotExtractable => RELATIONSHIPS_TOKEN,
            Self::SecondTable => "second-table",
            Self::SecondSection => "second-section",
            Self::NameNotIdentifier => "name-not-identifier",
            Self::DuplicateName => "duplicate-name",
            Self::InverseVerb { .. } => "inverse-verb",
            Self::Generalization => "generalization",
            Self::UnknownVerb => "unknown-verb",
            Self::VerbNotAllowed => "verb-not-allowed",
            Self::TargetNotId => "target-not-id",
            Self::TargetNotAllowed { .. } => "target-not-allowed",
            Self::Multiplicity { .. } => "multiplicity",
            Self::DeclaredInFrontmatter => "declared-in-frontmatter",
            Self::NoBlock => "no-block",
            Self::NoVocabulary => NO_VOCABULARY,
            Self::NoBundlePackage => NO_BUNDLE_PACKAGE,
            Self::UnresolvedTarget => "no-bundle-index",
        }
    }

    /// What is wrong, naming the offending cells of `row` when there is one.
    fn detail(&self, ctx: &SemanticContext, row: Option<&Row<'_>>) -> String {
        let (name, verb, target) = row.map_or(("", "", ""), |r| (r.name, r.verb, r.target));
        let object_type = ctx
            .relation_vocabulary
            .as_ref()
            .and_then(|v| v.object_type.as_deref())
            .unwrap_or("<none>");
        match self {
            Self::NotExtractable => {
                if ctx.module.mappings.iter().any(|m| m == RELATIONSHIPS_TOKEN) {
                    format!(
                        "relationships are declared only as one `{}` table under `## {SECTION}`",
                        COLUMNS.join(" | ")
                    )
                } else {
                    format!(
                        "the module {} manifest does not declare the `{RELATIONSHIPS_TOKEN}` mapping",
                        ctx.module.package
                    )
                }
            }
            Self::SecondTable => "a second table; one per artifact".to_string(),
            Self::SecondSection => "a second `## Relationships` section".to_string(),
            Self::NameNotIdentifier => format!("name {name:?} is not an Identifier"),
            Self::DuplicateName => format!("name {name} is declared twice"),
            Self::InverseVerb { forward } if forward == SPECIALIZES => format!(
                "`{verb}` is the inverse label of `{SPECIALIZES}`; {target} declares the specialization through the FR-075 `generalization` mapping (frontmatter `relationships` with `type: {SPECIALIZES}`)"
            ),
            Self::InverseVerb { forward } => format!(
                "`{verb}` is the inverse label of `{forward}`; declare `{forward}` on {target}, the artifact that declares the edge"
            ),
            Self::Generalization => format!(
                "`{SPECIALIZES}` is not a relationship row; declare the specialization through the FR-075 `generalization` mapping (frontmatter `relationships` with `type: {SPECIALIZES}`)"
            ),
            Self::UnknownVerb => format!("verb `{verb}` is not in the edge_types registry"),
            Self::VerbNotAllowed => {
                format!("verb `{verb}` is not in the allowed_links of object type {object_type}")
            }
            Self::TargetNotId => format!(
                "target {target:?} is not a bundle artifact id, an `ix://<org>/<repo>/<id>` identity of a bundle artifact, or an identity in an imported package"
            ),
            Self::TargetNotAllowed { object } => format!(
                "target {target} (object type {}) satisfies no allowed_links token of `{verb}` for object type {object_type}",
                object.as_deref().unwrap_or("<none>")
            ),
            Self::Multiplicity { why } => format!("row {name}: {why}"),
            Self::DeclaredInFrontmatter => format!(
                "`{verb}` {target} is also declared in frontmatter `relationships`; declare it once"
            ),
            Self::NoBlock => format!(
                "the section holds no `{}` table; nothing is extracted",
                COLUMNS.join(" | ")
            ),
            Self::NoVocabulary => "this surface supplies no relation vocabulary (edge_types, roles, allowed_links); the rows are not checked or extracted".to_string(),
            Self::NoBundlePackage => "this surface supplies no bundle package (`<org>/<repo>`) to qualify targets under; the rows are not checked or extracted".to_string(),
            Self::UnresolvedTarget => format!(
                "target {target} is not checked against the bundle: this surface supplies no bundle index"
            ),
        }
    }

    fn diagnostic(
        &self,
        ctx: &SemanticContext,
        lines: &[&str],
        section: Section<'_>,
        line: usize,
        row: Option<&Row<'_>>,
    ) -> SemanticDiagnostic {
        let mut d = SemanticDiagnostic::new(
            self.code(),
            self.severity(),
            line,
            format!(
                "artifact {} declares {RELATIONSHIPS_TOKEN} in {} at line {line}: {}",
                ctx.path,
                section.display(),
                self.detail(ctx, row)
            ),
        )
        .with_reason(self.reason());
        d.source_span = Some(line_span(ctx, lines, line));
        d.section = Some(section.label());
        d
    }
}

/// The `availability.relations` reason of a surface with no vocabulary.
const NO_VOCABULARY: &str = "no-relation-vocabulary";

/// The `availability.relations` reason of a surface with no bundle package.
const NO_BUNDLE_PACKAGE: &str = "no-bundle-package";

/// The `<org>/<repo>` targets qualify under: the bundle's package, else the
/// package of an explicit `ix://<org>/<repo>/…` source identity. Never the
/// module's package: that names the module, not the artifact's bundle.
fn bundle_package(ctx: &SemanticContext) -> Option<&str> {
    if !ctx.bundle.package.is_empty() {
        return Some(&ctx.bundle.package);
    }
    ctx.source_identity
        .as_deref()
        .and_then(IxRef::parse)
        .map(|identity| identity.package())
}

/// The stripped cells of one table row.
struct Row<'a> {
    line: usize,
    name: &'a str,
    verb: &'a str,
    target: &'a str,
    multiplicity: &'a str,
}

impl<'a> Row<'a> {
    fn new(line: usize, cells: &'a [String]) -> Self {
        let cell = |i: usize| cells.get(i).map_or("", |c| strip_ticks(c));
        Self {
            line,
            name: cell(0),
            verb: cell(1),
            target: cell(2),
            multiplicity: cell(3),
        }
    }
}

/// The extracted artifact's own `id` and object type: always a bundle
/// artifact (FR-104).
struct OwnArtifact<'a> {
    id: Option<&'a str>,
    object: Option<&'a str>,
}

/// A resolved `Target` cell.
enum Target<'a> {
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
    fn identity(&self) -> &str {
        match self {
            Self::Bundle { identity, .. }
            | Self::Imported { identity }
            | Self::Unchecked { identity } => identity,
        }
    }
}

/// An id in the semantic-core `SemanticId` id alphabet: an ASCII letter or
/// digit, then ASCII letters, digits, `.`, `_`, `~`, `:`, or `-`.
fn is_id_segment(id: &str) -> bool {
    id.starts_with(|c: char| c.is_ascii_alphanumeric())
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._~:-".contains(c))
}

/// `ix://<org>/<repo>/<id>` with a `SemanticId` package and id.
fn semantic_identity(cell: &str) -> Option<IxRef<'_>> {
    let identity = IxRef::parse(cell)?;
    let package_ok = |s: &str| {
        s.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
            && s.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "._-".contains(c))
    };
    (package_ok(identity.org()) && package_ok(identity.repo()) && is_id_segment(identity.path()))
        .then_some(identity)
}

fn resolve_target<'a>(
    ctx: &'a SemanticContext,
    package: &str,
    own: &OwnArtifact<'a>,
    cell: &str,
) -> Option<Target<'a>> {
    let (id, identity) =
        if cell.starts_with("ix://") {
            let parsed = semantic_identity(cell)?;
            if parsed.package() != package {
                return ctx.module.imports.contains_key(parsed.package()).then(|| {
                    Target::Imported {
                        identity: cell.to_string(),
                    }
                });
            }
            (parsed.path(), cell.to_string())
        } else if is_id_segment(cell) {
            (cell, format!("ix://{package}/{cell}"))
        } else {
            return None;
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
        Some(object) => Some(Target::Bundle { identity, object }),
        None if ctx.bundle.artifacts.is_empty() => Some(Target::Unchecked { identity }),
        None => None,
    }
}

/// The `(verb, identity)` of every frontmatter `relationships` entry: a bare
/// id under the bundle's package, an `ix://<org>/<repo>/…/<id>` path reduced
/// to `ix://<org>/<repo>/<id>` (FR-026), an absent `type` read as
/// `references`.
fn frontmatter_edges(
    frontmatter: Option<&serde_json::Map<String, Value>>,
    package: &str,
) -> Vec<(String, String)> {
    let Some(entries) = frontmatter
        .and_then(|fm| fm.get("relationships"))
        .and_then(Value::as_array)
    else {
        return Vec::new();
    };
    entries
        .iter()
        .filter_map(|entry| {
            let target = entry.get("target").and_then(Value::as_str)?;
            let verb = entry
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("references");
            let identity = if target.starts_with("ix://") {
                let parsed = IxRef::parse(target)?;
                format!("ix://{}/{}", parsed.package(), parsed.last_segment())
            } else {
                format!("ix://{package}/{target}")
            };
            Some((verb.to_string(), identity))
        })
        .collect()
}

/// A row that passed every check.
struct Lowered {
    relation: ExtractedRelation,
    /// Lowered without a bundle index to check its target.
    unchecked: bool,
}

/// Row reader state: names seen, frontmatter edges.
struct RowCheck<'a> {
    ctx: &'a SemanticContext,
    vocabulary: &'a RelationVocabulary,
    package: &'a str,
    own: OwnArtifact<'a>,
    names: Vec<String>,
    frontmatter: Vec<(String, String)>,
}

impl RowCheck<'_> {
    /// FR-104 row checks in order; the first failure is the row's refusal.
    fn check(&mut self, row: &Row<'_>, lines: &[&str]) -> Result<Lowered, Finding> {
        let vocabulary = self.vocabulary;
        if !is_identifier(row.name) {
            return Err(Finding::NameNotIdentifier);
        }
        if self.names.iter().any(|n| n == row.name) {
            return Err(Finding::DuplicateName);
        }
        self.names.push(row.name.to_string());
        if let Some(forward) = vocabulary.forward_of_inverse(row.verb) {
            return Err(Finding::InverseVerb {
                forward: forward.to_string(),
            });
        }
        if row.verb == SPECIALIZES {
            return Err(Finding::Generalization);
        }
        let Some(entry) = vocabulary.edge_types.get(row.verb) else {
            return Err(Finding::UnknownVerb);
        };
        let Some(allowed) = vocabulary.allowed_targets(row.verb) else {
            return Err(Finding::VerbNotAllowed);
        };
        let target = resolve_target(self.ctx, self.package, &self.own, row.target)
            .ok_or(Finding::TargetNotId)?;
        if let Target::Bundle { object, .. } = &target {
            if !allowed
                .iter()
                .any(|token| vocabulary.satisfies(token, *object))
            {
                return Err(Finding::TargetNotAllowed {
                    object: object.map(str::to_string),
                });
            }
        }
        let multiplicity = parse_multiplicity(row)?;
        let identity = target.identity();
        if self
            .frontmatter
            .iter()
            .any(|(verb, fm_target)| verb == row.verb && fm_target == identity)
        {
            return Err(Finding::DeclaredInFrontmatter);
        }
        Ok(Lowered {
            relation: ExtractedRelation {
                name: row.name.to_string(),
                decl: RelationDecl {
                    verb: row.verb.to_string(),
                    category: entry.category,
                    composite: entry.inverse.as_deref() == Some(COMPOSITE_INVERSE),
                    target: identity.to_string(),
                    multiplicity,
                },
                source_span: line_span(self.ctx, lines, row.line),
            },
            unchecked: matches!(target, Target::Unchecked { .. }),
        })
    }
}

/// A required FR-070 multiplicity cell.
fn parse_multiplicity(row: &Row<'_>) -> Result<Multiplicity, Finding> {
    if row.multiplicity.is_empty() {
        return Err(Finding::Multiplicity {
            why: "the Multiplicity cell is empty".to_string(),
        });
    }
    let mut found = Vec::new();
    map_multiplicity(row.multiplicity, row.line, &mut found).ok_or_else(|| Finding::Multiplicity {
        why: found
            .into_iter()
            .next()
            .map_or_else(|| "malformed multiplicity".to_string(), |d| d.message),
    })
}

fn is_relationship_header(headers: &[String]) -> bool {
    headers
        .first()
        .is_some_and(|h| strip_ticks(h) == COLUMNS[0])
        && headers.iter().all(|h| COLUMNS.contains(&strip_ticks(h)))
}

fn is_exact_header(table: &Table) -> bool {
    table.headers.len() == COLUMNS.len()
        && table
            .headers
            .iter()
            .zip(COLUMNS)
            .all(|(h, c)| strip_ticks(h) == c)
}

/// `unavailable` with `entry-errors` when any diagnostic is an error, else
/// `otherwise`.
fn settle(diagnostics: &[SemanticDiagnostic], otherwise: KindAvailability) -> KindAvailability {
    if diagnostics.iter().any(SemanticDiagnostic::is_error) {
        entry_errors(diagnostics)
    } else {
        otherwise
    }
}

/// FR-076 over one document.
pub(crate) fn extract_relations(raw: &str, ctx: &SemanticContext) -> RelationsOutcome {
    let lines = lines(raw);
    let fm = crate::parser::frontmatter::extract_frontmatter_ref(raw);
    let body_start = raw[..raw.len().saturating_sub(fm.body.len())]
        .split('\n')
        .count()
        .max(1);
    let headings = level2_headings(&lines);
    let declared = ctx.module.mappings.iter().any(|m| m == RELATIONSHIPS_TOKEN);
    let mut out = RelationsOutcome::default();

    // FR-075 refusal rule: a relationship-shaped table anywhere the feature
    // is not declared; under the token, anywhere but `## Relationships`.
    let preamble_end = headings.first().map_or(lines.len() + 1, |h| h.1);
    let regions = std::iter::once((Section::Preamble, body_start, preamble_end)).chain(
        headings
            .iter()
            .filter(|(h, _, _)| !(declared && h == SECTION))
            .map(|(h, start, end)| (Section::Heading(h), start + 1, *end)),
    );
    for (region, from, to) in regions {
        for block in blocks_in(&lines, from, to) {
            if let Block::Table(table) = block {
                if is_relationship_header(&table.headers) {
                    out.diagnostics.push(
                        Finding::NotExtractable.diagnostic(ctx, &lines, region, table.line, None),
                    );
                }
            }
        }
    }
    if !declared {
        out.availability = (!out.diagnostics.is_empty()).then(|| entry_errors(&out.diagnostics));
        return out;
    }

    let section = Section::Heading(SECTION);
    let mut sections = headings.iter().filter(|(h, _, _)| h == SECTION);
    let Some((_, heading, end)) = sections.next() else {
        out.availability = Some(settle(&out.diagnostics, KindAvailability::not_applicable()));
        return out;
    };
    for (_, line, _) in sections {
        out.diagnostics
            .push(Finding::SecondSection.diagnostic(ctx, &lines, section, *line, None));
    }
    let blocks = blocks_in(&lines, heading + 1, *end);
    if blocks.is_empty() {
        out.diagnostics
            .push(Finding::NoBlock.diagnostic(ctx, &lines, section, *heading, None));
        out.availability = Some(settle(&out.diagnostics, KindAvailability::not_applicable()));
        return out;
    }
    let mut read: Option<&Table> = None;
    let mut seen_table = false;
    for block in &blocks {
        let (refusal, line) = match block {
            Block::Table(table) if seen_table => (Finding::SecondTable, table.line),
            Block::Table(table) => {
                seen_table = true;
                if is_exact_header(table) {
                    read = Some(table);
                    continue;
                }
                (Finding::NotExtractable, table.line)
            }
            Block::Fence(fence) => (Finding::NotExtractable, fence.open_line),
            Block::List { line } => (Finding::NotExtractable, *line),
        };
        out.diagnostics
            .push(refusal.diagnostic(ctx, &lines, section, line, None));
    }
    let has_errors = out.diagnostics.iter().any(SemanticDiagnostic::is_error);
    let Some(table) = read else {
        out.availability = Some(entry_errors(&out.diagnostics));
        return out;
    };
    let Some(vocabulary) = ctx.relation_vocabulary.as_ref() else {
        // A shape error wins over the missing vocabulary (FR-104).
        out.availability = Some(if has_errors {
            entry_errors(&out.diagnostics)
        } else {
            out.diagnostics
                .push(Finding::NoVocabulary.diagnostic(ctx, &lines, section, *heading, None));
            KindAvailability::unavailable(NO_VOCABULARY)
        });
        return out;
    };
    let Some(package) = bundle_package(ctx) else {
        // As R1: a shape error wins over the missing package.
        out.availability = Some(if has_errors {
            entry_errors(&out.diagnostics)
        } else {
            out.diagnostics
                .push(Finding::NoBundlePackage.diagnostic(ctx, &lines, section, *heading, None));
            KindAvailability::unavailable(NO_BUNDLE_PACKAGE)
        });
        return out;
    };

    let frontmatter = fm.frontmatter.as_ref();
    let own_str = |key: &str| frontmatter.and_then(|m| m.get(key)).and_then(Value::as_str);
    let mut check = RowCheck {
        ctx,
        vocabulary,
        package,
        own: OwnArtifact {
            id: own_str("id"),
            object: own_str("object"),
        },
        names: Vec::new(),
        frontmatter: frontmatter_edges(frontmatter, package),
    };
    let mut rows = Vec::new();
    // `no-bundle-index` advisories land only when no row errs (FR-104).
    let mut unresolved = Vec::new();
    for (line, cells) in &table.rows {
        let row = Row::new(*line, cells);
        match check.check(&row, &lines) {
            Ok(lowered) => {
                if lowered.unchecked {
                    unresolved.push(Finding::UnresolvedTarget.diagnostic(
                        ctx,
                        &lines,
                        section,
                        row.line,
                        Some(&row),
                    ));
                }
                rows.push(lowered.relation);
            }
            Err(refusal) => {
                out.diagnostics
                    .push(refusal.diagnostic(ctx, &lines, section, row.line, Some(&row)))
            }
        }
    }
    if out.diagnostics.iter().any(SemanticDiagnostic::is_error) {
        out.availability = Some(entry_errors(&out.diagnostics));
    } else {
        out.diagnostics.extend(unresolved);
        out.availability = Some(KindAvailability::available(false));
        out.relations = Some(rows);
    }
    out
}
