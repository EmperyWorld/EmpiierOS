# Engineering bar

Registry filing status: **NOT_READY**. This repo is the OS shell and runtime, not the Buildroot factory. The factory lives in the Trusted Substrate tree of the monorepo. Do not copy that factory into this git history.

## Code

- Names say what the thing does. Side effects are visible at the call.
- Production Rust paths do not `unwrap()` unless a comment states the invariant that makes the panic a bug. Library and daemon code return `Result`.
- UI errors that a person can see go through `shell-ui/src/secure/locale/`. Russian is the primary locale. Do not hardcode a new user-facing sentence in a component when that locale module already exists.
- Do not log secrets, PINs, disk keys, or tokens.
- Least privilege on the runtime socket. Do not add a dev bypass on the production feature. `dev-peer-bypass` stays off the default `production` feature.
- New runtime behavior needs a test, or the change states why it cannot run here.
- Do not paste shell or security sources into docs.

## Build boundary

This repo builds the shell and the runtime. It does not emit the canonical rootfs. Image bytes and their hash are a factory step. Do not tell anyone to download a binary OS and drop it in as `rootfs.tar.gz`.

See [DEPENDENCY-POLICY.md](DEPENDENCY-POLICY.md) and [LICENSE-POLICY.md](LICENSE-POLICY.md).
