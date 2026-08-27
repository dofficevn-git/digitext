"""Comment strippers for the non-source formats: yaml, toml, sql, sh, css, tf, pbtxt.

These carry weaker safety nets than Rust or TS -- a mangled shell script or
migration fails at run time, not build time -- so each format is paired with a
semantic guard in run_strip_config.py: YAML/TOML must re-parse to an identical
structure, shell must pass `bash -n`, and everything else must satisfy the
deletion-only invariant.
"""

from __future__ import annotations

import re

# `#` opens a comment only at line start or after whitespace; `a#b` is a value.
_HASH_OK_BEFORE = set(" \t")


def _strip_hash_lang(src: str, keep_shebang: bool = True) -> str:
    """Strip `#` comments from a quote-using, non-block-scalar language."""
    out: list[str] = []
    i, n = 0, len(src)
    line_start = 0

    while i < n:
        c = src[i]

        if c == "\n":
            out.append(c)
            i += 1
            line_start = i
            continue

        if c in "\"'":
            j = i + 1
            while j < n:
                if c == '"' and src[j] == "\\":
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
            continue

        if c == "#":
            at_line_start = not src[line_start:i].strip()
            prev = src[i - 1] if i else "\n"
            if at_line_start and keep_shebang and i == 0 and src[1:2] == "!":
                end = src.find("\n", i)
                end = n if end == -1 else end
                out.append(src[i:end])
                i = end
                continue
            if at_line_start or prev in _HASH_OK_BEFORE:
                end = src.find("\n", i)
                end = n if end == -1 else end
                # Drop trailing space left by an end-of-line comment.
                while out and out[-1] in " \t":
                    out.pop()
                i = end
                continue

        out.append(c)
        i += 1

    return "".join(out)


def strip_toml(src: str) -> str:
    return _strip_hash_lang(src, keep_shebang=False)


def strip_pbtxt(src: str) -> str:
    return _strip_hash_lang(src, keep_shebang=False)


# Opens a literal/folded scalar, either as `key: |` or as a sequence item `- |`.
_BLOCK_OPEN = re.compile(r"(?:^|\s)(?:-\s*|[\w.\"'-]+:\s*)[|>][-+0-9]*\s*(?:#.*)?$")

# A scalar body is only rewritten when it is clearly shell; `#` means other
# things elsewhere (a markdown heading, a colour, a Dockerfile directive).
_SHELL_KEY = re.compile(r"(?:^|\s)(run|command|entrypoint|script|args):")
_SHELL_BODY = re.compile(r"^\s*(#!|set -[eux]|sudo |apt-get |python |npm |cargo |bash )")


def strip_yaml(src: str) -> str:
    """Strip `#` comments outside quotes; rewrite block scalars only if shell.

    A `run: |` body is data, not YAML syntax. Editing a non-shell one would
    change the parsed document, which the semantic guard rejects.
    """
    out: list[str] = []
    lines = src.splitlines(keepends=True)
    i, n = 0, len(lines)

    while i < n:
        line = lines[i]

        if _BLOCK_OPEN.search(line.rstrip("\n")):
            indent = len(line) - len(line.lstrip())
            out.append(_strip_hash_lang(line, keep_shebang=False))
            i += 1

            body: list[str] = []
            while i < n:
                b = lines[i]
                if b.strip() and (len(b) - len(b.lstrip())) <= indent:
                    break
                body.append(b)
                i += 1

            text = "".join(body)
            is_shell = _SHELL_KEY.search(line) or _SHELL_BODY.search(text)
            out.append(drop_emptied_lines(text, strip_sh(text)) if is_shell else text)
            continue

        out.append(_strip_hash_lang(line, keep_shebang=False))
        i += 1

    return "".join(out)


def drop_emptied_lines(before: str, after: str) -> str:
    """Delete lines the strip blanked out, keeping blank lines that were already blank."""
    b = before.splitlines(keepends=True)
    a = after.splitlines(keepends=True)
    if len(a) != len(b):
        return after
    return "".join(x for x, y in zip(a, b) if x.strip() or not y.strip())


def strip_sh(src: str) -> str:
    """Strip `#` comments from shell, preserving the shebang and heredoc bodies."""
    out: list[str] = []
    lines = src.splitlines(keepends=True)
    i, n = 0, len(lines)
    heredoc_re = re.compile(r"<<-?\s*(['\"]?)([A-Za-z_][A-Za-z0-9_]*)\1")

    while i < n:
        line = lines[i]

        if i == 0 and line.startswith("#!"):
            out.append(line)
            i += 1
            continue

        m = heredoc_re.search(line)
        out.append(_strip_hash_lang(line, keep_shebang=False))
        i += 1

        if m:
            # Heredoc bodies are data: copy through until the terminator.
            tag, dashed = m.group(2), "<<-" in line
            while i < n:
                body = lines[i]
                out.append(body)
                i += 1
                if (body.strip() if dashed else body.rstrip("\n")) == tag:
                    break

    return "".join(out)


def strip_sql(src: str) -> str:
    """Strip `--` and `/* */` from SQL, respecting quotes and `$tag$` bodies."""
    out: list[str] = []
    i, n = 0, len(src)

    while i < n:
        c = src[i]

        # Postgres dollar-quoted string
        if c == "$":
            m = re.match(r"\$[A-Za-z_0-9]*\$", src[i:])
            if m:
                tag = m.group(0)
                end = src.find(tag, i + len(tag))
                end = n if end == -1 else end + len(tag)
                out.append(src[i:end])
                i = end
                continue

        if c in "'\"":
            j = i + 1
            while j < n:
                if src[j] == c:
                    # '' and "" are escapes inside SQL literals.
                    if j + 1 < n and src[j + 1] == c:
                        j += 2
                        continue
                    j += 1
                    break
                j += 1
            out.append(src[i:j])
            i = j
            continue

        if src.startswith("--", i):
            end = src.find("\n", i)
            end = n if end == -1 else end
            while out and out[-1] in " \t":
                out.pop()
            i = end
            continue

        if src.startswith("/*", i):
            end = src.find("*/", i + 2)
            end = n if end == -1 else end + 2
            out.append(" ")
            i = end
            continue

        out.append(c)
        i += 1

    return "".join(out)


def strip_css(src: str) -> str:
    """Strip `/* */` from CSS. `//` is not a comment in plain CSS."""
    out: list[str] = []
    i, n = 0, len(src)

    while i < n:
        c = src[i]

        if c in "'\"":
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
            continue

        if src.startswith("/*", i):
            end = src.find("*/", i + 2)
            end = n if end == -1 else end + 2
            out.append(" ")
            i = end
            continue

        out.append(c)
        i += 1

    return "".join(out)


def strip_tf(src: str) -> str:
    """Terraform/HCL accepts `#`, `//` and `/* */`."""
    src = _strip_hash_lang(src, keep_shebang=False)
    out: list[str] = []
    i, n = 0, len(src)

    while i < n:
        c = src[i]

        if c == '"':
            j = i + 1
            while j < n:
                if src[j] == "\\":
                    j += 2
                    continue
                if src[j] == '"':
                    j += 1
                    break
                if src[j] == "\n":
                    break
                j += 1
            out.append(src[i:j])
            i = j
            continue

        if src.startswith("//", i):
            end = src.find("\n", i)
            end = n if end == -1 else end
            while out and out[-1] in " \t":
                out.pop()
            i = end
            continue

        if src.startswith("/*", i):
            end = src.find("*/", i + 2)
            end = n if end == -1 else end + 2
            out.append(" ")
            i = end
            continue

        out.append(c)
        i += 1

    return "".join(out)
