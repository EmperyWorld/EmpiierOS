# EmpiierOS — Seed policy (F6)

**Status:** Policy live under product unlock + PW0 A + Wave B chrome seed (2026-09-13). Further monorepo FE **copy** still requires a scoped seed wave + filled ORIGIN.  
**Method:** copy → paste → edit from monorepo FE kinship; conscious duplication; ORIGIN required when a seed task runs.

## Allowlist

What **may** be considered for a seed/scaffold task (requires operator scope lock + ORIGIN.md):

- Thin shell chrome / layout shells that do not pull connection-agent wholesale  
- Isolated UI primitives / design tokens adapted for OS Shell (React)  
- Small typed bridge stubs **rewritten** toward EmpiierOS Runtime contracts (not Tauri SoT)  
- Docs patterns / ORIGIN provenance discipline  

Always: paste → edit for OS axis; do not keep monorepo door names as product DNA.

## Denylist

Do **not** copy (day-1 or casually):

- Wholesale `connection-agent` / full SPA tree  
- Cloud deploy / mobile / Keycloak / Site Studio / UGS doors  
- Buildroot Dual SoT / Trusted Substrate factory into EmpiierOS  
- Monorepo `.cursor` zoo, Gemma gates, prod-line bake rituals  
- `node_modules/`, secrets, `.env`, audit dumps, `trash/`  
- Anything that recreates a second EliteEW cloud product under this root  

## Conscious duplication

Until Shell contracts stabilize: prefer **duplicated** adapted files over shared packages across EmpiierOS ↔ monorepo. Premature shared packages = Dual SoT risk.

## ORIGIN required

Any seed into `shell-ui/` (or related) must ship an `ORIGIN.md` filled from `ORIGIN.template.md` in the same phase. Preflight allows `shell-ui/package.json` **only** with ORIGIN substance headings; still **FAIL** on `node_modules` / kinship mirrors.

## Not this document

Actual file copy, `npm install`, or claiming Shell product Done.
