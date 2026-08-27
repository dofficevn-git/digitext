#!/usr/bin/env python3
"""Strip comments/docstrings across the tracked source tree.

Every rewrite is checked against a deletion-only invariant before it is allowed
to land: the output, with whitespace removed, must be a subsequence of the input
with whitespace removed. A stripper that corrupted a string literal or reordered
code would violate it. Python files get a stronger check -- the AST must match
the original's once docstrings are dropped from both.

    python3 dev/obfuscate/run_strip.py --check    # dry run + guards
    python3 dev/obfuscate/run_strip.py --apply    # rewrite in place
"""

from __future__ import annotations

import argparse
import ast
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from strip_comments import (  # noqa: E402
    collapse_blank_lines,
    strip_python,
    strip_rust,
    strip_ts,
)

REPO = Path(__file__).resolve().parents[2]

# Tauri-generated sources are rebuilt from config; rewriting them is pointless.
#
# `dev/obfuscate/` is the machinery, not the product. Obfuscating it hides
# nothing about DigiText while deleting the reasoning behind each exclusion rule
# -- why `None` must not be renamed, why a Tauri parameter is a wire contract --
# which is what makes a re-run safe. Drop the entry here to include it.
EXCLUDE_PREFIXES = ("desktop/src-tauri/gen/", "dev/obfuscate/")


def tracked(patterns: list[str]) -> list[Path]:
    out = subprocess.run(
        ["git", "ls-files", *patterns],
        cwd=REPO,
        capture_output=True,
        text=True,
        check=True,
    ).stdout.split()
    return [REPO / p for p in out if not p.startswith(EXCLUDE_PREFIXES)]


def is_subsequence(needle: str, haystack: str) -> bool:
    """True when `needle` can be obtained from `haystack` by deletion alone."""
    it = iter(haystack)
    return all(ch in it for ch in needle)


def _strip_docstrings_from_ast(tree: ast.AST) -> ast.AST:
    for node in ast.walk(tree):
        if not isinstance(node, (ast.Module, ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef)):
            continue
        body = node.body
        if (
            body
            and isinstance(body[0], ast.Expr)
            and isinstance(body[0].value, ast.Constant)
            and isinstance(body[0].value.value, str)
        ):
            node.body = body[1:] or [ast.Pass()]
    return tree


def python_ast_matches(before: str, after: str) -> bool:
    """Equal modulo docstrings -- dropped from both sides, since the stripper may
    leave an empty `""` behind where `__doc__` is read at runtime."""
    try:
        a = _strip_docstrings_from_ast(ast.parse(before))
        b = _strip_docstrings_from_ast(ast.parse(after))
    except SyntaxError:
        return False
    return ast.dump(a) == ast.dump(b)


@dataclass
class Stats:
    files: int = 0
    changed: int = 0
    lines_before: int = 0
    lines_after: int = 0
    violations: list[str] = field(default_factory=list)

    @property
    def removed(self) -> int:
        return self.lines_before - self.lines_after


def process(paths: list[Path], fn, apply: bool, ast_check: bool = False) -> Stats:
    st = Stats()
    for p in paths:
        try:
            before = p.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            st.violations.append(f"{p.relative_to(REPO)}: not utf-8")
            continue

        after = collapse_blank_lines(fn(before))
        st.files += 1
        st.lines_before += before.count("\n")
        st.lines_after += after.count("\n")

        rel = p.relative_to(REPO)
        if ast_check:
            # Stronger than deletion-only: proves the code means the same thing.
            # It also tolerates the `pass` this stripper must insert where a body
            # held nothing but a docstring.
            if not python_ast_matches(before, after):
                st.violations.append(f"{rel}: AST changed")
                continue
        elif not is_subsequence("".join(after.split()), "".join(before.split())):
            st.violations.append(f"{rel}: NOT deletion-only")
            continue

        if after != before:
            st.changed += 1
            if apply:
                p.write_text(after, encoding="utf-8")
    return st


def report(label: str, st: Stats) -> None:
    print(
        f"{label:6} files={st.files:4d} changed={st.changed:4d} "
        f"lines {st.lines_before:6d} -> {st.lines_after:6d} (-{st.removed})"
    )
    for v in st.violations:
        print(f"  !! {v}")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--apply", action="store_true", help="rewrite files in place")
    ap.add_argument("--keep-license", action="store_true", default=True)
    ap.add_argument(
        "--no-keep-license",
        dest="keep_license",
        action="store_false",
        help="also remove the Copyright header (breaks dev/check-license-header.sh)",
    )
    args = ap.parse_args()

    groups = [
        ("rust", tracked(["*.rs"]), lambda s: strip_rust(s, args.keep_license), False),
        ("ts", tracked(["*.ts", "*.tsx"]), lambda s: strip_ts(s, args.keep_license), False),
        ("py", tracked(["*.py"]), strip_python, True),
    ]

    total_violations = 0
    for label, paths, fn, ast_check in groups:
        st = process(paths, fn, args.apply, ast_check)
        report(label, st)
        total_violations += len(st.violations)

    if total_violations:
        print(f"\nFAILED: {total_violations} guard violation(s); nothing unsafe was written.")
        return 1
    print("\nAll files satisfied the deletion-only invariant.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
