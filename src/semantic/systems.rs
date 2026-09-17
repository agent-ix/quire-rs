//! The systems-model tables of FR-075 (QSpec FR-152): `part`, `port`,
//! `connection`, and `allocation`. Each object type declares its members as
//! the one row of one table; the row lowers to the record keys its data
//! schema requires, carried on `model` with the row span and merged into the
//! declaration record.

use serde::{Deserialize, Serialize};

use super::clauses::SourceLocus;
use super::contract::SemanticSeverity;
use super::decl::{is_identifier, Multiplicity, TypeRef};
use super::model::TableRead;
use super::properties::{map_multiplicity, map_type, RowInput};
use super::relations::is_id_segment;
use super::SemanticDiagnostic;

/// One systems record at its declaring row. The record keys serialize flat
/// beside `sourceSpan`; the declaration record carries the keys alone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemsDecl<R> {
    #[serde(flatten)]
    pub record: R,
    pub source_span: SourceLocus,
}

/// A `Owner | Declared Type | Multiplicity` row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartRecord {
    /// The owning declaration's `SemanticId`.
    pub owner: String,
    pub declared_type: TypeRef,
    pub multiplicity: Multiplicity,
}

/// The closed set of port directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PortDirection {
    In,
    Out,
    Inout,
}

/// A `Direction` cell of a port that names no [`PortDirection`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("port direction {0:?} is not in, out, or inout")]
pub struct UnknownPortDirection(pub String);

impl std::str::FromStr for PortDirection {
    type Err = UnknownPortDirection;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "in" => Ok(Self::In),
            "out" => Ok(Self::Out),
            "inout" => Ok(Self::Inout),
            other => Err(UnknownPortDirection(other.to_string())),
        }
    }
}

/// A `Owner | Direction | Interface | Multiplicity` row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortRecord {
    /// The owning part's `SemanticId`.
    pub owner: String,
    pub direction: PortDirection,
    pub interface_type: TypeRef,
    pub multiplicity: Multiplicity,
}

/// The closed set of connection flow directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConnectionDirection {
    SourceToTarget,
    TargetToSource,
    Bidirectional,
}

/// A `Direction` cell of a connection that names no [`ConnectionDirection`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("connection direction {0:?} is not source-to-target, target-to-source, or bidirectional")]
pub struct UnknownConnectionDirection(pub String);

impl std::str::FromStr for ConnectionDirection {
    type Err = UnknownConnectionDirection;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "source-to-target" => Ok(Self::SourceToTarget),
            "target-to-source" => Ok(Self::TargetToSource),
            "bidirectional" => Ok(Self::Bidirectional),
            other => Err(UnknownConnectionDirection(other.to_string())),
        }
    }
}

/// One end of a connection: the port's `SemanticId` and, when the end
/// states one, its multiplicity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionEnd {
    #[serde(rename = "type")]
    pub port: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multiplicity: Option<Multiplicity>,
}

/// A `Source | Source Multiplicity | Target | Target Multiplicity |
/// Direction` row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionRecord {
    pub source_end: ConnectionEnd,
    pub target_end: ConnectionEnd,
    pub flow_direction: ConnectionDirection,
}

/// A `Source | Target` row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationRecord {
    /// A part, a port, or an operation (`<id>/<operation>`).
    pub source_element: String,
    pub target_element: String,
}

/// Which references a cell admits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RefForm {
    /// An artifact `id`.
    Artifact,
    /// An artifact `id`, or `<id>/<member>` naming one of its members.
    ArtifactOrMember,
}

impl<'a> TableRead<'a> {
    /// The one row of a single-row table: an empty table and every row after
    /// the first are errors.
    fn single_row(&mut self) -> Option<(usize, &'a [String])> {
        let table = self.table;
        let feature = self.feature.name();
        let Some((line, cells)) = table.rows.first() else {
            self.error(
                "semantic.invalid-model-cell",
                table.line,
                format!("a {feature} table declares no row; it declares exactly one"),
            );
            return None;
        };
        for (extra, _) in &table.rows[1..] {
            self.error(
                "semantic.duplicate-model-entry",
                *extra,
                format!("a second {feature} row; a {feature} table declares exactly one"),
            );
        }
        (table.rows.len() == 1).then_some((*line, cells.as_slice()))
    }

    /// A reference cell lowered to its `SemanticId` under the identity
    /// package. The artifact `id` resolves against the bundle's artifacts
    /// (or the artifact itself); a bundle with no artifacts lowers it
    /// unchecked with a `no-bundle-index` advisory.
    fn reference(&mut self, text: &str, what: &str, line: usize, form: RefForm) -> Option<String> {
        let feature = self.feature.name();
        if text.is_empty() {
            self.error(
                "semantic.invalid-model-cell",
                line,
                format!("a {feature} row names no {what}"),
            );
            return None;
        }
        let (id, member) = match (form, text.split_once('/')) {
            (RefForm::ArtifactOrMember, Some((id, member))) => (id, Some(member)),
            _ => (text, None),
        };
        if !is_id_segment(id) || member.is_some_and(|m| !is_identifier(m)) {
            let expected = match form {
                RefForm::Artifact => "an artifact id",
                RefForm::ArtifactOrMember => "an artifact id or `<id>/<member>`",
            };
            self.error(
                "semantic.invalid-model-cell",
                line,
                format!("{what} {text:?} is not {expected}"),
            );
            return None;
        }
        let identity = format!("ix://{}/{text}", self.ctx.identity_package());
        let own = self.out.model.identity.as_ref().map(|d| d.value.as_str());
        let bundle = &self.ctx.bundle.artifacts;
        if own == Some(id) || bundle.iter().any(|a| a.id == id) {
            return Some(identity);
        }
        if bundle.is_empty() {
            self.out.diagnostics.push(
                SemanticDiagnostic::new(
                    "semantic.unresolved-target",
                    SemanticSeverity::Advisory,
                    line,
                    format!(
                        "{what} {id} is not checked against the bundle: this surface supplies no bundle index"
                    ),
                )
                .with_reason("no-bundle-index"),
            );
            return Some(identity);
        }
        self.error(
            "semantic.unknown-reference",
            line,
            format!("{what} {id} names no artifact of the bundle"),
        );
        None
    }

