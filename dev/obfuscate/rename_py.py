#!/usr/bin/env python3
"""Local-variable obfuscation for the Python sources.

Python resolves names at run time, so most mistakes here would not surface
until a script is actually executed. The rules below keep the pass to the cases
that cannot change behaviour:

* **Parameters are never renamed.** Any caller may pass them by keyword, and
  nothing in the AST tells us whether one does.
* **Only function-local names.** Module- and class-level names may be imported
  or read as attributes elsewhere.
* **Nothing from a dynamic-lookup function.** `locals()`, `globals()`, `vars()`,
  `eval` and `exec` read names as data.
* **Nothing quoted anywhere in the file**, which covers `getattr(o, "name")`
  and any other string-keyed access.

Edits are applied by source position rather than by unparsing, so formatting
survives and the diff stays reviewable.

    python3 dev/obfuscate/rename_py.py           # dry run
    python3 dev/obfuscate/rename_py.py --apply   # rewrite in place
"""

from __future__ import annotations

import argparse
import ast
import builtins
import json
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]

DYNAMIC = {"locals", "globals", "vars", "eval", "exec", "compile"}

FUNC = (ast.FunctionDef, ast.AsyncFunctionDef)


# The obfuscator itself stays readable -- see the note in run_strip.py.
EXCLUDE_PREFIXES = ("dev/obfuscate/",)


def tracked() -> list[Path]:
    out = subprocess.run(
        ["git", "ls-files", "*.py"], cwd=REPO, capture_output=True, text=True, check=True
    ).stdout.split()
    return [REPO / p for p in out if not p.startswith(EXCLUDE_PREFIXES)]


def string_words(tree: ast.AST) -> set[str]:
    words: set[str] = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Constant) and isinstance(node.value, str):
            cur = ""
            for ch in node.value:
                if ch.isalnum() or ch == "_":
                    cur += ch
                elif cur:
                    words.add(cur)
                    cur = ""
            if cur:
                words.add(cur)
    return words


def all_param_names(tree: ast.AST) -> set[str]:
    names: set[str] = set()
    for node in ast.walk(tree):
        if isinstance(node, FUNC):
            a = node.args
            for arg in [*a.posonlyargs, *a.args, *a.kwonlyargs]:
                names.add(arg.arg)
            for arg in (a.vararg, a.kwarg):
                if arg:
                    names.add(arg.arg)
        elif isinstance(node, ast.Lambda):
            a = node.args
            for arg in [*a.posonlyargs, *a.args, *a.kwonlyargs]:
                names.add(arg.arg)
    return names


def module_and_class_names(tree: ast.Module) -> set[str]:
    """Names visible outside a function body: importable or attribute-reachable."""
    names: set[str] = set()

    def bound_by(stmt: ast.stmt) -> set[str]:
        out: set[str] = set()
        for t in ast.walk(stmt):
            if isinstance(t, ast.Name) and isinstance(t.ctx, ast.Store):
                out.add(t.id)
            elif isinstance(t, (ast.Import, ast.ImportFrom)):
                for al in t.names:
                    out.add(al.asname or al.name.split(".")[0])
            elif isinstance(t, (*FUNC, ast.ClassDef)):
                out.add(t.name)
            elif isinstance(t, ast.ExceptHandler) and t.name:
                out.add(t.name)
        return out

    for stmt in tree.body:
        if isinstance(stmt, (*FUNC,)):
            names.add(stmt.name)
            continue
        if isinstance(stmt, ast.ClassDef):
            names.add(stmt.name)
            for sub in stmt.body:
                if isinstance(sub, (*FUNC, ast.ClassDef)):
                    names.add(sub.name)
                else:
                    names |= bound_by(sub)
            continue
        names |= bound_by(stmt)
    return names


def uses_dynamic(fn: ast.AST) -> bool:
    for node in ast.walk(fn):
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Name):
            if node.func.id in DYNAMIC:
                return True
        if isinstance(node, ast.Name) and node.id in DYNAMIC:
            return True
    return False


