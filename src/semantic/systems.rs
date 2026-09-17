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
use super::target::{is_object_id, resolve_target, OwnArtifact, Target, Unresolved};
use super::SemanticDiagnostic;

/// One systems record at its declaring row. The record keys serialize flat
/// beside `sourceSpan`; the declaration record carries the keys alone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemsDecl<R> {
    /// The record keys.
    #[serde(flatten)]
    pub record: R,
    /// The span of the declaring row.
    pub source_span: SourceLocus,
}

/// A `Owner | Declared Type | Multiplicity` row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartRecord {
    /// The owning declaration's `SemanticId`.
    pub owner: String,
    /// The part's type.
    pub declared_type: TypeRef,
    /// How many of the part the owner holds.
    pub multiplicity: Multiplicity,
}

/// The closed set of port directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PortDirection {
    /// Flows enter the owner.
    In,
    /// Flows leave the owner.
    Out,
    /// Flows pass both ways.
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
    /// Which way the port passes flows.
    pub direction: PortDirection,
    /// The interface the port exposes.
    pub interface_type: TypeRef,
    /// How many of the port the owner holds.
    pub multiplicity: Multiplicity,
}

/// The closed set of connection flow directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConnectionDirection {
    /// From the source end to the target end.
    SourceToTarget,
    /// From the target end to the source end.
    TargetToSource,
    /// Both ways.
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
    /// The port's `SemanticId`.
    #[serde(rename = "type")]
    pub port: String,
    /// The end's multiplicity, when the row states one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multiplicity: Option<Multiplicity>,
}

/// A `Source | Source Multiplicity | Target | Target Multiplicity |
/// Direction` row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionRecord {
    /// The `Source` port end.
    pub source_end: ConnectionEnd,
    /// The `Target` port end.
    pub target_end: ConnectionEnd,
    /// The `Direction` of flow.
    pub flow_direction: ConnectionDirection,
}

/// A `Source | Target` row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationRecord {
    /// A part, a port, or an operation (`<id>/<operation>`).
    pub source_element: String,
    /// The part the source is allocated to.
    pub target_element: String,
}

/// The diagnostics the systems-model tables emit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SystemsFinding {
    /// A table with no row, an empty or malformed cell, or an unknown
    /// direction.
    InvalidCell,
    /// A row after the first.
    ExtraRow,
    /// A reference no bundle artifact or imported package carries.
    UnknownReference,
    /// A reference to an artifact of an object type the cell does not admit.
    WrongKind,
    /// A reference lowered without a bundle index to check it (advisory).
    UnresolvedTarget,
}

impl SystemsFinding {
    fn code(self) -> &'static str {
        match self {
            Self::InvalidCell => "semantic.invalid-model-cell",
            Self::ExtraRow => "semantic.duplicate-model-entry",
            Self::UnknownReference => "semantic.unknown-reference",
            Self::WrongKind => "semantic.reference-kind-mismatch",
            Self::UnresolvedTarget => "semantic.unresolved-target",
        }
    }
}

/// The role a reference cell plays, which fixes the form it admits and the
/// object types it may name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CellRole {
    /// A part's `Owner`.
    PartOwner,
    /// A port's `Owner`.
    PortOwner,
    /// A connection's `Source`.
    SourcePort,
    /// A connection's `Target`.
    TargetPort,
    /// An allocation's `Source`.
    SourceElement,
    /// An allocation's `Target`.
    TargetElement,
}

impl CellRole {
    fn label(self) -> &'static str {
        match self {
            Self::PartOwner | Self::PortOwner => "owner",
            Self::SourcePort => "source port",
            Self::TargetPort => "target port",
            Self::SourceElement => "source element",
            Self::TargetElement => "target element",
        }
    }

    /// Whether the cell may name `<id>/<member>`.
    fn admits_member(self) -> bool {
        match self {
            Self::SourceElement => true,
            Self::PartOwner
            | Self::PortOwner
            | Self::SourcePort
            | Self::TargetPort
            | Self::TargetElement => false,
        }
    }

    /// The object types the named artifact may have. A `<id>/<member>`
    /// operation reference names an `interface` and its operation.
    fn kinds(self, member: bool) -> &'static [&'static str] {
        match self {
            Self::PartOwner | Self::PortOwner | Self::TargetElement => &["part"],
            Self::SourcePort | Self::TargetPort => &["port"],
            Self::SourceElement if member => &["interface"],
            Self::SourceElement => &["part", "port"],
        }
    }
}

