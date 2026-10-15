# EmpiierOS — Language bridge (disabled)

**Date:** 2026-09-17  
**Status:** **Disabled** by operator. Hook, skill, and Gemma CLI are off-disk.  
**Not:** senior-auditor / ОДОБРЕНО stop-loop (still DROP).

## Current law

Do not invoke a RU↔EN Gemma pipeline from hooks or skills.

- Operator language is used directly in chat replies.
- Touched docs under `docs/**` stay English.
- Historical Path A assets below are not living controls.

## Removed assets

| Former path | Role | State |
|-------------|------|-------|
| `.cursor/hooks/language_bridge_gate.py` | Overlay + outtake marks | removed |
| `.cursor/skills/language-bridge/` | Intake / outtake / ui-ru / health | removed |
| `.cursor/commands/language-bridge.md` | Slash pointer | trashed |

Advisory hooks `session-root-hygiene.py` and `foundation-completion-hint.py` no longer import the gate. Bindings stay fail-open and bounded.

## Historical note

Path A (2026-09-13) folded RU↔EN into those advisory hooks and a dedicated skill slot. Endpoint was LAN Gemma (`http://192.168.1.111:8080`). That path is not re-enabled unless the operator explicitly restores it.
