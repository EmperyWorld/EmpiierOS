# ADR-004: MIT and a public source snapshot

- Status: accepted
- Date: 2026-10-16
- Supersedes: ADR-003 for first-party licensing; ADR-001's no-GitHub restriction

## Decision

At the project owner's explicit request, first-party EmpiierOS code defaults
to MIT. Root LICENSE preserves the existing copyright holder. Package metadata,
UI notices, NOTICE and the license map agree on MIT. Existing third-party
licenses, kernel and Buildroot terms, and earlier distributed grants remain
unchanged. This decision does not relicense upstream components.

Publish the current source state as a single initial commit in
EmperyWorld/EmpiierOS. Exclude secrets, local agent configuration, build caches,
and OS images. The Forgejo origin remains unchanged. This source publication
is not a signed binary release, Ministry filing, or readiness claim.

## Consequences

Preserve MIT copyright and permission notices in copies or substantial parts.
Continue dependency compatibility checks and per-component legal-info/SBOMs.
The product remains NOT_READY until its existing acceptance gates pass.