/// A resolved reference cell.
struct Reference {
    identity: String,
    /// Lowered without a bundle index; its advisory lands when the row does.
    unchecked: bool,
    role: CellRole,
}

impl<'a> TableRead<'a> {
    fn finding(&mut self, finding: SystemsFinding, line: usize, message: impl Into<String>) {
        self.error(finding.code(), line, message);
    }

    /// The one row of a single-row table: an empty table and every row after
    /// the first are errors.
    fn single_row(&mut self) -> Option<(usize, &'a [String])> {
        let table = self.table;
        let feature = self.feature.name();
        let Some((line, cells)) = table.rows.first() else {
            self.finding(
                SystemsFinding::InvalidCell,
                table.line,
                format!("a {feature} table declares no row; it declares exactly one"),
            );
            return None;
        };
        for (extra, _) in &table.rows[1..] {
            self.finding(
                SystemsFinding::ExtraRow,
                *extra,
                format!("a second {feature} row; a {feature} table declares exactly one"),
            );
        }
        (table.rows.len() == 1).then_some((*line, cells.as_slice()))
    }

    /// A reference cell resolved through the shared target resolver under
    /// the identity package, with the named artifact's object type checked
    /// against the cell's role.
    fn reference(&mut self, text: &str, role: CellRole, line: usize) -> Option<Reference> {
        let feature = self.feature.name();
        let what = role.label();
        if text.is_empty() {
            self.finding(
                SystemsFinding::InvalidCell,
                line,
                format!("a {feature} row names no {what}"),
            );
            return None;
        }
        let (base, member) = split_member(text, role.admits_member());
        if member.is_some_and(|m| !is_identifier(m)) {
            self.finding(
                SystemsFinding::InvalidCell,
                line,
                format!("{what} {text:?}: the member is not an Identifier"),
            );
            return None;
        }
        let ctx = self.ctx;
        let own_object = self.out.object.clone();
        let own = OwnArtifact {
            id: self.out.model.identity.as_ref().map(|d| d.value.as_str()),
            object: own_object.as_deref(),
        };
        let resolved = resolve_target(ctx, ctx.identity_package(), &own, base, is_object_id);
        let (identity, unchecked, object) = match resolved {
            Ok(Target::Bundle { identity, object }) => {
                (identity, false, Some(object.map(str::to_string)))
            }
            Ok(Target::Imported { identity }) => (identity, false, None),
            Ok(Target::Unchecked { identity }) => (identity, true, None),
            Err(Unresolved::Malformed) => {
                let expected = if role.admits_member() {
                    "an object id or `<id>/<member>`"
                } else {
                    "an object id"
                };
                self.finding(
                    SystemsFinding::InvalidCell,
                    line,
                    format!("{what} {text:?} is not {expected}"),
                );
                return None;
            }
            Err(Unresolved::UnknownId | Unresolved::UnimportedPackage) => {
                self.finding(
                    SystemsFinding::UnknownReference,
                    line,
                    format!("{what} {base} names no artifact of the bundle or an imported package"),
                );
                return None;
            }
        };
        if role == CellRole::PartOwner
            && own
                .id
                .is_some_and(|id| identity == format!("ix://{}/{id}", ctx.identity_package()))
        {
            self.finding(
                SystemsFinding::WrongKind,
                line,
                format!("{what} {base} is the part itself; a part cannot own itself"),
            );
            return None;
        }
        if let Some(object) = object {
            let kinds = role.kinds(member.is_some());
            if !object.as_deref().is_some_and(|o| kinds.contains(&o)) {
                let found = object.as_deref().unwrap_or("no object type");
                self.finding(
                    SystemsFinding::WrongKind,
                    line,
                    format!("{what} {base} is a {found}, not a {}", kinds.join(" or ")),
                );
                return None;
            }
        }
        let identity = match member {
            Some(member) => format!("{identity}/{member}"),
            None => identity,
        };
        Some(Reference {
            identity,
            unchecked,
            role,
        })
    }