def local_bindings(fn: ast.AST) -> set[str]:
    """Names the function body binds, excluding parameters and handler names."""
    names: set[str] = set()
    for node in ast.walk(fn):
        if isinstance(node, ast.Name) and isinstance(node.ctx, (ast.Store, ast.Del)):
            names.add(node.id)
        elif isinstance(node, ast.NamedExpr) and isinstance(node.target, ast.Name):
            names.add(node.target.id)
    return names


def top_level_functions(tree: ast.Module) -> list[ast.AST]:
    """Outermost functions only; nested defs are renamed as part of their parent."""
    out: list[ast.AST] = []
    for stmt in tree.body:
        if isinstance(stmt, FUNC):
            out.append(stmt)
        elif isinstance(stmt, ast.ClassDef):
            out.extend(s for s in stmt.body if isinstance(s, FUNC))
    return out


class Counter:
    def __init__(self) -> None:
        self.n = 0

    def fresh(self) -> str:
        name = f"p_{self.n:x}"
        self.n += 1
        return name


def already(name: str) -> bool:
    return name.startswith("p_") and name[2:].isalnum()


def all_names(tree: ast.AST) -> set[str]:
    return {n.id for n in ast.walk(tree) if isinstance(n, ast.Name)}


def plan_file(
    tree: ast.Module, counter: Counter
) -> tuple[list[tuple[int, int, int, str]], dict[str, str]]:
    """Return positional edits plus the new-name -> old-name inverse map."""
    excluded = (
        all_param_names(tree)
        | module_and_class_names(tree)
        | string_words(tree)
        | set(dir(builtins))
    )
    taken = all_names(tree)
    edits: list[tuple[int, int, int, str]] = []
    inverse: dict[str, str] = {}

    for fn in top_level_functions(tree):
        if uses_dynamic(fn):
            continue
        candidates = {
            n
            for n in local_bindings(fn)
            if n not in excluded and not n.startswith("__") and not already(n)
        }
        if not candidates:
            continue
        mapping: dict[str, str] = {}
        for name in sorted(candidates):
            new = counter.fresh()
            while new in taken:  # never collide with a name the file already uses
                new = counter.fresh()
            taken.add(new)
            mapping[name] = new
            inverse[new] = name
        for node in ast.walk(fn):
            if isinstance(node, ast.Name) and node.id in mapping:
                edits.append(
                    (node.lineno, node.col_offset, node.end_col_offset, mapping[node.id])
                )
    return edits, inverse


def apply_edits(src: str, edits: list[tuple[int, int, int, str]]) -> str:
    """Apply edits on UTF-8 bytes.

    `ast` reports `col_offset` as a byte offset into the encoded line, not a
    character index. Slicing the `str` directly corrupts every line containing
    non-ASCII text -- which, in this codebase, is most of them.
    """
    lines = [ln.encode("utf-8") for ln in src.splitlines(keepends=True)]
    for lineno, col, end_col, new in sorted(edits, key=lambda e: (-e[0], -e[1])):
        line = lines[lineno - 1]
        lines[lineno - 1] = line[:col] + new.encode("utf-8") + line[end_col:]
    return b"".join(lines).decode("utf-8")


