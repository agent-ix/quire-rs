---
id: FR-097
title: "Private vulnerability reporting policy"
type: FR
---
# FR-097: Private vulnerability reporting policy

Tracked by Linear PLAT-1843.

## Description

The repository-root `SECURITY.md` SHALL direct security vulnerability reports for
Quire-rs to GitHub's private vulnerability reporting flow for
`agent-ix/quire-rs`. It SHALL tell reporters not to disclose vulnerabilities
through public issues, discussions, or pull requests. It SHALL NOT promise a
response or remediation time, supported version range, safe harbor, severity
classification, or disclosure schedule.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-097-AC-1 | `SECURITY.md` links to `https://github.com/agent-ix/quire-rs/security/advisories/new`, directs reports through that private route, and warns against public issues, discussions, and pull requests without making unsupported response, remediation, version, safe-harbor, severity, or disclosure-schedule promises. | Inspection |

## Dependencies

- None.
