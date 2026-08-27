"""Comment/docstring strippers driven by real lexers, not regexes.

A regex pass corrupts this repo: 124 Rust lines carry `://` inside string
literals, 19 files use raw strings, and the TS sources hold 1314 template
literal lines plus 37 regex literals. Each stripper below tracks lexer state so
that only genuine comment spans are removed.
"""

from __future__ import annotations

import ast
import io
import re
import tokenize

LICENSE_MARKER = "Copyright (c) 2026 DigiText"

# eslint/tsc directives are compiler input, not prose. Removing them changes
# build behaviour, so they survive the strip.
_TS_DIRECTIVE = re.compile(r"@ts-(expect-error|ignore|nocheck)|eslint-(disable|enable)")

_IDENT_CHAR = re.compile(r"[A-Za-z0-9_]")

# Characters after which a `/` starts a regex literal rather than a division.
_REGEX_OK_AFTER = set("(,=:[!&|?{};+-*%~^<>")


def _is_license(text: str) -> bool:
    return LICENSE_MARKER in text


def strip_rust(src: str, keep_license: bool = True) -> str:
    """Remove `//`, `///`, `//!` and (nesting-aware) `/* */` comments from Rust.

    Handles raw strings (`r#"..."#`, `br##"..."##`), byte strings, escapes, and
    the char-literal / lifetime ambiguity at `'`.
    """
    out: list[str] = []
    i, n = 0, len(src)
    seen_license = False

    while i < n:
        c = src[i]

        # ---- raw string: (b)r#*" ... "#*
        if c in "rb":
            j = i
            if src[j] == "b" and j + 1 < n and src[j + 1] == "r":
                j += 1
            if src[j] == "r":
                k = j + 1
                hashes = 0
                while k < n and src[k] == "#":
                    hashes += 1
                    k += 1
                if k < n and src[k] == '"':
                    prev = src[i - 1] if i else ""
                    # Only a raw string when `r` does not continue an identifier.
                    if not (prev and _IDENT_CHAR.match(prev)):
                        close = '"' + "#" * hashes
                        end = src.find(close, k + 1)
                        end = n if end == -1 else end + len(close)
                        out.append(src[i:end])
                        i = end
                        continue

        # ---- line comment
        if c == "/" and i + 1 < n and src[i + 1] == "/":
            end = src.find("\n", i)
            end = n if end == -1 else end
            text = src[i:end]
            if keep_license and not seen_license and _is_license(text):
                out.append(text)
                seen_license = True
            i = end
            continue

        # ---- block comment (Rust nests them)
        if c == "/" and i + 1 < n and src[i + 1] == "*":
            depth, j = 1, i + 2
            while j < n and depth:
                if src.startswith("/*", j):
                    depth += 1
                    j += 2
                elif src.startswith("*/", j):
                    depth -= 1
                    j += 2
                else:
                    j += 1
            text = src[i:j]
            if keep_license and not seen_license and _is_license(text):
                out.append(text)
                seen_license = True
            else:
                out.append(" ")  # avoid gluing `a/**/b` into `ab`
            i = j
            continue

        # ---- string literal
        if c == '"':
            j = i + 1
            while j < n:
                if src[j] == "\\":
                    j += 2
                    continue
                if src[j] == '"':
                    j += 1
                    break
                j += 1
            out.append(src[i:j])
            i = j
            continue

        # ---- char literal vs lifetime
        if c == "'":
            if i + 1 < n and src[i + 1] == "\\":
                # The escape is backslash + one char, so the earliest possible
                # closing quote is i+3. Starting the scan at i+2 makes `'\\'`
                # consume its own closing quote and run on into the next
                # literal, after which comments stop being recognised.
                j = i + 3
                while j < n and src[j] != "'":
                    j += 1
                j = min(j + 1, n)
                out.append(src[i:j])
                i = j
                continue
            if i + 2 < n and src[i + 2] == "'":
                out.append(src[i : i + 3])
                i += 3
                continue
            out.append(c)  # lifetime
            i += 1
            continue

        out.append(c)
        i += 1

    return "".join(out)


def _scan_template(src: str, i: int, tmpl_stack: list[int], brace_depth: int) -> tuple[str, int]:
    """Return a template-literal chunk, stopping at `${` so nested code is lexed."""
    n = len(src)
    j = i + 1
    while j < n:
        if src[j] == "\\":
            j += 2
            continue
        if src[j] == "`":
            j += 1
            break
        if src.startswith("${", j):
            j += 2
            tmpl_stack.append(brace_depth)
            break
        j += 1
    return src[i:j], j


