//! Combinatorial obligations from declared configuration dimensions (FR-061).
//!
//! Config-space bugs hide in interactions no single-dimension test exercises,
//! and *"we tested the configurations"* is unquantifiable without a declared
//! space. This mints the quantity: given the dimensions a spec declares, how
//! many t-way value combinations exist to be covered.
//!
//! **The engine derives the number; it neither generates nor runs the
//! combinations** (ADR-0011 invariant 1). A covering-array skeleton is a
//! Generator's job and is not scoped here.
//!
//! ## What the number counts
//!
//! For strength `t`, the obligation is over every **t-way value tuple**: for
//! each set of `t` distinct dimensions, the product of their value counts,
//! summed over all such sets. For dimensions of sizes 2, 3 and 2 at `t = 2`
//! that is `2·3 + 2·2 + 3·2 = 16` pairs.
//!
//! That is the standard t-way denominator, and deliberately **not** the size of
//! a covering array. The minimum array size is NP-hard to compute and depends
//! on the generator; the number of tuples to cover is a property of the
//! declared space alone, which is what an obligation must be able to restate
//! from the spec at any later time.

use std::collections::BTreeSet;

/// One declared configuration dimension and the values it takes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dimension {
    pub name: String,
    pub values: Vec<String>,
}

/// A forbidden combination: value assignments that cannot co-occur.
///
/// Real configuration spaces have them — a feature unavailable on a target, a
/// codec absent from a build — and a covering array over an unconstrained
/// product demands combinations that cannot exist. Counting those as
/// obligations would make the target permanently unreachable, which is the
/// fastest way to get a coverage number ignored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exclusion {
    /// `(dimension, value)` pairs that together are forbidden.
    pub assignments: Vec<(String, String)>,
}

/// The declared space, parsed and validated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigurationSpace {
    pub dimensions: Vec<Dimension>,
    pub exclusions: Vec<Exclusion>,
}

impl ConfigurationSpace {
    /// Number of t-way value tuples the space contains, excluding any that a
    /// declared exclusion forbids.
    ///
    /// Returns 0 when `strength` exceeds the number of dimensions: there is no
    /// 3-way interaction among two dimensions, and reporting one would demand
    /// coverage of combinations that do not exist.
    pub fn tuples(&self, strength: usize) -> usize {
        if strength == 0 || strength > self.dimensions.len() {
            return 0;
        }
        let mut total = 0usize;
        for combo in index_combinations(self.dimensions.len(), strength) {
            total += self.tuples_over(&combo);
        }
        total
    }

    /// t-way tuples over one specific set of dimensions.
    fn tuples_over(&self, dims: &[usize]) -> usize {
        let mut assignment = vec![0usize; dims.len()];
        let mut count = 0usize;
        loop {
            if !self.forbidden(dims, &assignment) {
                count += 1;
            }
            // Odometer over the chosen dimensions' value indices.
            let mut carry = dims.len();
            while carry > 0 {
                carry -= 1;
                assignment[carry] += 1;
                if assignment[carry] < self.dimensions[dims[carry]].values.len() {
                    break;
                }
                assignment[carry] = 0;
                if carry == 0 {
                    return count;
                }
            }
            if dims.is_empty() {
                return count;
            }
        }
    }

    /// Whether an assignment contains every pair of some declared exclusion.
    ///
    /// A *superset* match, not equality: an exclusion naming two values
    /// forbids every wider tuple containing both, which is what makes a
    /// two-value constraint meaningful at strength 3.
    fn forbidden(&self, dims: &[usize], assignment: &[usize]) -> bool {
        let chosen: Vec<(&str, &str)> = dims
            .iter()
            .enumerate()
            .map(|(slot, &d)| {
                (
                    self.dimensions[d].name.as_str(),
                    self.dimensions[d].values[assignment[slot]].as_str(),
                )
            })
            .collect();
        self.exclusions.iter().any(|exclusion| {
            exclusion
                .assignments
                .iter()
                .all(|(dim, value)| chosen.iter().any(|(d, v)| d == dim && v == value))
        })
    }

    /// The canonical statement this space hashes to.
    ///
    /// Every declared value appears, in declared order, so **any change to the
    /// space changes the hash** and every binding over it becomes suspect.
    /// That is the entire suspect-link mechanism, inherited rather than
    /// reinvented: adding a value to a dimension really does invalidate a
    /// coverage claim made before it existed.
    pub fn statement(&self, strength: usize) -> String {
        let mut out = format!("{strength}-way over");
        for dimension in &self.dimensions {
            out.push_str(&format!(
                " {}({})",
                dimension.name,
                dimension.values.join("|")
            ));
        }
        for exclusion in &self.exclusions {
            let pairs: Vec<String> = exclusion
                .assignments
                .iter()
                .map(|(d, v)| format!("{d}={v}"))
                .collect();
            out.push_str(&format!(" excluding[{}]", pairs.join(",")));
        }
        out
    }
}

/// Every `size`-element subset of `0..n`, in lexicographic order.
fn index_combinations(n: usize, size: usize) -> Vec<Vec<usize>> {
    if size > n {
        return Vec::new();
    }
    let mut current: Vec<usize> = (0..size).collect();
    let mut out = vec![current.clone()];
    loop {
        let mut i = size;
        loop {
            if i == 0 {
                return out;
            }
            i -= 1;
            if current[i] != i + n - size {
                break;
            }
            if i == 0 {
                return out;
            }
        }
        current[i] += 1;
        for j in i + 1..size {
            current[j] = current[j - 1] + 1;
        }
        out.push(current.clone());
    }
}

/// Split a declared cell into values: comma-separated, trimmed, de-duplicated.
///
/// De-duplication is not tidying. A dimension listing the same value twice
/// would inflate every tuple count it participates in, so the obligation would
/// demand coverage of combinations that do not exist — the same defect a
/// forbidden combination causes, arriving by typo instead of by design.
pub fn split_values(cell: &str) -> Vec<String> {
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut out = Vec::new();
    for value in cell.split(',') {
        let value = value.trim().trim_matches('`').trim();
        if value.is_empty() || !seen.insert(value) {
            continue;
        }
        out.push(value.to_string());
    }
    out
}

/// Parse one exclusion cell: `dim=value & dim=value`, or empty for none.
pub fn parse_exclusion(cell: &str) -> Option<Exclusion> {
    let mut assignments = Vec::new();
    for clause in cell.split('&') {
        let clause = clause.trim().trim_matches('`').trim();
        if clause.is_empty() {
            continue;
        }
        let (dim, value) = clause.split_once('=')?;
        let (dim, value) = (dim.trim(), value.trim());
        if dim.is_empty() || value.is_empty() {
            return None;
        }
        assignments.push((dim.to_string(), value.to_string()));
    }
    // A single assignment is not an interaction constraint — it says a value is
    // never used, which is a shorter values list. Rejecting it here keeps the
    // two ways of saying that from disagreeing.
    (assignments.len() >= 2).then_some(Exclusion { assignments })
}
