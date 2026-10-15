#!/usr/bin/env python3
# ---
# EmpiierOS · Empery World · https://empery.world
# Author: po&T && TZBrand
# ---
# Purpose:
# Command-post ingest + pack/publish for EmpiierOS OTA edge (E0c/E1b):
# verify F9 artifact Ed25519 capsule, enforce size/path/href locks, push into
# stack-empiieros-ota tmpfs via podman. Pack builds blob-ref capsules (no host bind).
# Invariants:
# - Unsigned / wrong-key / oversize / path_escape / href escape → refuse (no write).
# - No host /srv capsule bind; writes only into running container tmpfs.
# - Artifact SK never accepted on ingest — verify-only PK (env or --pk-file).
# - Pack/sign uses CP-local SK only; never commit SK; prod disk is not capsule SoT.
# - Verify: cryptography if present, else openssl pkeyutl (prod without pip).
# Related:
# - docs/plans/TZ-EMPIERO-OS-OTA-EDGE.md §E0c §E1
# - runtime/src/ota_capsule_store.rs (canonical payload bytes + blob-ref)
# ---
from __future__ import annotations

import argparse
import base64
import hashlib
import json
import os
import secrets
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

KIND = "empiieros_ota_capsule_v1"
# Ed25519 SubjectPublicKeyInfo prefix (RFC 8410) for openssl pkeyutl -verify.
_ED25519_SPKI_PREFIX = bytes.fromhex("302a300506032b6570032100")
CHANNELS = frozenset({"os_component", "fofline_update"})
MAX_FILE_BYTES = int(os.environ.get("EMPIIERO_OTA_MAX_FILE_BYTES", str(256 * 1024 * 1024)))
MAX_TOTAL_BYTES = int(os.environ.get("EMPIIERO_OTA_MAX_TOTAL_BYTES", str(512 * 1024 * 1024)))
MAX_FILES = int(os.environ.get("EMPIIERO_OTA_MAX_FILES", "64"))
CONTAINER = os.environ.get("EMPIERO_OS_OTA_CONTAINER", "stack-empiieros-ota")
DEST_ROOT = "/srv/empiieros-ota"


def die(msg: str, code: int = 2) -> None:
    print(f"ERROR: {msg}", file=sys.stderr)
    raise SystemExit(code)


def sort_value(value: Any) -> Any:
    if isinstance(value, dict):
        return {k: sort_value(value[k]) for k in sorted(value.keys())}
    if isinstance(value, list):
        return [sort_value(v) for v in value]
    return value


def canonical_bytes(payload: dict[str, Any]) -> bytes:
    return json.dumps(sort_value(payload), separators=(",", ":"), ensure_ascii=False).encode(
        "utf-8"
    )


def parse_pk_hex(raw: str) -> bytes:
    h = raw.strip().lower()
    if len(h) != 64:
        die("artifact_role_mismatch: pubkey must be 32-byte hex")
    try:
        return bytes.fromhex(h)
    except ValueError as e:
        die(f"artifact_role_mismatch: {e}")


def load_pk(args: argparse.Namespace) -> bytes:
    if args.pk_file:
        return parse_pk_hex(Path(args.pk_file).read_text(encoding="utf-8"))
    env = os.environ.get("EMPIIEROS_OTA_ARTIFACT_PK", "").strip()
    if env:
        return parse_pk_hex(env)
    die("capsule_signature_required: set EMPIIEROS_OTA_ARTIFACT_PK or --pk-file")


