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
use crate::vocab::{AllowedLinks, EdgeCategory};

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

impl From<&crate::vocab::EdgeTypeDef> for EdgeVerb {
    fn from(def: &crate::vocab::EdgeTypeDef) -> Self {
        Self {
            category: def.category,
            inverse: def.inverse.clone(),
        }
    }
}

/// The registry and object-type facts relationships extraction checks rows
/// against (FR-076 Inputs). An empty vocabulary registers no verb.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RelationVocabulary {
    /// The object type the artifact is extracted under.
    #[serde(default)]
    pub object_type: Option<String>,
    /// The merged `edge_types` registry: forward verb → entry.
    #[serde(default)]
    pub edge_types: BTreeMap<String, EdgeVerb>,
    /// Object type → the roles it carries.
    #[serde(default)]
    pub roles: BTreeMap<String, Vec<String>>,
    /// Object type → its `allowed_links`.
    #[serde(default)]
    pub allowed_links: BTreeMap<String, AllowedLinks>,
}

impl RelationVocabulary {
    /// The forward verb whose registry entry declares `label` as its
    /// inverse, when `label` is not itself a forward verb (FR-041).
    fn forward_of_inverse(&self, label: &str) -> Option<&str> {
        if self.edge_types.contains_key(label) {
            return None;
        }
        self.edge_types
            .iter()
            .find(|(_, entry)| entry.inverse.as_deref() == Some(label))
            .map(|(verb, _)| verb.as_str())
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
        if token == "*" {
            return true;
        }
        let Some(object) = target_object else {
            return false;
        };
        token == object
            || self
                .roles
                .get(object)
                .is_some_and(|roles| roles.iter().any(|r| r == token))
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

/// Every refusal FR-104 names; each variant fixes its code and `reason`.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Refusal {
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
}

impl Refusal {
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
        }
    }

    /// What is wrong, naming the offending cells of `row` when there is one.
    fn detail(&self, ctx: &SemanticContext, row: Option<&Row<'_>>) -> String {
        let (name, verb, target) = row.map_or(("", "", ""), |r| (r.name, r.verb, r.target));
        let object_type = ctx
            .relation_vocabulary
            .object_type
            .as_deref()
            .unwrap_or("<none>");
        match self {
            Self::NotExtractable => {
                if ctx.module.mappings.iter().any(|m| m == RELATIONSHIPS_TOKEN) {
                    format!(
                        "the section holds no `{}` table",
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
            Self::InverseVerb { forward } => format!(
                "`{verb}` is the inverse label of `{forward}`; declare `{forward}` on {target}, the artifact that declares the edge"
            ),
            Self::Generalization => format!(
                "`{SPECIALIZES}` declares a specialization; declare it through the `generalization` mapping (frontmatter `relationships` with `type: {SPECIALIZES}`)"
            ),
            Self::UnknownVerb => format!("verb `{verb}` is not in the edge_types registry"),
            Self::VerbNotAllowed => {
                format!("verb `{verb}` is not in the allowed_links of object type {object_type}")
            }
            Self::TargetNotId => format!(
                "target {target:?} is not an artifact id of the bundle or an identity of the bundle or an imported package"
            ),
            Self::TargetNotAllowed { object } => format!(
                "target {target} (object type {}) satisfies no allowed_links token of `{verb}` for object type {object_type}",
                object.as_deref().unwrap_or("<none>")
            ),
            Self::Multiplicity { why } => format!("row {name}: {why}"),
            Self::DeclaredInFrontmatter => format!(
                "`{verb}` {target} is also declared in frontmatter `relationships`; declare it once"
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
            SemanticSeverity::Error,
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

/// A resolved `Target` cell.
enum Target<'v> {
    /// An artifact of the bundle, with its object type.
    Bundle {
        identity: String,
        object: Option<&'v str>,
    },
    /// An identity in an imported package; its object type is unknown.
    Imported { identity: String },
}

impl Target<'_> {
    fn identity(&self) -> &str {
        match self {
            Self::Bundle { identity, .. } | Self::Imported { identity } => identity,
        }
    }
}

/// `(org/repo, id)` of `ix://<org>/<repo>/<id>`, each part in the
/// semantic-core `SemanticId` alphabet.
fn split_identity(cell: &str) -> Option<(&str, &str)> {
    let rest = cell.strip_prefix("ix://")?;
    let (package, id) = rest.rsplit_once('/')?;
    let (org, repo) = package.split_once('/')?;
    let package_ok = |s: &str| {
        s.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
            && s.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "._-".contains(c))
    };
    let id_ok = id.starts_with(|c: char| c.is_ascii_alphanumeric())
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._~:-".contains(c));
    (package_ok(org) && package_ok(repo) && id_ok).then_some((package, id))
}

fn resolve_target<'c>(ctx: &'c SemanticContext, cell: &str) -> Option<Target<'c>> {
    let package = ctx.identity_package();
    let bundle_object = |id: &str| {
        ctx.bundle
            .artifacts
            .iter()
            .find(|a| a.id == id)
            .map(|a| a.object.as_deref())
    };
    if !cell.starts_with("ix://") {
        let object = bundle_object(cell)?;
        return Some(Target::Bundle {
            identity: format!("ix://{package}/{cell}"),
            object,
        });
    }
    let (target_package, id) = split_identity(cell)?;
    if target_package == package {
        let object = bundle_object(id)?;
        Some(Target::Bundle {
            identity: cell.to_string(),
            object,
        })
    } else if ctx.module.imports.contains_key(target_package) {
        Some(Target::Imported {
            identity: cell.to_string(),
        })
    } else {
        None
    }
}

/// The `(verb, identity)` of every frontmatter `relationships` entry: a bare
/// id under the bundle's package, an `ix://<org>/<repo>/…/<id>` path reduced
/// to `ix://<org>/<repo>/<id>`, an absent `type` read as `references`.
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
            let identity = match target.strip_prefix("ix://") {
                None => format!("ix://{package}/{target}"),
                Some(rest) => {
                    let mut parts = rest.split('/');
                    let (org, repo) = (parts.next()?, parts.next()?);
                    let id = parts.next_back()?;
                    format!("ix://{org}/{repo}/{id}")
                }
            };
            Some((verb.to_string(), identity))
        })
        .collect()
}

