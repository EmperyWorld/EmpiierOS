#!/usr/bin/env python3
# ---
# EmpiierOS · Empery World · https://empery.world
# Author: po&T && TZBrand
# ---
# Purpose:
# Lab/fixture plant of verify-only F9 artifact PK at
# {state}/ota/artifact.pk.hex. Production plant is Air license
# (entitlement payload.ota_artifact_pk on launcher.air_activate).
# Never accepts or writes an artifact SK. Not the installer.
# Related:
# - docs/plans/TZ-EMPIERO-OS-OTA-EDGE.md §E3
# - runtime/src/ota_capsule_store.rs plant_artifact_pk
# ---
from __future__ import annotations

import argparse
import os
import sys
from pathlib import Path


def die(msg: str, code: int = 2) -> None:
    print(f"ERROR: {msg}", file=sys.stderr)
    raise SystemExit(code)


def main() -> None:
    ap = argparse.ArgumentParser(description="Plant EmpiierOS OTA artifact verify PK (E3)")
    ap.add_argument(
        "--state-dir",
        default=os.environ.get("EMPIIEROS_RUNTIME_STATE_DIR", "/var/lib/empiieros-runtime"),
        help="Runtime state root (default EMPIIEROS_RUNTIME_STATE_DIR or /var/lib/empiieros-runtime)",
    )
    ap.add_argument("--pk-file", required=True, help="32-byte hex pubkey file (verify-only)")
    args = ap.parse_args()
    raw = Path(args.pk_file).read_text(encoding="utf-8").strip().lower()
    if len(raw) != 64 or any(c not in "0123456789abcdef" for c in raw):
        die("artifact_role_mismatch: pubkey must be 32-byte hex")
    state = Path(args.state_dir)
    ota = state / "ota"
    ota.mkdir(parents=True, exist_ok=True)
    os.chmod(ota, 0o700)
    dest = ota / "artifact.pk.hex"
    dest.write_text(raw + "\n", encoding="utf-8")
    os.chmod(dest, 0o600)
    print(f"OK: planted {dest}")


if __name__ == "__main__":
    main()