def _verify_ed25519(pk: bytes, sig: bytes, message: bytes) -> bool:
    """Verify Ed25519; prefer cryptography, else openssl (prod hosts without pip)."""
    force_openssl = os.environ.get("EMPIIEROS_OTA_VERIFY_OPENSSL_ONLY", "").strip() in (
        "1",
        "true",
        "yes",
    )
    if not force_openssl:
        try:
            from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey
            from cryptography.exceptions import InvalidSignature

            try:
                Ed25519PublicKey.from_public_bytes(pk).verify(sig, message)
                return True
            except InvalidSignature:
                return False
        except ImportError:
            pass

    with tempfile.TemporaryDirectory(prefix="empiieros-ota-verify-") as tmp:
        root = Path(tmp)
        (root / "msg.bin").write_bytes(message)
        (root / "sig.bin").write_bytes(sig)
        spki = _ED25519_SPKI_PREFIX + pk
        pem = (
            "-----BEGIN PUBLIC KEY-----\n"
            + base64.encodebytes(spki).decode("ascii")
            + "-----END PUBLIC KEY-----\n"
        )
        (root / "pk.pem").write_text(pem, encoding="utf-8")
        r = subprocess.run(
            [
                "openssl",
                "pkeyutl",
                "-verify",
                "-pubin",
                "-inkey",
                str(root / "pk.pem"),
                "-rawin",
                "-in",
                str(root / "msg.bin"),
                "-sigfile",
                str(root / "sig.bin"),
            ],
            check=False,
            capture_output=True,
            text=True,
        )
        if r.returncode == 0:
            return True
        if "Verification Failure" in (r.stdout + r.stderr) or r.returncode == 1:
            return False
        die(f"artifact_role_mismatch: openssl verify: {r.stderr.strip() or r.stdout.strip()}")


def verify_doc(doc: dict[str, Any], pk: bytes) -> dict[str, Any]:
    if doc.get("v") != 1:
        die("capsule_signature_required: v must be 1")
    payload = doc.get("payload")
    sig_hex = doc.get("sig")
    if not isinstance(payload, dict) or not isinstance(sig_hex, str) or not sig_hex.strip():
        die("capsule_signature_required")
    kind = payload.get("kind")
    channel = payload.get("channel")
    if kind != KIND:
        die("artifact_role_mismatch: kind")
    if channel not in CHANNELS:
        die("artifact_role_mismatch: channel")
    try:
        sig = bytes.fromhex(sig_hex.strip())
    except ValueError:
        die("capsule_signature_required: sig hex")
    if len(sig) != 64:
        die("capsule_signature_required: sig length")
    if not _verify_ed25519(pk, sig, canonical_bytes(payload)):
        die("capsule_signature_required: bad signature")
    return payload


def path_safe(rel: str) -> bool:
    t = rel.strip()
    if not t or t.startswith("/") or ".." in t or "\0" in t or ":" in t:
        return False
    return t.startswith("runtime/") or t.startswith("shell/") or t.startswith("helpers/")


def is_sha256_hex(s: str) -> bool:
    return len(s) == 64 and all(c in "0123456789abcdef" for c in s)


BLOB_HREF_PREFIX = "/empiieros-ota/v1/blobs/"


def parse_blob_href(href: str, want_sha256: str) -> str:
    t = href.strip()
    if not t.startswith(BLOB_HREF_PREFIX):
        die(f"path_escape: href prefix {href}")
    name = t[len(BLOB_HREF_PREFIX) :]
    if (
        not name
        or "/" in name
        or ".." in name
        or "\0" in name
        or "?" in name
        or "#" in name
        or ":" in name
    ):
        die(f"path_escape: href {href}")
    name = name.lower()
    if not is_sha256_hex(name):
        die(f"path_escape: href digest {href}")
    if name != want_sha256.strip().lower():
        die("path_escape: href sha256 mismatch")
    return name


