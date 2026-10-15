#!/usr/bin/env python3
"""Run a needle file and report every failed assert.

Bare ``assert`` stops at the first miss and the shell label cannot name it.
This runner rewrites each assert into a collected needle, then prints every
miss. A later statement that raises still stops the file; needles already
seen are printed first.
"""

from __future__ import annotations

import ast
import sys
from pathlib import Path


class CollectAsserts(ast.NodeTransformer):
    def visit_Assert(self, node: ast.Assert) -> ast.AST:
        label = ast.unparse(node.test)
        if node.msg is not None:
            label = f"{label} ({ast.unparse(node.msg)})"
        call = ast.Expr(
            value=ast.Call(
                func=ast.Name(id="_needle", ctx=ast.Load()),
                args=[node.test, ast.Constant(value=label)],
                keywords=[],
            )
        )
        return ast.copy_location(call, node)


def main() -> int:
    path = Path(sys.argv[1])
    source = path.read_text(encoding="utf-8")
    tree = CollectAsserts().visit(ast.parse(source, filename=str(path)))
    ast.fix_missing_locations(tree)
    failures: list[str] = []

    def _needle(ok: object, label: str) -> None:
        if not ok:
            failures.append(label)

    globs: dict[str, object] = {"_needle": _needle, "__name__": "preflight_needles"}
    try:
        exec(compile(tree, str(path), "exec"), globs)
    except Exception as exc:
        failures.append(f"stopped: {type(exc).__name__}: {exc}")
    if failures:
        print(f"{len(failures)} needle(s) failed:", file=sys.stderr)
        for item in failures:
            print(f"  - {item}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
