//! The metric provenance envelope (FR-063).
//!
//! Every number this engine emits states what it counts, over what population,
//! by what method, and how much of its input the measurement actually read.
//!
//! ## Why this exists
//!
//! `quire coverage` reported `555/2389 rows backed (23%)` over
//! `agent-ix/filament-ide-rs` while its declared tag patterns matched **0 of
//! 1,292** source symbols. The arithmetic was correct and the number was
//! meaningless, and nothing in the payload distinguished the two. Three
//! published SpecReviews were built on it.
//!
//! [`crate::coverage`]'s binding census (FR-050-AC-27) fixed that one case by
//! naming the one premise it happened to be about. This module generalizes it
//! into a schema invariant: **a percentage cannot be emitted without its match
//! counts**, so the hollow-denominator shape is structurally visible rather
//! than something a reader has to already suspect.
//!
//! ## Not computed is a value
//!
//! An empty list and an uncomputed one serialize identically — the key is
//! absent either way — so a consumer could not tell "the engine looked and
//! found none" from "the engine never looked" (`agent-ix/quire-rs#226`).
//! Two diagnostics were tried for that and both fired on healthy input: not
//! adopting an optional declaration is not a defect. It is a fact about what
//! was asked, so it belongs here, as [`Measurement::NotComputed`], rather than
//! in the warning stream.

use serde::{Deserialize, Serialize};

/// What a measurement found, or why it did not run.
///
/// An enum rather than a nullable value triple: "not computed" and "computed
/// zero" are different facts, and a shape that can express only one of them is
/// the defect this envelope exists to close.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Measurement {
    /// The measurement ran.
    Measured {
        /// The numerator, in `unit`.
        value: u64,
        /// The denominator, in `unit`. Zero is legitimate and is exactly the
        /// case a bare percentage cannot express (FR-050-AC-14).
        population: u64,
        /// How many **inputs** the measurement was offered, in whatever unit
        /// `method` names — source symbols walked, criteria reaching the
        /// classifier, rows reconciled.
        ///
        /// Paired with `matched` because their difference is the whole point:
        /// `matched` 0 of `examined` 0 means there was nothing to read and the
        /// zero is honest; `matched` 0 of `examined` 1,292 means the
        /// measurement was handed its input and could not read any of it.
        examined: u64,
        /// How many of those inputs it successfully read — the premise. This
        /// is the field that makes a hollow denominator visible.
        matched: u64,
    },
    /// The measurement did not run, and why. Never a zero dressed as an
    /// answer.
    NotComputed {
        /// The condition, in the engine's own vocabulary — "the module
        /// declares no `implements` marker forms", not "no data".
        because: String,
    },
}

/// Whether `value` is a share of `population` or a tally in its own right
/// (CR-102).
///
/// The distinction exists because **hollowness is a property of ratios only**.
/// A ratio whose numerator came from reading nothing is arithmetic over
/// nothing; a count that read everything and found none is simply zero, and
/// zero is the answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricShape {
    /// `value` out of `population` — the share is the point, and a denominator
    /// the measurement could not read makes the share meaningless.
    Ratio,
    /// `value` is a tally; `population` states what was searched to reach it.
    /// `matched` and `value` coincide, so `matched: 0` is the finding itself
    /// rather than a failure to look.
    Count,
}

/// One emitted number, with everything a reader needs to know what it counts.
///
/// Constructed through [`Metric::measured`] / [`Metric::counted`] /
/// [`Metric::not_computed`] so a metric cannot be built without its unit,
/// method and shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metric {
    /// Stable dotted token — `coverage.backed`. Machine-joinable, never
    /// rendered prose.
    pub name: String,
    /// What **one** of `value` is: `matrix row`, `acceptance criterion`,
    /// `source symbol`. Singular, so a reader can say "555 matrix rows".
    pub unit: String,
    /// How the number was arrived at, and what `matched` counts for it. One
    /// sentence, written for somebody deciding whether to trust the ratio.
    pub method: String,
    /// Whether `value` is a share or a tally. Decides whether
    /// [`Metric::is_hollow`] applies at all.
    pub shape: MetricShape,
    #[serde(flatten)]
    pub measurement: Measurement,
}

