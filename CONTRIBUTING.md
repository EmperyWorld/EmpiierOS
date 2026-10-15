# Contributing

Registry status: **NOT_READY**. Do not claim inclusion.

## Remote

- `origin` is Forgejo: `ssh://git@130.193.43.111:2222/eliteew/empiieros.git`.
- Push only when the operator asked. The public GitHub snapshot is
  `EmperyWorld/EmpiierOS`, authorized by ADR-004; keep secrets and local
  agent configuration out of that export.
- Do not commit rootfs images, Buildroot `dl/`, or secrets.

## Change

- Message says why.
- User-facing strings go through `shell-ui/src/secure/locale/` (Russian first).
- Dependencies: [docs/engineering-bar/DEPENDENCY-POLICY.md](docs/engineering-bar/DEPENDENCY-POLICY.md).
- `bash scripts/check-eng-bar.sh` exits 0.
- New runtime behavior has a test or an explicit reason in the commit message.

## Do not

- Relicense the kernel or third-party components by changing the root MIT license.
- Paste UI or security sources into docs.
- Download a binary OS and call it the rootfs.