    /// Emit the `no-bundle-index` advisories of a row that lowered, and hand
    /// back the identities.
    fn lowered<const N: usize>(&mut self, line: usize, refs: [Reference; N]) -> [String; N] {
        refs.map(|r| {
            if r.unchecked {
                self.out.diagnostics.push(
                    SemanticDiagnostic::new(
                        SystemsFinding::UnresolvedTarget.code(),
                        SemanticSeverity::Advisory,
                        line,
                        format!(
                            "{} {} is not checked against the bundle: this surface supplies no bundle index",
                            r.role.label(),
                            r.identity
                        ),
                    )
                    .with_reason("no-bundle-index"),
                );
            }
            r.identity
        })
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
                self.finding(SystemsFinding::InvalidCell, line, unknown.to_string());
                None
            }
        }
    }
}

/// Split `<id>/<member>` when the cell admits a member: a bare cell at its
/// first `/`, an `ix://<org>/<repo>/<id>/<member>` identity at its last.
fn split_member(text: &str, admits_member: bool) -> (&str, Option<&str>) {
    if !admits_member {
        return (text, None);
    }
    let split = if text.starts_with("ix://") {
        text.rsplit_once('/')
            .filter(|(base, _)| base.matches('/').count() >= 4)
    } else {
        text.split_once('/')
    };
    match split {
        Some((base, member)) => (base, Some(member)),
        None => (text, None),
    }
}

pub(super) fn read_part(r: &mut TableRead<'_>) {
    let Some((line, cells)) = r.single_row() else {
        return;
    };
    let owner = r.reference(r.cell(cells, "Owner"), CellRole::PartOwner, line);
    let declared_type = r.type_cell(r.cell(cells, "Declared Type"), line);
    let multiplicity = r.multiplicity(r.cell(cells, "Multiplicity"), line);
    let (Some(owner), Some(declared_type), Some(multiplicity)) =
        (owner, declared_type, multiplicity)
    else {
        return;
    };
    let [owner] = r.lowered(line, [owner]);
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
    let owner = r.reference(r.cell(cells, "Owner"), CellRole::PortOwner, line);
    let direction = r.closed::<PortDirection>(r.cell(cells, "Direction"), line);
    let interface_type = r.type_cell(r.cell(cells, "Interface"), line);
    let multiplicity = r.multiplicity(r.cell(cells, "Multiplicity"), line);
    let (Some(owner), Some(direction), Some(interface_type), Some(multiplicity)) =
        (owner, direction, interface_type, multiplicity)
    else {
        return;
    };
    let [owner] = r.lowered(line, [owner]);
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
    let source = r.reference(r.cell(cells, "Source"), CellRole::SourcePort, line);
    let source_mult = r.end_multiplicity(r.cell(cells, "Source Multiplicity"), line);
    let target = r.reference(r.cell(cells, "Target"), CellRole::TargetPort, line);
    let target_mult = r.end_multiplicity(r.cell(cells, "Target Multiplicity"), line);
    let flow_direction = r.closed::<ConnectionDirection>(r.cell(cells, "Direction"), line);
    let (Some(source), Some(source_mult), Some(target), Some(target_mult), Some(flow_direction)) =
        (source, source_mult, target, target_mult, flow_direction)
    else {
        return;
    };
    let [source, target] = r.lowered(line, [source, target]);
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
    let source = r.reference(r.cell(cells, "Source"), CellRole::SourceElement, line);
    let target = r.reference(r.cell(cells, "Target"), CellRole::TargetElement, line);
    let (Some(source), Some(target)) = (source, target) else {
        return;
    };
    let [source_element, target_element] = r.lowered(line, [source, target]);
    r.out.model.allocation = Some(SystemsDecl {
        record: AllocationRecord {
            source_element,
            target_element,
        },
        source_span: r.span(line),
    });
}