def collect_files(payload: dict[str, Any], blob_dir: Path | None) -> list[tuple[str, bytes]]:
    files = payload.get("files")
    if not isinstance(files, list) or not files:
        die("payload_incomplete: files")
    if len(files) > MAX_FILES:
        die("path_escape: too many files")
    out: list[tuple[str, bytes]] = []
    total = 0
    for entry in files:
        if not isinstance(entry, dict):
            die("payload_incomplete: file entry")
        rel = entry.get("path")
        want = entry.get("sha256")
        if not isinstance(rel, str) or not isinstance(want, str):
            die("payload_incomplete: path/sha256")
        if not path_safe(rel):
            die(f"path_escape: {rel}")
        want_norm = want.strip().lower()
        if not is_sha256_hex(want_norm):
            die("payload_incomplete: sha256")
        raw: bytes | None = None
        b64 = entry.get("content_base64")
        href = entry.get("href")
        has_b64 = isinstance(b64, str) and b64.strip()
        has_href = isinstance(href, str) and href.strip()
        if has_b64 and has_href:
            die("payload_incomplete: both content_base64 and href")
        if has_b64:
            # Bound before decode so refuse does not allocate a multi-GiB buffer.
            est = (len(b64.strip()) * 3) // 4
            if est > MAX_FILE_BYTES:
                die("path_escape: file too large")
            try:
                raw = base64.b64decode(b64.strip(), validate=True)
            except Exception:
                die("payload_incomplete: content_base64")
        elif has_href:
            name = parse_blob_href(href, want_norm)
            if not blob_dir:
                die("payload_incomplete: --blob-dir required for href blobs")
            path = blob_dir / name
            if not path.is_file():
                die(f"payload_incomplete: missing blob {name}")
            raw = path.read_bytes()
        else:
            die("payload_incomplete: need content_base64 or href+--blob-dir")
        assert raw is not None
        if len(raw) > MAX_FILE_BYTES:
            die("path_escape: file too large")
        total += len(raw)
        if total > MAX_TOTAL_BYTES:
            die("path_escape: total too large")
        got = hashlib.sha256(raw).hexdigest()
        if got != want_norm:
            die("capsule_signature_required: sha256 mismatch")
        out.append((rel, raw))
    return out