def undo(tree: ast.AST, inverse: dict[str, str]) -> str:
    """Dump the tree with generated names mapped back to the originals.

    Comparing this against the untouched source proves the rewrite was a pure
    rename: any other difference in structure shows up as a dump mismatch.
    """

    class Fold(ast.NodeTransformer):
        def visit_Name(self, node: ast.Name):  # noqa: N802
            node.id = inverse.get(node.id, node.id)
            return node

    return ast.dump(Fold().visit(tree))


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--apply", action="store_true")
    ap.add_argument("--items", action="store_true", help="rename module-level defs/classes")
    ap.add_argument("--strings", action="store_true", help="base64-encode Vietnamese literals")
    args = ap.parse_args()

    if args.strings:
        return run_strings(args.apply)
    if args.items:
        return run_items(args.apply)

    counter = Counter()
    n_files = n_renamed = n_changed = n_occurrences = 0
    problems: list[str] = []

    for path in tracked():
        src = path.read_text(encoding="utf-8")
        try:
            tree = ast.parse(src)
        except SyntaxError as e:
            problems.append(f"{path.relative_to(REPO)}: parse: {e}")
            continue
        n_files += 1

        edits, inverse = plan_file(tree, counter)
        if not edits:
            continue
        out = apply_edits(src, edits)

        # Guard: the result must parse, and must be the same program once the
        # generated names are folded together.
        try:
            after = ast.parse(out)
        except SyntaxError as e:
            problems.append(f"{path.relative_to(REPO)}: broke syntax: {e}")
            continue
        if undo(after, inverse) != ast.dump(ast.parse(src)):
            problems.append(f"{path.relative_to(REPO)}: AST changed beyond renaming")
            continue

        n_renamed += len(inverse)
        n_occurrences += len(edits)
        n_changed += 1
        if args.apply:
            path.write_text(out, encoding="utf-8")

    print(f"files parsed    : {n_files}")
    print(f"names renamed   : {n_renamed}")
    print(f"occurrences     : {n_occurrences}")
    print(f"files changed   : {n_changed}")
    for p in problems:
        print(f"  !! {p}")
    if problems:
        print(f"\nFAILED: {len(problems)} problem(s); nothing unsafe was written.")
        return 1
    if not args.apply:
        print("dry run -- pass --apply to write.")
    return 0




# ---------------------------------------------------------------- items

def _local_modules(paths) -> set[str]:
    """Module and package names that belong to this repo."""
    names: set[str] = set()
    for p in paths:
        rel = p.relative_to(REPO)
        names.add(rel.stem)
        for part in rel.parts[:-1]:
            names.add(part)
    return names


def _collect_scope(
    trees: dict[Path, ast.Module], local_mods: set[str] | None = None
) -> tuple[set[str], set[str]]:
    """Module-level def/class names, and everything that disqualifies a name.

    Python resolves attributes at run time, so `mod.helper()` and `obj.helper`
    are indistinguishable from here -- any name ever written after a dot is off
    limits. So is any name quoted anywhere (getattr, importlib, argparse dest)
    and any name that is also a parameter somewhere.
    """
    counts: dict[str, int] = {}
    banned: set[str] = set()

    for tree in trees.values():
        for st in tree.body:
            if isinstance(st, (*FUNC, ast.ClassDef)):
                counts[st.name] = counts.get(st.name, 0) + 1
        for node in ast.walk(tree):
            if isinstance(node, ast.Attribute):
                banned.add(node.attr)
            elif isinstance(node, ast.Constant) and isinstance(node.value, str):
                cur = ""
                for ch in node.value:
                    if ch.isalnum() or ch == "_":
                        cur += ch
                    elif cur:
                        banned.add(cur)
                        cur = ""
                if cur:
                    banned.add(cur)
            elif isinstance(node, (*FUNC, ast.Lambda)):
                a = node.args
                for arg in [*a.posonlyargs, *a.args, *a.kwonlyargs]:
                    banned.add(arg.arg)
                for arg in (a.vararg, a.kwarg):
                    if arg:
                        banned.add(arg.arg)
            elif isinstance(node, ast.Import):
                for al in node.names:
                    banned.add(al.name.split(".")[0])
                    if al.asname:
                        banned.add(al.asname)
            elif isinstance(node, ast.ImportFrom):
                # `from numpy import array` names a third-party symbol, but
                # `from gate import decide` names one of ours -- and that is
                # exactly the case worth renaming, since the import site is
                # rewritten along with the definition.
                root = (node.module or "").split(".")[0]
                foreign = node.level == 0 and root not in (local_mods or set())
                for al in node.names:
                    if foreign:
                        banned.add(al.name.split(".")[0])
                    if al.asname:
                        banned.add(al.asname)

    unique = {n for n, c in counts.items() if c == 1}
    eligible = {
        n
        for n in unique
        if n not in banned and not n.startswith("__") and not n.startswith("q_")
    }
    return eligible, banned


