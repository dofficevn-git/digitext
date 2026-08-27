#!/usr/bin/env python3
"""Strip comments from the non-source formats, each behind a semantic guard.

A broken workflow or migration would not surface until run time, so nothing is
written unless the format's guard passes:

    yaml/yml  re-parses to an identical document (needs PyYAML; skipped without)
    toml      re-parses to an identical table (tomllib)
    sh        passes `bash -n`
    sql/css/tf/pbtxt   satisfies the deletion-only invariant

    python3 dev/obfuscate/run_strip_config.py           # dry run + guards
    python3 dev/obfuscate/run_strip_config.py --apply   # rewrite in place
"""

from __future__ import annotations

import argparse
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from run_strip import REPO, is_subsequence, tracked  # noqa: E402
from strip_config import (  # noqa: E402
    drop_emptied_lines,
    strip_css,
    strip_pbtxt,
    strip_sh,
    strip_sql,
    strip_tf,
    strip_toml,
    strip_yaml,
)

try:
    import yaml

    HAVE_YAML = True
except ImportError:  # pragma: no cover - CI installs PyYAML
    HAVE_YAML = False


def _scalar_ok(before: str, after: str) -> bool:
    """A leaf string may only change by having its own shell comments stripped."""
    if before == after:
        return True
    if drop_emptied_lines(before, strip_sh(before)) != after:
        return False
    return guard_sh(before, after) is None


def _docs_equal(a, b, path: str = "") -> str | None:
    if isinstance(a, str) and isinstance(b, str):
        return None if _scalar_ok(a, b) else f"scalar changed at {path or '<root>'}"
    if type(a) is not type(b):
        return f"type changed at {path or '<root>'}"
    if isinstance(a, dict):
        if a.keys() != b.keys():
            return f"keys changed at {path or '<root>'}"
        return next((w for k in a if (w := _docs_equal(a[k], b[k], f"{path}.{k}"))), None)
    if isinstance(a, list):
        if len(a) != len(b):
            return f"length changed at {path or '<root>'}"
        return next(
            (w for i, (u, v) in enumerate(zip(a, b)) if (w := _docs_equal(u, v, f"{path}[{i}]"))),
            None,
        )
    return None if a == b else f"value changed at {path or '<root>'}"


def guard_yaml(before: str, after: str) -> str | None:
    if not HAVE_YAML:
        return None  # cannot verify -> caller downgrades to deletion-only
    try:
        a = list(yaml.safe_load_all(before))
        b = list(yaml.safe_load_all(after))
    except yaml.YAMLError as e:
        return f"no longer parses: {e.__class__.__name__}"
    if len(a) != len(b):
        return "document count changed"
    return next((w for u, v in zip(a, b) if (w := _docs_equal(u, v))), None)


def guard_toml(before: str, after: str) -> str | None:
    try:
        a = tomllib.loads(before)
        b = tomllib.loads(after)
    except tomllib.TOMLDecodeError as e:
        return f"no longer parses: {e}"
    return None if a == b else "parsed table changed"


def guard_sh(_before: str, after: str) -> str | None:
    with tempfile.NamedTemporaryFile("w", suffix=".sh", delete=False) as fh:
        fh.write(after)
        tmp = fh.name
    try:
        r = subprocess.run(["bash", "-n", tmp], capture_output=True, text=True)
    finally:
        Path(tmp).unlink(missing_ok=True)
    return None if r.returncode == 0 else f"bash -n: {r.stderr.strip().splitlines()[:1]}"


def guard_deletion_only(before: str, after: str) -> str | None:
    if is_subsequence("".join(after.split()), "".join(before.split())):
        return None
    return "NOT deletion-only"


GROUPS = [
    ("yaml", ["*.yml", "*.yaml"], strip_yaml, guard_yaml),
    ("toml", ["*.toml"], strip_toml, guard_toml),
    ("sh", ["*.sh"], strip_sh, guard_sh),
    ("sql", ["*.sql"], strip_sql, guard_deletion_only),
    ("css", ["*.css"], strip_css, guard_deletion_only),
    ("tf", ["*.tf"], strip_tf, guard_deletion_only),
    ("pbtxt", ["*.pbtxt"], strip_pbtxt, guard_deletion_only),
]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--apply", action="store_true")
    args = ap.parse_args()

    failures = 0
    for label, pats, fn, guard in GROUPS:
        files = changed = 0
        before_lines = after_lines = 0
        problems: list[str] = []

        for p in tracked(pats):
            before = p.read_text(encoding="utf-8")
            after = drop_emptied_lines(before, fn(before))
            files += 1
            before_lines += before.count("\n")
            after_lines += after.count("\n")

            rel = p.relative_to(REPO)
            why = guard(before, after) or guard_deletion_only(before, after)
            if why:
                problems.append(f"{rel}: {why}")
                continue
            if after != before:
                changed += 1
                if args.apply:
                    p.write_text(after, encoding="utf-8")

        note = "" if (label != "yaml" or HAVE_YAML) else "  [PyYAML absent: deletion-only]"
        print(
            f"{label:6} files={files:3d} changed={changed:3d} "
            f"lines {before_lines:5d} -> {after_lines:5d} (-{before_lines - after_lines}){note}"
        )
        for pr in problems:
            print(f"  !! {pr}")
        failures += len(problems)

    if failures:
        print(f"\nFAILED: {failures} guard violation(s); nothing unsafe was written.")
        return 1
    print("\nAll formats passed their semantic guard.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
