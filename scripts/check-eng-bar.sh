#!/usr/bin/env bash
# Usage: bash scripts/check-eng-bar.sh
# Dirty-tree gate. Does not build the image.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "${root}"

fail=0

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  printf 'BLOCK not a git tree\n'
  exit 1
fi

while IFS= read -r line; do
  [[ -z "${line}" ]] && continue
  path="${line:3}"
  case "${path}" in
    *"/dl/"*|*/rootfs.tar.gz|*/rootfs.tar|*/bzImage|*SECRETS-HANDOFF*|*admin.pass|*.env)
      printf 'BLOCK %s\n' "${path}"
      fail=1
      ;;
  esac
  case "${path}" in
    */package.json|*/Cargo.toml)
      if [[ -f "${path}" ]] && ! grep -Eq 'license(-file)?[[:space:]]*=' "${path}" && ! grep -Eq '"license"[[:space:]]*:' "${path}"; then
        printf 'BLOCK %s has no license field\n' "${path}"
        fail=1
      fi
      ;;
  esac
done < <(git status --porcelain)

if [[ "${fail}" -ne 0 ]]; then
  exit 1
fi
printf 'OK eng-bar\n'