def _def_name_span(line_bytes: bytes, name: str) -> tuple[int, int] | None:
    """Byte span of `name` right after `def `/`class ` on a definition line."""
    for kw in (b"def ", b"class "):
        i = line_bytes.find(kw)
        if i == -1:
            continue
        start = i + len(kw)
        while start < len(line_bytes) and line_bytes[start : start + 1].isspace():
            start += 1
        end = start + len(name.encode("utf-8"))
        if line_bytes[start:end] == name.encode("utf-8"):
            return start, end
    return None


def plan_items(
    tree: ast.Module, src: str, mapping: dict[str, str]
) -> list[tuple[int, int, int, str]]:
    """Positional edits renaming module-level definitions and their references."""
    edits: list[tuple[int, int, int, str]] = []
    lines = [ln.encode("utf-8") for ln in src.splitlines(keepends=True)]

    for st in tree.body:
        if isinstance(st, (*FUNC, ast.ClassDef)) and st.name in mapping:
            span = _def_name_span(lines[st.lineno - 1], st.name)
            if span:
                edits.append((st.lineno, span[0], span[1], mapping[st.name]))

    for node in ast.walk(tree):
        if isinstance(node, ast.Name) and node.id in mapping:
            edits.append(
                (node.lineno, node.col_offset, node.end_col_offset, mapping[node.id])
            )
        elif isinstance(node, ast.ImportFrom):
            for al in node.names:
                if al.name in mapping and al.asname is None and al.end_col_offset:
                    edits.append(
                        (al.lineno, al.col_offset, al.end_col_offset, mapping[al.name])
                    )
    return edits


def undo_items(tree: ast.AST, inverse: dict[str, str]) -> str:
    """Inverse of the items rename, for the equivalence guard.

    Definition names and `from x import y` aliases are plain string attributes,
    not Name nodes, so the Name-only inverse would report every file as changed.
    """

    class Fold(ast.NodeTransformer):
        def visit_Name(self, node: ast.Name):  # noqa: N802
            node.id = inverse.get(node.id, node.id)
            return node

        def _named(self, node):
            node.name = inverse.get(node.name, node.name)
            self.generic_visit(node)
            return node

        visit_FunctionDef = _named  # noqa: N815
        visit_AsyncFunctionDef = _named  # noqa: N815
        visit_ClassDef = _named  # noqa: N815

        def visit_alias(self, node: ast.alias):
            node.name = inverse.get(node.name, node.name)
            return node

    return ast.dump(Fold().visit(tree))


def run_items(apply: bool) -> int:
    paths = tracked()
    trees: dict[Path, ast.Module] = {}
    srcs: dict[Path, str] = {}
    for p in paths:
        s = p.read_text(encoding="utf-8")
        try:
            trees[p] = ast.parse(s)
        except SyntaxError as e:
            print(f"  !! {p.relative_to(REPO)}: parse: {e}")
            return 1
        srcs[p] = s

    eligible, _ = _collect_scope(trees, _local_modules(paths))
    counter = Counter()
    mapping: dict[str, str] = {}
    for n in sorted(eligible):
        mapping[n] = f"q_{counter.n:x}"
        counter.n += 1
    inverse = {v: k for k, v in mapping.items()}

    changed = occurrences = 0
    problems: list[str] = []
    for p in paths:
        edits = plan_items(trees[p], srcs[p], mapping)
        if not edits:
            continue
        out = apply_edits(srcs[p], edits)
        try:
            after = ast.parse(out)
        except SyntaxError as e:
            problems.append(f"{p.relative_to(REPO)}: broke syntax: {e}")
            continue
        if undo_items(after, inverse) != ast.dump(ast.parse(srcs[p])):
            problems.append(f"{p.relative_to(REPO)}: AST changed beyond renaming")
            continue
        occurrences += len(edits)
        changed += 1
        if apply:
            p.write_text(out, encoding="utf-8")

    print(f"items eligible  : {len(mapping)}")
    print(f"occurrences     : {occurrences}")
    print(f"files changed   : {changed}")
    for pr in problems:
        print(f"  !! {pr}")
    if problems:
        print(f"\nFAILED: {len(problems)} problem(s); nothing unsafe was written.")
        return 1
    if not apply:
        print("dry run -- pass --apply to write.")
    return 0