impl Metric {
    /// A **ratio** whose measurement ran: `value` out of `population`, from
    /// `matched` of the `examined` inputs.
    pub fn measured(
        name: &str,
        unit: &str,
        method: &str,
        value: usize,
        population: usize,
        examined: usize,
        matched: usize,
    ) -> Self {
        Self {
            name: name.to_string(),
            unit: unit.to_string(),
            method: method.to_string(),
            shape: MetricShape::Ratio,
            measurement: Measurement::Measured {
                value: value as u64,
                population: population as u64,
                examined: examined as u64,
                matched: matched as u64,
            },
        }
    }

    /// A **count** whose measurement ran: `value` found by searching
    /// `population`, having examined `examined` inputs.
    ///
    /// `matched` is `value` by construction — for a tally they are the same
    /// fact, and letting a caller pass them separately invites the pair to
    /// disagree.
    pub fn counted(
        name: &str,
        unit: &str,
        method: &str,
        value: usize,
        population: usize,
        examined: usize,
    ) -> Self {
        Self {
            name: name.to_string(),
            unit: unit.to_string(),
            method: method.to_string(),
            shape: MetricShape::Count,
            measurement: Measurement::Measured {
                value: value as u64,
                population: population as u64,
                examined: examined as u64,
                matched: value as u64,
            },
        }
    }

