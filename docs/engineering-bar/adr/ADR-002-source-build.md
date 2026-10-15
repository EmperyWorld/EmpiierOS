# ADR-002: The OS image is built from source

- Status: accepted
- Date: 2026-09-23

## Context

This repo packages the shell and runtime. The bootable image is produced by Buildroot in the Trusted Substrate factory, from upstream source tarballs, then an overlay from this repo's packager.

## Decision

- Do not check in a rootfs, a vendor ISO, or a prebuilt toolchain as the product image.
- Do not document "download a binary OS and replace rootfs.tar.gz".
- The canonical image hash is updated only as its own step after a factory build, in the registry artifact pack.

## Consequences

Scripts in this repo may plant an already-built archive onto installer media. They are not a second way to compile the OS, and they are not permission to swap in an unknown blob.