# -------------------------------------------------------------- strings

_VIET = set("àáâãèéêìíòóôõùúỳăđĩũơưạảấầẩẫậắằẳẵặẹẻẽếềểễệỉịọỏốồổỗộớờởỡợụủứừửữựỵỷỹ")
_VIET |= {c.upper() for c in _VIET}


def _has_viet(s: str) -> bool:
    return any(c in _VIET for c in s)


def _is_b64(s: str) -> bool:
    # `str.isalnum()` is Unicode-aware, so 'Đ' and 'công' would pass as base64
    # and never get encoded. Base64 is ASCII by definition.
    return bool(s) and all((c.isascii() and c.isalnum()) or c in "+/=" for c in s)


def encode_format(s: str) -> str:
    """Encode the literal runs of a format string, leaving `{...}` spans and the
    `{{`/`}}` escapes untouched so the placeholder structure is preserved."""
    import base64

    out: list[str] = []
    run = ""
    i, n = 0, len(s)

    def flush() -> None:
        nonlocal run
        if run:
            if _has_viet(run):
                out.append(base64.b64encode(run.encode("utf-8")).decode("ascii"))
            else:
                out.append(run)
            run = ""

    while i < n:
        c = s[i]
        if c == "{" and s[i : i + 2] == "{{":
            run += "{{"
            i += 2
        elif c == "}" and s[i : i + 2] == "}}":
            run += "}}"
            i += 2
        elif c == "{":
            flush()
            j = s.find("}", i)
            j = n - 1 if j == -1 else j
            out.append(s[i : j + 1])
            i = j + 1
        else:
            run += c
            i += 1
    flush()
    return "".join(out)


def encodable(s: str) -> bool:
    return _has_viet(s) and not _is_b64(s)


def plan_strings(tree: ast.Module) -> list[tuple[int, int, int, str]]:
    """Positional edits replacing Vietnamese literals with their base64 form."""
    import base64

    edits: list[tuple[int, int, int, str]] = []

    # A Constant inside an f-string spans only the literal text, with no quotes
    # of its own, so its replacement must not add any either. `ast.walk` yields
    # those Constants independently of their JoinedStr, hence the id set.
    fstring_parts: set[int] = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.JoinedStr):
            for sub in node.values:
                if isinstance(sub, ast.Constant):
                    fstring_parts.add(id(sub))

    for node in ast.walk(tree):
        if not (isinstance(node, ast.Constant) and isinstance(node.value, str)):
            continue
        if not encodable(node.value):
            continue
        if node.end_lineno != node.lineno:
            continue  # multi-line literal: the edit model is line-based
        token = encode_format(node.value)
        if token == node.value:
            continue
        if id(node) in fstring_parts:
            # Raw substitution: quotes belong to the enclosing f-string.
            if '"' in token or "'" in token or "\\" in token:
                continue
            edits.append((node.lineno, node.col_offset, node.end_col_offset, token))
        else:
            edits.append(
                (node.lineno, node.col_offset, node.end_col_offset, json.dumps(token))
            )
    return edits


def run_strings(apply: bool) -> int:
    n = changed = 0
    problems: list[str] = []
    for p in tracked():
        src = p.read_text(encoding="utf-8")
        try:
            tree = ast.parse(src)
        except SyntaxError as e:
            problems.append(f"{p.relative_to(REPO)}: parse: {e}")
            continue
        edits = plan_strings(tree)
        if not edits:
            continue
        out = apply_edits(src, edits)
        try:
            ast.parse(out)
        except SyntaxError as e:
            problems.append(f"{p.relative_to(REPO)}: broke syntax: {e}")
            continue
        n += len(edits)
        changed += 1
        if apply:
            p.write_text(out, encoding="utf-8")

    print(f"literals encoded: {n}")
    print(f"files changed   : {changed}")
    for pr in problems:
        print(f"  !! {pr}")
    if problems:
        return 1
    if not apply:
        print("dry run -- pass --apply to write.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