    /// A `Declared Type` or `Interface` cell, mapped as the FR-070 type cell.
    fn type_cell(&mut self, text: &str, line: usize) -> Option<TypeRef> {
        let row = RowInput {
            line,
            name: text.to_string(),
            type_cell: text.to_string(),
            mult_cell: String::new(),
            constraints_cell: String::new(),
            reference_only: false,
        };
        let (type_ref, lossy) = map_type(text, &row, self.ctx, &mut self.out.diagnostics);
        self.out.lossy |= lossy;
        type_ref
    }

    /// A required multiplicity cell, mapped as the FR-070 multiplicity cell.
    fn multiplicity(&mut self, text: &str, line: usize) -> Option<Multiplicity> {
        map_multiplicity(text, line, &mut self.out.diagnostics)
    }

    /// A connection end multiplicity: an empty cell states none.
    fn end_multiplicity(&mut self, text: &str, line: usize) -> Option<Option<Multiplicity>> {
        if text.is_empty() {
            return Some(None);
        }
        self.multiplicity(text, line).map(Some)
    }

    /// A closed-set cell: the parsed value, or `semantic.invalid-model-cell`
    /// carrying the parse error.
    fn closed<T>(&mut self, text: &str, line: usize) -> Option<T>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
    {
        match text.parse::<T>() {
            Ok(value) => Some(value),
            Err(unknown) => {
                self.error("semantic.invalid-model-cell", line, unknown.to_string());
                None
            }
        }
    }
}

pub(super) fn read_part(r: &mut TableRead<'_>) {
    let Some((line, cells)) = r.single_row() else {
        return;
    };
    let owner = r.reference(r.cell(cells, "Owner"), "owner", line, RefForm::Artifact);
    let declared_type = r.type_cell(r.cell(cells, "Declared Type"), line);
    let multiplicity = r.multiplicity(r.cell(cells, "Multiplicity"), line);
    let (Some(owner), Some(declared_type), Some(multiplicity)) =
        (owner, declared_type, multiplicity)
    else {
        return;
    };
    r.out.model.part = Some(SystemsDecl {
        record: PartRecord {
            owner,
            declared_type,
            multiplicity,
        },
        source_span: r.span(line),
    });
}

pub(super) fn read_port(r: &mut TableRead<'_>) {
    let Some((line, cells)) = r.single_row() else {
        return;
    };
    let owner = r.reference(r.cell(cells, "Owner"), "owner", line, RefForm::Artifact);
    let direction = r.closed::<PortDirection>(r.cell(cells, "Direction"), line);
    let interface_type = r.type_cell(r.cell(cells, "Interface"), line);
    let multiplicity = r.multiplicity(r.cell(cells, "Multiplicity"), line);
    let (Some(owner), Some(direction), Some(interface_type), Some(multiplicity)) =
        (owner, direction, interface_type, multiplicity)
    else {
        return;
    };
    r.out.model.port = Some(SystemsDecl {
        record: PortRecord {
            owner,
            direction,
            interface_type,
            multiplicity,
        },
        source_span: r.span(line),
    });
}

pub(super) fn read_connection(r: &mut TableRead<'_>) {
    let Some((line, cells)) = r.single_row() else {
        return;
    };
    let source = r.reference(
        r.cell(cells, "Source"),
        "source port",
        line,
        RefForm::Artifact,
    );
    let source_mult = r.end_multiplicity(r.cell(cells, "Source Multiplicity"), line);
    let target = r.reference(
        r.cell(cells, "Target"),
        "target port",
        line,
        RefForm::Artifact,
    );
    let target_mult = r.end_multiplicity(r.cell(cells, "Target Multiplicity"), line);
    let flow_direction = r.closed::<ConnectionDirection>(r.cell(cells, "Direction"), line);
    let (Some(source), Some(source_mult), Some(target), Some(target_mult), Some(flow_direction)) =
        (source, source_mult, target, target_mult, flow_direction)
    else {
        return;
    };
    r.out.model.connection = Some(SystemsDecl {
        record: ConnectionRecord {
            source_end: ConnectionEnd {
                port: source,
                multiplicity: source_mult,
            },
            target_end: ConnectionEnd {
                port: target,
                multiplicity: target_mult,
            },
            flow_direction,
        },
        source_span: r.span(line),
    });
}

pub(super) fn read_allocation(r: &mut TableRead<'_>) {
    let Some((line, cells)) = r.single_row() else {
        return;
    };
    let source = r.reference(
        r.cell(cells, "Source"),
        "source element",
        line,
        RefForm::ArtifactOrMember,
    );
    let target = r.reference(
        r.cell(cells, "Target"),
        "target element",
        line,
        RefForm::Artifact,
    );
    let (Some(source_element), Some(target_element)) = (source, target) else {
        return;
    };
    r.out.model.allocation = Some(SystemsDecl {
        record: AllocationRecord {
            source_element,
            target_element,
        },
        source_span: r.span(line),
    });
}