def podman(*args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
    cmd = ["podman", *args]
    return subprocess.run(cmd, check=check, text=True, capture_output=True)


def ensure_container() -> None:
    r = podman("inspect", "-f", "{{.State.Status}}", CONTAINER, check=False)
    if r.returncode != 0 or r.stdout.strip() != "running":
        die(f"container {CONTAINER} not running")


def push_tmpfs(doc: dict[str, Any], files: list[tuple[str, bytes]]) -> None:
    ensure_container()
    with tempfile.TemporaryDirectory(prefix="empiieros-ota-ingest-") as tmp:
        root = Path(tmp)
        (root / "blobs").mkdir()
        (root / "current.json").write_text(
            json.dumps(doc, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
        )
        for rel, raw in files:
            # Serve blobs by sha256 name for href layout; also keep path tree for debugging.
            digest = hashlib.sha256(raw).hexdigest()
            (root / "blobs" / digest).write_bytes(raw)
            dest = root / "tree" / rel
            dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_bytes(raw)
        # Clear and copy into container tmpfs (no host bind).
        clr = podman(
            "exec",
            CONTAINER,
            "sh",
            "-c",
            f"rm -rf {DEST_ROOT}/blobs {DEST_ROOT}/tree {DEST_ROOT}/current.json "
            f"&& mkdir -p {DEST_ROOT}/blobs {DEST_ROOT}/tree",
            check=False,
        )
        if clr.returncode != 0:
            die(f"podman clear failed: {clr.stderr.strip()}")
        for name in ("current.json", "blobs", "tree"):
            src = root / name
            cp = podman("cp", str(src), f"{CONTAINER}:{DEST_ROOT}/", check=False)
            if cp.returncode != 0:
                die(f"podman cp {name} failed: {cp.stderr.strip()}")
    # Smoke: current must exist inside container
    smoke = podman(
        "exec",
        CONTAINER,
        "sh",
        "-c",
        f"test -s {DEST_ROOT}/current.json && wget -qO- http://127.0.0.1/v1/current.json | head -c 20",
        check=False,
    )
    if smoke.returncode != 0:
        die(f"post-ingest smoke failed: {smoke.stderr.strip()}")
    print("OK: ingested signed capsule into tmpfs")


def cmd_ingest(args: argparse.Namespace) -> None:
    pk = load_pk(args)
    doc = json.loads(Path(args.capsule).read_text(encoding="utf-8"))
    if not isinstance(doc, dict):
        die("capsule_signature_required: root object")
    payload = verify_doc(doc, pk)
    blob_dir = Path(args.blob_dir) if args.blob_dir else None
    files = collect_files(payload, blob_dir)
    if args.verify_only:
        print(f"OK: signature valid files={len(files)}")
        return
    push_tmpfs(doc, files)


def _require_cryptography_sign():
    try:
        from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

        return Ed25519PrivateKey
    except ImportError as e:
        die(f"sign/selftest needs python3-cryptography on CP host: {e}")


def cmd_selftest(_: argparse.Namespace) -> None:
    Ed25519PrivateKey = _require_cryptography_sign()
    sk = Ed25519PrivateKey.generate()
    pk = sk.public_key().public_bytes_raw()
    body = b"empiieros-e0c-fixture\n"
    digest = hashlib.sha256(body).hexdigest()
    payload = {
        "kind": KIND,
        "channel": "os_component",
        "files": [
            {
                "path": "helpers/empiieros-ota-fetch",
                "sha256": digest,
                "content_base64": base64.b64encode(body).decode("ascii"),
            }
        ],
    }
    sig = sk.sign(canonical_bytes(payload)).hex()
    doc = {"v": 1, "payload": payload, "sig": sig}

    # Force openssl path once so prod-shaped hosts stay covered.
    os.environ["EMPIIEROS_OTA_VERIFY_OPENSSL_ONLY"] = "1"
    try:
        verify_doc(doc, pk)
    finally:
        os.environ.pop("EMPIIEROS_OTA_VERIFY_OPENSSL_ONLY", None)
    collect_files(payload, None)

    bad = {"v": 1, "payload": payload}
    try:
        verify_doc(bad, pk)
        die("selftest: unsigned accepted")
    except SystemExit:
        pass

    other = Ed25519PrivateKey.generate().public_key().public_bytes_raw()
    try:
        verify_doc(doc, other)
        die("selftest: wrong key accepted")
    except SystemExit:
        pass

    # Cap override keeps selftest allocation tiny.
    os.environ["EMPIIERO_OTA_MAX_FILE_BYTES"] = "64"
    global MAX_FILE_BYTES
    MAX_FILE_BYTES = 64
    huge = {
        "kind": KIND,
        "channel": "os_component",
        "files": [
            {
                "path": "helpers/empiieros-ota-fetch",
                "sha256": hashlib.sha256(b"x").hexdigest(),
                "content_base64": base64.b64encode(b"x" * 80).decode("ascii"),
            }
        ],
    }
    try:
        collect_files(huge, None)
        die("selftest: oversize accepted")
    except SystemExit:
        pass
    os.environ.pop("EMPIIERO_OTA_MAX_FILE_BYTES", None)
    MAX_FILE_BYTES = int(os.environ.get("EMPIIERO_OTA_MAX_FILE_BYTES", str(256 * 1024 * 1024)))

    esc = {
        "kind": KIND,
        "channel": "os_component",
        "files": [
            {
                "path": "../etc/passwd",
                "sha256": digest,
                "content_base64": base64.b64encode(body).decode("ascii"),
            }
        ],
    }
    try:
        collect_files(esc, None)
        die("selftest: path_escape accepted")
    except SystemExit:
        pass

    # Blob-ref href locks (E1a).
    with tempfile.TemporaryDirectory(prefix="empiieros-ota-href-") as tmp:
        bdir = Path(tmp)
        (bdir / digest).write_bytes(body)
        href_ok = {
            "kind": KIND,
            "channel": "os_component",
            "files": [
                {
                    "path": "helpers/empiieros-ota-fetch",
                    "sha256": digest,
                    "href": f"{BLOB_HREF_PREFIX}{digest}",
                }
            ],
        }
        collect_files(href_ok, bdir)
        bad_href = {
            "kind": KIND,
            "channel": "os_component",
            "files": [
                {
                    "path": "helpers/empiieros-ota-fetch",
                    "sha256": digest,
                    "href": f"{BLOB_HREF_PREFIX}../{digest}",
                }
            ],
        }
        try:
            collect_files(bad_href, bdir)
            die("selftest: href escape accepted")
        except SystemExit:
            pass
        mismatch = {
            "kind": KIND,
            "channel": "os_component",
            "files": [
                {
                    "path": "helpers/empiieros-ota-fetch",
                    "sha256": digest,
                    "href": f"{BLOB_HREF_PREFIX}{'00' * 32}",
                }
            ],
        }
        try:
            collect_files(mismatch, bdir)
            die("selftest: href sha mismatch accepted")
        except SystemExit:
            pass

    print("OK: empiieros-ota-cp-ingest selftest")
    print(f"TIP: EMPIIEROS_OTA_ARTIFACT_PK={pk.hex()}")


def operator_sk_path() -> Path:
    xdg = Path(os.environ.get("XDG_DATA_HOME", str(Path.home() / ".local/share")))
    override = os.environ.get("EMPIIEROS_OTA_OPERATOR_DIR", "").strip()
    if override:
        return Path(override) / "artifact.sk.hex"
    return xdg / "empiieros" / "ota" / "artifact.sk.hex"


def _load_sk(args: argparse.Namespace) -> Any:
    Ed25519PrivateKey = _require_cryptography_sign()
    raw = ""
    if args.sk_file:
        raw = Path(args.sk_file).read_text(encoding="utf-8").strip()
    else:
        raw = os.environ.get("EMPIIEROS_OTA_ARTIFACT_SK", "").strip()
        if not raw:
            living = operator_sk_path()
            if living.is_file():
                raw = living.read_text(encoding="utf-8").strip()
    if raw.strip().lower() == "redacted":
        die("capsule_signature_required: living SK is REDACTED")
    if not raw or len(raw) != 64:
        die(
            "capsule_signature_required: set EMPIIEROS_OTA_ARTIFACT_SK, --sk-file, "
            "or scripts/empiieros-ota-operator-key.sh init"
        )
    try:
        sk_bytes = bytes.fromhex(raw.lower())
    except ValueError as e:
        die(f"capsule_signature_required: {e}")
    if len(sk_bytes) != 32:
        die("capsule_signature_required: sk length")
    return Ed25519PrivateKey.from_private_bytes(sk_bytes)


def _collect_pack_files(file_args: list[str], src: Path | None) -> list[tuple[str, bytes]]:
    out: list[tuple[str, bytes]] = []
    seen: set[str] = set()
    for item in file_args:
        if "=" not in item:
            die(f"payload_incomplete: --file expects rel=path, got {item}")
        rel, abs_s = item.split("=", 1)
        rel = rel.strip()
        if not path_safe(rel):
            die(f"path_escape: {rel}")
        if rel in seen:
            die(f"path_escape: duplicate {rel}")
        seen.add(rel)
        path = Path(abs_s)
        if not path.is_file():
            die(f"payload_incomplete: missing {path}")
        raw = path.read_bytes()
        if len(raw) > MAX_FILE_BYTES:
            die(f"path_escape: file too large {rel}")
        out.append((rel, raw))
    if src is not None:
        if not src.is_dir():
            die(f"payload_incomplete: --src not a dir: {src}")
        for prefix in ("runtime", "shell", "helpers"):
            base = src / prefix
            if not base.is_dir():
                continue
            for path in sorted(base.rglob("*")):
                if not path.is_file():
                    continue
                rel = str(path.relative_to(src)).replace("\\", "/")
                if not path_safe(rel):
                    die(f"path_escape: {rel}")
                if rel in seen:
                    die(f"path_escape: duplicate {rel}")
                seen.add(rel)
                raw = path.read_bytes()
                if len(raw) > MAX_FILE_BYTES:
                    die(f"path_escape: file too large {rel}")
                out.append((rel, raw))
    if not out:
        die("payload_incomplete: no files (--file and/or --src)")
    if len(out) > MAX_FILES:
        die("path_escape: too many files")
    total = sum(len(b) for _, b in out)
    if total > MAX_TOTAL_BYTES:
        die("path_escape: total too large")
    return out


def cmd_pack(args: argparse.Namespace) -> Path:
    """Build blob-ref capsule + blobs/ from allowlisted files; sign with artifact SK."""
    sk = _load_sk(args)
    pk = sk.public_key().public_bytes_raw()
    files = _collect_pack_files(list(args.file or []), Path(args.src) if args.src else None)
    out = Path(args.out_dir)
    blobs = out / "blobs"
    out.mkdir(parents=True, exist_ok=True)
    if blobs.exists():
        for child in blobs.iterdir():
            if child.is_file():
                child.write_bytes(b"")
                child.unlink(missing_ok=True)
    blobs.mkdir(parents=True, exist_ok=True)
    entries: list[dict[str, str]] = []
    for rel, raw in files:
        digest = hashlib.sha256(raw).hexdigest()
        (blobs / digest).write_bytes(raw)
        entries.append(
            {
                "path": rel,
                "sha256": digest,
                "href": f"{BLOB_HREF_PREFIX}{digest}",
            }
        )
    payload: dict[str, Any] = {
        "kind": KIND,
        "channel": args.channel,
        "files": entries,
    }
    if args.version:
        payload["version"] = args.version
    if args.app_build_id:
        payload["app_build_id"] = args.app_build_id
    doc = {"v": 1, "payload": payload, "sig": sk.sign(canonical_bytes(payload)).hex()}
    (out / "capsule.json").write_text(json.dumps(doc, indent=2) + "\n", encoding="utf-8")
    (out / "artifact.pk.hex").write_text(pk.hex() + "\n", encoding="utf-8")
    # Re-verify with blob-dir before claiming OK.
    verify_doc(doc, pk)
    collect_files(payload, blobs)
    print(f"OK: packed blob-ref capsule in {out} files={len(entries)}")
    return out


def cmd_publish(args: argparse.Namespace) -> None:
    """Pack (if needed) then ingest into stack-empiieros-ota tmpfs (local or --ssh host)."""
    if args.capsule and args.out_dir:
        die("publish: use either --capsule+--blob-dir or pack flags (--out-dir), not both")
    if args.capsule:
        capsule = Path(args.capsule)
        blob_dir = Path(args.blob_dir) if args.blob_dir else None
        if not blob_dir:
            die("publish: --blob-dir required with --capsule")
        pk_file = Path(args.pk_file) if args.pk_file else None
    else:
        if not args.out_dir:
            die("publish: need --out-dir (pack) or --capsule")
        out = cmd_pack(args)
        capsule = out / "capsule.json"
        blob_dir = out / "blobs"
        pk_file = out / "artifact.pk.hex"

    if not args.ssh:
        ns = argparse.Namespace(
            capsule=str(capsule),
            blob_dir=str(blob_dir),
            pk_file=str(pk_file) if pk_file else None,
            verify_only=False,
        )
        cmd_ingest(ns)
        return

    # Remote: rsync verified tree + run ingest on host that owns the container.
    host = args.ssh
    remote_root = f"/tmp/empiieros-ota-publish-{secrets.token_hex(4)}"
    script = Path(__file__).resolve()
    subprocess.run(
        ["ssh", host, f"mkdir -p {remote_root}/blobs"],
        check=True,
    )
    subprocess.run(
        ["rsync", "-a", str(script), f"{host}:{remote_root}/empiieros-ota-cp-ingest.py"],
        check=True,
    )
    subprocess.run(
        ["rsync", "-a", str(capsule), f"{host}:{remote_root}/capsule.json"],
        check=True,
    )
    subprocess.run(
        ["rsync", "-a", f"{blob_dir}/", f"{host}:{remote_root}/blobs/"],
        check=True,
    )
    if pk_file and pk_file.is_file():
        subprocess.run(
            ["rsync", "-a", str(pk_file), f"{host}:{remote_root}/artifact.pk.hex"],
            check=True,
        )
    remote_cmd = (
        f"python3 {remote_root}/empiieros-ota-cp-ingest.py ingest "
        f"--capsule {remote_root}/capsule.json "
        f"--blob-dir {remote_root}/blobs "
        f"--pk-file {remote_root}/artifact.pk.hex"
    )
    subprocess.run(["ssh", host, remote_cmd], check=True)
    print(f"OK: published via ssh {host}")


def cmd_sign_fixture(args: argparse.Namespace) -> None:
    """Lab-only: mint SK/PK, write signed fixture + pk file (never commit SK)."""
    Ed25519PrivateKey = _require_cryptography_sign()
    out = Path(args.out_dir)
    out.mkdir(parents=True, exist_ok=True)
    sk = Ed25519PrivateKey.generate()
    pk = sk.public_key().public_bytes_raw()
    body = args.note.encode("utf-8") if args.note else f"e0c-{secrets.token_hex(4)}\n".encode()
    digest = hashlib.sha256(body).hexdigest()
    payload = {
        "kind": KIND,
        "channel": args.channel,
        "files": [
            {
                "path": "helpers/empiieros-ota-fetch",
                "sha256": digest,
                "content_base64": base64.b64encode(body).decode("ascii"),
            }
        ],
    }
    doc = {"v": 1, "payload": payload, "sig": sk.sign(canonical_bytes(payload)).hex()}
    (out / "artifact.pk.hex").write_text(pk.hex() + "\n", encoding="utf-8")
    (out / "artifact.sk.hex").write_text(sk.private_bytes_raw().hex() + "\n", encoding="utf-8")
    (out / "capsule.json").write_text(json.dumps(doc, indent=2) + "\n", encoding="utf-8")
    print(f"OK: fixture in {out} (delete artifact.sk.hex after ingest; never commit)")


def main() -> None:
    ap = argparse.ArgumentParser(description="EmpiierOS OTA CP ingest + pack/publish (E0c/E1b)")
    sub = ap.add_subparsers(dest="cmd", required=True)

    p_ing = sub.add_parser("ingest", help="verify + push signed capsule into container tmpfs")
    p_ing.add_argument("--capsule", required=True, help="path to signed capsule JSON")
    p_ing.add_argument("--blob-dir", default="", help="directory of blob files for href entries")
    p_ing.add_argument("--pk-file", default="", help="artifact verify pubkey hex file")
    p_ing.add_argument(
        "--verify-only",
        action="store_true",
        help="verify locks only; do not podman push",
    )
    p_ing.set_defaults(func=cmd_ingest)

    p_st = sub.add_parser("selftest", help="unsigned/wrong-key/oversize/path/href refuse")
    p_st.set_defaults(func=cmd_selftest)

    p_fx = sub.add_parser("sign-fixture", help="lab mint signed inline capsule (SK local only)")
    p_fx.add_argument("--out-dir", required=True)
    p_fx.add_argument("--channel", default="os_component", choices=sorted(CHANNELS))
    p_fx.add_argument("--note", default="")
    p_fx.set_defaults(func=cmd_sign_fixture)

    def add_pack_flags(p: argparse.ArgumentParser, *, out_required: bool) -> None:
        p.add_argument(
            "--out-dir",
            required=out_required,
            default="",
            help="write capsule.json + blobs/ + artifact.pk.hex",
        )
        p.add_argument("--channel", default="os_component", choices=sorted(CHANNELS))
        p.add_argument("--sk-file", default="", help="artifact SK hex (or EMPIIEROS_OTA_ARTIFACT_SK)")
        p.add_argument("--src", default="", help="tree with runtime/ shell/ helpers/")
        p.add_argument(
            "--file",
            action="append",
            default=[],
            help="rel=abs path (repeatable), e.g. helpers/x=/tmp/x",
        )
        p.add_argument("--version", default="")
        p.add_argument("--app-build-id", default="")

    p_pack = sub.add_parser("pack", help="E1b: pack allowlisted files → signed blob-ref capsule")
    add_pack_flags(p_pack, out_required=True)
    p_pack.set_defaults(func=cmd_pack)

    p_pub = sub.add_parser("publish", help="E1b: pack (or existing capsule) + ingest into tmpfs")
    add_pack_flags(p_pub, out_required=False)
    p_pub.add_argument("--capsule", default="", help="existing signed capsule (skip pack)")
    p_pub.add_argument("--blob-dir", default="", help="blobs for --capsule")
    p_pub.add_argument("--pk-file", default="", help="verify PK for --capsule")
    p_pub.add_argument(
        "--ssh",
        default="",
        help="remote host that owns stack-empiieros-ota (rsync + ingest)",
    )
    p_pub.set_defaults(func=cmd_publish)

    args = ap.parse_args()
    if getattr(args, "blob_dir", "") == "":
        args.blob_dir = None
    if getattr(args, "pk_file", "") == "":
        args.pk_file = None
    if getattr(args, "sk_file", "") == "":
        args.sk_file = None
    if getattr(args, "src", "") == "":
        args.src = None
    if getattr(args, "ssh", "") == "":
        args.ssh = None
    if getattr(args, "capsule", "") == "":
        args.capsule = None
    if getattr(args, "out_dir", "") == "":
        args.out_dir = None
    args.func(args)


if __name__ == "__main__":
    main()
