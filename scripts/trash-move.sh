#!/usr/bin/env bash
# Soft-delete: move paths into trash/<UTC-timestamp>/ preserving relative layout.
# Never rm. EmpiierOS trash-not-rm policy.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

usage() {
  cat <<'EOF'
Usage: scripts/trash-move.sh <path> [path...]

Moves each path into trash/<UTC-timestamp>/<relative-path> under the EmpiierOS root.
Does not rm. Prefer this over Delete / rm -rf.

Examples:
  scripts/trash-move.sh tmp-stub.txt
  scripts/trash-move.sh docs/plans/old-note.md
EOF
}

if [[ $# -lt 1 ]]; then
  usage >&2
  exit 2
fi

stamp="$(date -u +%Y%m%dT%H%M%SZ)"
dest_root="trash/${stamp}"
mkdir -p "$dest_root"

moved=0
for raw in "$@"; do
  # Normalize: strip leading ./ ; reject absolute escapes outside repo
  rel="${raw#./}"
  if [[ "$rel" = /* ]]; then
    echo "error: refuse absolute path: $raw" >&2
    exit 1
  fi
  if [[ "$rel" == trash/* || "$rel" == "trash" ]]; then
    echo "error: refuse moving trash into itself: $raw" >&2
    exit 1
  fi
  if [[ ! -e "$rel" && ! -L "$rel" ]]; then
    echo "error: path does not exist: $raw" >&2
    exit 1
  fi
  target="${dest_root}/${rel}"
  mkdir -p "$(dirname "$target")"
  mv -- "$rel" "$target"
  echo "moved: $rel -> $target"
  moved=$((moved + 1))
done

echo "trash-move: ${moved} path(s) -> ${dest_root}/"
