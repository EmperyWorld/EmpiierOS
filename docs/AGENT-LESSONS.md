# EmpiierOS agent lessons

**Status:** Living compressed lessons after the 2026-09-16 audit.
**Historical source:** `docs/capture/05-lessons-to-harvest.md`.

## Durable lessons

1. Evidence outranks route count, wave count, green badges, and LLM prose.
2. A mature capability is a Shell surface, an authenticated Runtime operation,
   a real substrate policy, and representative evidence.
3. File existence and mutable JSON state do not prove an OS operation.
4. Browser-supplied identity is not authentication; use kernel-provided peer
   identity and a pinned native boundary.
5. Fail-closed means command, persistence, and verification failures propagate
   as failures. Never log success before observation.
6. Production simulations must be compiled out or explicitly unavailable.
7. One service manager owns lifecycle. Nested restart loops hide the first
   failure and create boot storms.
8. The live installer, packaged archive, and installed system must derive from
   one verified artifact.
9. Accounts, SSH, sudo, and secret storage are image policy, not post-install
   cleanup.
10. Hooks are bounded workflow guardrails. They must not run acceptance, depend
    on network services, or brick the editor on advisory failures.
11. A semantic test validates behavior and negative cases; path, substring, and
    size checks are insufficient.
12. Fix the generator that admitted a class of failure, then measure it with a
    repeatable test.
13. Contract changes update living docs in the same phase.
14. Historical plans remain useful records but never supersede current observed
    maturity.
15. Trusted Substrate changes stay in the explicitly approved kinship scope;
    unrelated monorepo and Tauri paths remain untouched.
16. Commit and push remain operator decisions, with precise path staging.

## Current working loop

```text
scope → cause → trust boundary → implementation
      → positive and negative tests → image evidence → target evidence
      → honest maturity update
```

## Current authority

- `docs/NORTH-STAR.md`
- `docs/FOUNDATION-STATUS.md`
- `docs/plans/MATURITY-LEDGER.md`
- `docs/foundation/AGENT-KERNEL-SPEC.md`
- `.cursor/orchestration/reports/deep_secure_reset_audit.md`