    /// A metric the engine did not compute, and the condition that decided it.
    pub fn not_computed(
        name: &str,
        unit: &str,
        method: &str,
        shape: MetricShape,
        because: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            unit: unit.to_string(),
            method: method.to_string(),
            shape,
            measurement: Measurement::NotComputed {
                because: because.to_string(),
            },
        }
    }

    /// A ratio computed over a population the measurement was handed input for
    /// and could not read: a non-empty denominator, input offered, and
    /// **nothing** matched.
    ///
    /// Four exclusions, each a shape that looks similar and is not:
    ///
    /// - **[`MetricShape::Count`]** — `matched` *is* `value` for a tally, so
    ///   `matched: 0` reports the finding rather than a failure to look
    ///   (CR-102). `coverage.implements` read 0 of 42 production symbols on
    ///   `agent-ix/spec-artifacts-process` and was called arithmetic over
    ///   nothing; the honest report is "0 of 42". This crate reads 15 of 1,412,
    ///   which is why a ratchet over one repository never saw it.
    /// - **`examined` 0** — there was nothing to read. A repository with no
    ///   tests reports 0% honestly, and calling that hollow would fire the
    ///   check on every greenfield corpus.
    /// - **`population` 0** — measuring nothing and saying so is honest;
    ///   FR-050-AC-14 already covers the model that mints no ids.
    /// - **`matched` low but non-zero** — a judgement, because the tail of a
    ///   migration looks the same from here. Reported with both counts where
    ///   the engine has the context to say so (FR-050-AC-27), never collapsed
    ///   to a boolean.
    ///
    /// What remains needs no threshold: input was offered, none of it was
    /// read, and a ratio was published anyway.
    pub fn is_hollow(&self) -> bool {
        self.shape == MetricShape::Ratio
            && matches!(
                self.measurement,
                Measurement::Measured {
                    population,
                    examined,
                    matched: 0,
                    ..
                } if population > 0 && examined > 0
            )
    }

    /// The numerator, when the measurement ran.
    pub fn value(&self) -> Option<u64> {
        match self.measurement {
            Measurement::Measured { value, .. } => Some(value),
            Measurement::NotComputed { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;

    #[trace("TC-985", "FR-063-AC-1", "FR-063-AC-6")]
    // a hollow denominator is a population the
    // measurement could not read, and it is distinguishable from every
    // neighbouring shape.
    #[test]
    fn tc985_hollow_is_a_nonempty_population_with_nothing_matched() {
        // 2,389 rows, 1,292 symbols walked, none of them read: the shape that
        // produced a confident 23% over a corpus the binder could not parse.
        let hollow = Metric::measured("coverage.backed", "matrix row", "…", 555, 2389, 1292, 0);
        assert!(hollow.is_hollow());

        // Something was read: not hollow, however low. The judgement call is
        // reported elsewhere with both counts, never collapsed to a boolean.
        assert!(
            !Metric::measured("coverage.backed", "matrix row", "…", 1, 2389, 1292, 1).is_hollow()
        );

        // Nothing was offered: a repository with no tests reports 0% honestly,
        // and calling that hollow would fire on every greenfield corpus.
        assert!(!Metric::measured("coverage.backed", "matrix row", "…", 0, 9, 0, 0).is_hollow());

        // Measuring nothing and saying so is honest, not hollow.
        assert!(!Metric::measured("coverage.backed", "matrix row", "…", 0, 0, 0, 0).is_hollow());

        // A measurement that never ran has no denominator to be hollow.
        let absent = Metric::not_computed(
            "coverage.implements",
            "requirement",
            "…",
            MetricShape::Count,
            "the module declares no `implements` marker forms",
        );
        assert!(!absent.is_hollow());
        assert_eq!(absent.value(), None);

        // A COUNT is never hollow (CR-102): `matched` IS the value, so zero is
        // the finding rather than a failure to read. This is the shape that
        // reported `coverage.implements` -- 0 of 42 production symbols on
        // agent-ix/spec-artifacts-process -- as arithmetic over nothing.
        let honest_zero =
            Metric::counted("coverage.implements", "production symbol", "…", 0, 42, 42);
        assert!(!honest_zero.is_hollow());
        // The same numbers as a RATIO still are, so this measures the shape
        // rather than the arithmetic.
        assert!(Metric::measured("coverage.backed", "matrix row", "…", 0, 42, 42, 0).is_hollow());
    }

    #[trace("TC-986", "FR-063-AC-2")]
    // `not computed` and `computed zero` are different
    // facts and serialize differently — the ambiguity #226 is about.
    #[test]
    fn tc986_not_computed_is_not_a_zero() {
        let zero = Metric::measured("m", "row", "counted every row", 0, 12, 12, 12);
        let absent = Metric::not_computed(
            "m",
            "row",
            "counted every row",
            MetricShape::Ratio,
            "nothing declared it",
        );
        assert_ne!(zero, absent);

        let zero_json = serde_json::to_value(&zero).expect("serialize");
        let absent_json = serde_json::to_value(&absent).expect("serialize");
        assert_eq!(zero_json["state"], "measured");
        assert_eq!(zero_json["value"], 0);
        assert_eq!(zero_json["population"], 12);
        assert_eq!(absent_json["state"], "not_computed");
        assert_eq!(absent_json["because"], "nothing declared it");
        // The uncomputed metric carries no numbers at all: there is no zero to
        // be mistaken for an answer.
        assert!(absent_json.get("value").is_none());
        assert!(absent_json.get("population").is_none());
        assert!(absent_json.get("examined").is_none());
        assert!(absent_json.get("matched").is_none());

        // Both round-trip, so a consumer reads back what the engine meant.
        for metric in [zero, absent] {
            let json = serde_json::to_string(&metric).expect("serialize");
            let back: Metric = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, metric);
        }
    }

    #[trace("TC-987", "FR-063-AC-1")]
    // every metric carries a unit and a method, because
    // there is no constructor that can omit them.
    #[test]
    fn tc987_a_metric_cannot_be_built_without_unit_and_method() {
        let metric = Metric::measured(
            "coverage.backed",
            "matrix row",
            "one row of a declared reference table; `matched` counts source \
             symbols that bound a trace id",
            555,
            2389,
            1292,
            0,
        );
        assert_eq!(metric.name, "coverage.backed");
        assert!(!metric.unit.is_empty());
        assert!(!metric.method.is_empty());
        assert_eq!(metric.value(), Some(555));
    }
}