def strip_ts(src: str, keep_license: bool = True, keep_directives: bool = True) -> str:
    """Remove `//` and `/* */` comments from TS/TSX.

    Tracks single/double/template strings (with `${}` nesting) and separates
    regex literals from division by looking at the previous significant token.
    """
    out: list[str] = []
    i, n = 0, len(src)
    seen_license = False
    tmpl_stack: list[int] = []  # brace depth recorded at each `${` entered
    brace_depth = 0
    prev_sig = ""

    while i < n:
        c = src[i]

        if c == "{":
            brace_depth += 1
        elif c == "}":
            if tmpl_stack and brace_depth == tmpl_stack[-1]:
                tmpl_stack.pop()
                chunk, i = _scan_template(src, i, tmpl_stack, brace_depth)
                out.append(chunk)
                prev_sig = "`"
                continue
            brace_depth -= 1

        # ---- line comment
        if c == "/" and i + 1 < n and src[i + 1] == "/":
            end = src.find("\n", i)
            end = n if end == -1 else end
            text = src[i:end]
            if keep_license and not seen_license and _is_license(text):
                out.append(text)
                seen_license = True
            elif keep_directives and _TS_DIRECTIVE.search(text):
                out.append(text)
            i = end
            continue

        # ---- block comment (does not nest in JS)
        if c == "/" and i + 1 < n and src[i + 1] == "*":
            end = src.find("*/", i + 2)
            end = n if end == -1 else end + 2
            text = src[i:end]
            if keep_license and not seen_license and _is_license(text):
                out.append(text)
                seen_license = True
            elif keep_directives and _TS_DIRECTIVE.search(text):
                out.append(text)
            else:
                out.append(" ")
            i = end
            continue

        # ---- regex literal (only where a value may start)
        if c == "/" and (prev_sig in _REGEX_OK_AFTER or prev_sig == ""):
            j, in_class, ok = i + 1, False, False
            while j < n:
                d = src[j]
                if d == "\\":
                    j += 2
                    continue
                if d == "\n":
                    break
                if d == "[":
                    in_class = True
                elif d == "]":
                    in_class = False
                elif d == "/" and not in_class:
                    ok = True
                    j += 1
                    break
                j += 1
            if ok:
                while j < n and src[j].isalpha():
                    j += 1
                out.append(src[i:j])
                i = j
                prev_sig = "/"
                continue

        # ---- quoted strings
        if c in "\"'":
            j = i + 1
            while j < n:
                if src[j] == "\\":
                    j += 2
                    continue
                if src[j] == c:
                    j += 1
                    break
                if src[j] == "\n":
                    break
                j += 1
            out.append(src[i:j])
            i = j
            prev_sig = '"'
            continue

        # ---- template literal
        if c == "`":
            chunk, i = _scan_template(src, i, tmpl_stack, brace_depth)
            out.append(chunk)
            prev_sig = "`"
            continue

        out.append(c)
        if not c.isspace():
            prev_sig = c
        i += 1

    return "".join(out)


def strip_python(src: str, keep_shebang: bool = True) -> str:
    """Remove `#` comments and docstrings via the stdlib tokenizer + AST."""
    shebang = ""
    if keep_shebang and src.startswith("#!"):
        nl = src.find("\n")
        shebang, src = (src[: nl + 1], src[nl + 1 :]) if nl != -1 else (src, "")

    # 1. Docstrings: locate via the AST, blank them out by line range.
    try:
        tree = ast.parse(src)
    except SyntaxError:
        return shebang + src

    # 32 scripts here read `__doc__` at runtime (`argparse(description=__doc__...)`),
    # so for those the module docstring is program data, not prose. Emptying it
    # keeps `__doc__` a string while leaking nothing.
    uses_doc = any(isinstance(n, ast.Name) and n.id == "__doc__" for n in ast.walk(tree))

    doc_spans: set[tuple[int, int]] = set()
    repl_at: dict[int, str] = {}
    for node in ast.walk(tree):
        if not isinstance(node, (ast.Module, ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef)):
            continue
        body = node.body
        if not (
            body
            and isinstance(body[0], ast.Expr)
            and isinstance(body[0].value, ast.Constant)
            and isinstance(body[0].value.value, str)
        ):
            continue
        doc = body[0]
        doc_spans.add((doc.lineno, doc.end_lineno or doc.lineno))
        if isinstance(node, ast.Module):
            if uses_doc:
                repl_at[doc.lineno] = '""\n'
        elif len(body) == 1:
            # A body that was only a docstring still needs a statement.
            repl_at[doc.lineno] = " " * doc.col_offset + "pass\n"

    lines = src.splitlines(keepends=True)
    for start, end in doc_spans:
        for ln in range(start, end + 1):
            lines[ln - 1] = repl_at.get(ln, "") if ln == start else ""
    src = "".join(lines)

    # 2. `#` comments: tokenize is quote- and f-string-aware.
    try:
        toks = list(tokenize.generate_tokens(io.StringIO(src).readline))
    except (tokenize.TokenError, IndentationError, SyntaxError):
        return shebang + src

    drop = [(t.start[0], t.start[1]) for t in toks if t.type == tokenize.COMMENT]
    lines = src.splitlines(keepends=True)
    for row, col in sorted(drop, reverse=True):
        keep = lines[row - 1][:col].rstrip()
        lines[row - 1] = (keep + "\n") if keep else ""
    return shebang + "".join(lines)


def collapse_blank_lines(src: str, limit: int = 1) -> str:
    """Squeeze runs of blank lines left behind by removed comment blocks."""
    return re.sub(r"\n{%d,}" % (limit + 2), "\n" * (limit + 1), src)
