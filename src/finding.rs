//! Stable structured guidance shared by machine-generated findings.

use serde::{Deserialize, Serialize};

/// A finding's affected subject, repair target, and one safe next move.
///
/// The next move is an enum so a producer cannot serialize both a prescribed
/// remedy and an uncertainty-shaped diagnostic step. Human messages remain a
/// separate backward-compatible surface; consumers grade these typed fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindingGuidance {
    pub subject: String,
    pub change_target: String,
    #[serde(flatten)]
    pub next_move: FindingNextMove,
}

impl FindingGuidance {
    pub fn remedy(
        subject: impl Into<String>,
        change_target: impl Into<String>,
        remedy: impl Into<String>,
    ) -> Self {
        Self {
            subject: subject.into(),
            change_target: change_target.into(),
            next_move: FindingNextMove::Remedy {
                remedy: remedy.into(),
            },
        }
    }

    pub fn diagnostic(
        subject: impl Into<String>,
        change_target: impl Into<String>,
        next_diagnostic_step: impl Into<String>,
    ) -> Self {
        Self {
            subject: subject.into(),
            change_target: change_target.into(),
            next_move: FindingNextMove::NextDiagnosticStep {
                next_diagnostic_step: next_diagnostic_step.into(),
            },
        }
    }
}

/// Exactly one action shape, flattened beside the rest of the finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FindingNextMove {
    Remedy { remedy: String },
    NextDiagnosticStep { next_diagnostic_step: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn remedy_and_diagnostic_step_are_mutually_exclusive_on_the_wire() {
        let remedy = serde_json::to_value(FindingGuidance::remedy("row", "cell", "fix it"))
            .expect("guidance serializes");
        assert_eq!(
            remedy,
            json!({"subject": "row", "change_target": "cell", "remedy": "fix it"})
        );

        let diagnostic = serde_json::to_value(FindingGuidance::diagnostic(
            "metric",
            "trace forms",
            "inspect the census",
        ))
        .expect("guidance serializes");
        assert_eq!(
            diagnostic,
            json!({
                "subject": "metric",
                "change_target": "trace forms",
                "next_diagnostic_step": "inspect the census"
            })
        );
    }

    #[test]
    fn pre_guidance_coverage_diagnostics_still_deserialize() {
        let diagnostic: crate::coverage::CoverageDiagnostic = serde_json::from_value(json!({
            "declaration": "acceptance-criterion",
            "reason": "section-matches-nothing",
            "message": "the declared section matched no heading"
        }))
        .expect("old diagnostic payload remains readable");

        assert_eq!(diagnostic.guidance, None);
    }
}