/// Row reader state: names seen, frontmatter edges.
struct RowCheck<'c> {
    ctx: &'c SemanticContext,
    names: Vec<String>,
    frontmatter: Vec<(String, String)>,
}

impl RowCheck<'_> {
    /// FR-104 row checks in order; the first failure is the row's refusal.
    fn check(&mut self, row: &Row<'_>, lines: &[&str]) -> Result<ExtractedRelation, Refusal> {
        let vocabulary = &self.ctx.relation_vocabulary;
        if !is_identifier(row.name) {
            return Err(Refusal::NameNotIdentifier);
        }
        if self.names.iter().any(|n| n == row.name) {
            return Err(Refusal::DuplicateName);
        }
        self.names.push(row.name.to_string());
        if let Some(forward) = vocabulary.forward_of_inverse(row.verb) {
            return Err(Refusal::InverseVerb {
                forward: forward.to_string(),
            });
        }
        if row.verb == SPECIALIZES {
            return Err(Refusal::Generalization);
        }
        let Some(entry) = vocabulary.edge_types.get(row.verb) else {
            return Err(Refusal::UnknownVerb);
        };
        let Some(allowed) = vocabulary.allowed_targets(row.verb) else {
            return Err(Refusal::VerbNotAllowed);
        };
        let target = resolve_target(self.ctx, row.target).ok_or(Refusal::TargetNotId)?;
        if let Target::Bundle { object, .. } = &target {
            if !allowed
                .iter()
                .any(|token| vocabulary.satisfies(token, *object))
            {
                return Err(Refusal::TargetNotAllowed {
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
            return Err(Refusal::DeclaredInFrontmatter);
        }
        Ok(ExtractedRelation {
            name: row.name.to_string(),
            decl: RelationDecl {
                verb: row.verb.to_string(),
                category: entry.category,
                composite: entry.inverse.as_deref() == Some(COMPOSITE_INVERSE),
                target: identity.to_string(),
                multiplicity,
            },
            source_span: line_span(self.ctx, lines, row.line),
        })
    }
}

/// A required FR-070 multiplicity cell.
fn parse_multiplicity(row: &Row<'_>) -> Result<Multiplicity, Refusal> {
    if row.multiplicity.is_empty() {
        return Err(Refusal::Multiplicity {
            why: "the Multiplicity cell is empty".to_string(),
        });
    }
    let mut found = Vec::new();
    map_multiplicity(row.multiplicity, row.line, &mut found).ok_or_else(|| Refusal::Multiplicity {
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

/// FR-076 over one document.
pub(crate) fn extract_relations(raw: &str, ctx: &SemanticContext) -> RelationsOutcome {
    let lines = lines(raw);
    let fm = crate::parser::frontmatter::extract_frontmatter_ref(raw);
    let body_start = raw[..raw.len().saturating_sub(fm.body.len())]
        .split('\n')
        .count()
        .max(1);
    let headings = level2_headings(&lines);
    let mut out = RelationsOutcome::default();

    if !ctx.module.mappings.iter().any(|m| m == RELATIONSHIPS_TOKEN) {
        let preamble_end = headings.first().map_or(lines.len() + 1, |h| h.1);
        let regions = std::iter::once((Section::Preamble, body_start, preamble_end)).chain(
            headings
                .iter()
                .map(|(h, start, end)| (Section::Heading(h), start + 1, *end)),
        );
        for (section, from, to) in regions {
            for block in blocks_in(&lines, from, to) {
                if let Block::Table(table) = block {
                    if is_relationship_header(&table.headers) {
                        out.diagnostics.push(
                            Refusal::NotExtractable
                                .diagnostic(ctx, &lines, section, table.line, None),
                        );
                    }
                }
            }
        }
        out.availability = (!out.diagnostics.is_empty()).then(|| entry_errors(&out.diagnostics));
        return out;
    }

    let section = Section::Heading(SECTION);
    let mut sections = headings.iter().filter(|(h, _, _)| h == SECTION);
    let Some((_, start, end)) = sections.next() else {
        out.availability = Some(KindAvailability::not_applicable());
        return out;
    };
    for (_, line, _) in sections {
        out.diagnostics
            .push(Refusal::SecondSection.diagnostic(ctx, &lines, section, *line, None));
    }
    let blocks = blocks_in(&lines, start + 1, *end);
    let mut read: Option<&Table> = None;
    let mut seen_table = false;
    for block in &blocks {
        let (refusal, line) = match block {
            Block::Table(table) if seen_table => (Refusal::SecondTable, table.line),
            Block::Table(table) => {
                seen_table = true;
                if is_exact_header(table) {
                    read = Some(table);
                    continue;
                }
                (Refusal::NotExtractable, table.line)
            }
            Block::Fence(fence) => (Refusal::NotExtractable, fence.open_line),
            Block::List { line } => (Refusal::NotExtractable, *line),
        };
        out.diagnostics
            .push(refusal.diagnostic(ctx, &lines, section, line, None));
    }
    if blocks.is_empty() && out.diagnostics.is_empty() {
        out.availability = Some(KindAvailability::not_applicable());
        return out;
    }

    let mut rows = Vec::new();
    if let Some(table) = read {
        let mut check = RowCheck {
            ctx,
            names: Vec::new(),
            frontmatter: frontmatter_edges(fm.frontmatter.as_ref(), ctx.identity_package()),
        };
        for (line, cells) in &table.rows {
            let row = Row::new(*line, cells);
            match check.check(&row, &lines) {
                Ok(relation) => rows.push(relation),
                Err(refusal) => out.diagnostics.push(refusal.diagnostic(
                    ctx,
                    &lines,
                    section,
                    row.line,
                    Some(&row),
                )),
            }
        }
    }
    if out.diagnostics.iter().any(SemanticDiagnostic::is_error) {
        out.availability = Some(entry_errors(&out.diagnostics));
    } else {
        out.availability = Some(KindAvailability::available(false));
        out.relations = Some(rows);
    }
    out
}
