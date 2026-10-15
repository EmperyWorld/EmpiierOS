# ADR-001: Forgejo is the only git remote

- Status: accepted
- Date: 2026-09-23

## Context

Registry item «и» needs the sources on infrastructure in the Russian Federation. This repo's `origin` is Forgejo on `130.193.43.111`, zone `ru-central1-b`. There is no GitHub remote. There is no HTTPS domain for git.

## Decision

- `origin` stays `ssh://git@130.193.43.111:2222/eliteew/empiieros.git`.
- Do not add a GitHub remote and do not push anywhere except `origin`, and only when the operator asked.
- Do not commit the factory `dl/` cache, rootfs, or secrets into this repo.

## Consequences

A second public remote is a new ADR, not a convenience push.
