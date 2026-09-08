"""Adversarial cases for the strippers, drawn from patterns present in this repo.

Stdlib unittest so CI can pick them up with `python -m unittest discover`.
"""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from strip_comments import (  # noqa: E402
    collapse_blank_lines,
    strip_python,
    strip_rust,
    strip_ts,
)

LIC = "// Copyright (c) 2026 DigiText. All rights reserved."


class TestRust(unittest.TestCase):
    def test_url_in_string_survives(self):
        src = 'let e = "https://example.com/DigiText";'
        self.assertEqual(strip_rust(src), src)

    def test_raw_string_with_comment_markers_survives(self):
        src = 'let q = r#"SELECT 1 -- x /* y */ // z"#;'
        self.assertEqual(strip_rust(src), src)

    def test_hashed_raw_string_survives(self):
        src = 'let q = r##"a "# b"##;'
        self.assertEqual(strip_rust(src), src)

    def test_identifier_ending_in_r_is_not_a_raw_string(self):
        src = 'let scanner = "x"; let cur = "y";'
        self.assertEqual(strip_rust(src), src)

    def test_line_comment_removed_newline_kept(self):
        self.assertEqual(
            strip_rust("let a = 1; // note\nlet b = 2;\n"), "let a = 1; \nlet b = 2;\n"
        )

    def test_doc_comments_removed(self):
        src = "/// Doc line.\n//! Inner doc.\npub fn f() {}\n"
        self.assertEqual(strip_rust(src), "\n\npub fn f() {}\n")

    def test_nested_block_comment(self):
        # The inner `*/` must not end the comment early; the whole span collapses.
        self.assertEqual(strip_rust("a/* x /* y */ z */b"), "a b")

    def test_block_comment_does_not_glue_tokens(self):
        self.assertNotIn("ab", strip_rust("a/**/b"))

    def test_lifetime_not_treated_as_char(self):
        src = "fn f<'a>(x: &'a str) -> &'a str { x }"
        self.assertEqual(strip_rust(src), src)

    def test_char_literal_slash_survives(self):
        src = "let c = '/'; let d = '\\''; let e = '\\n';"
        self.assertEqual(strip_rust(src), src)

    def test_escaped_backslash_char_does_not_swallow_the_next_literal(self):
        # `'\\'` used to eat its own closing quote and run on, after which every
        # comment further down the file was left in place.
        src = "if matches!(c, '\\\\' | '%' | '_') { }\n// gone\nlet a = 1;\n"
        out = strip_rust(src)
        self.assertNotIn("gone", out)
        self.assertIn("'%'", out)
        self.assertIn("'_'", out)

    def test_comments_after_an_escaped_char_literal_are_stripped(self):
        src = "let q = '\\n';\n/// doc gone\npub fn f() {}\n"
        self.assertNotIn("doc gone", strip_rust(src))

    def test_license_header_preserved_once(self):
        out = strip_rust(f"{LIC}\n// ordinary\npub fn f() {{}}\n")
        self.assertIn(LIC, out)
        self.assertNotIn("ordinary", out)

    def test_license_not_preserved_when_disabled(self):
        self.assertNotIn(
            "Copyright", strip_rust(f"{LIC}\nfn f() {{}}\n", keep_license=False)
        )


class TestTypeScript(unittest.TestCase):
    def test_url_in_string_survives(self):
        src = 'const u = "https://ipc.localhost";'
        self.assertEqual(strip_ts(src), src)

    def test_template_literal_with_slashes_survives(self):
        src = "const p = `${base}//${name}`;"
        self.assertEqual(strip_ts(src), src)

    def test_template_nested_expression_comment_removed(self):
        out = strip_ts("const p = `a${ b /* gone */ }c`;")
        self.assertNotIn("gone", out)
        self.assertIn("`a${", out)
        self.assertIn("}c`", out)

    def test_regex_literal_survives(self):
        src = "const re = /https:\\/\\/[a-z]+/g;"
        self.assertEqual(strip_ts(src), src)

    def test_regex_with_slash_in_class_survives(self):
        src = "const re = /[/]/;"
        self.assertEqual(strip_ts(src), src)

    def test_division_then_comment(self):
        self.assertEqual(strip_ts("const a = b / c; // note\n"), "const a = b / c; \n")

    def test_line_and_block_comments_removed(self):
        self.assertNotIn("gone", strip_ts("const a = 1; // gone\n/* gone */\n"))

    def test_eslint_directive_preserved(self):
        src = "// eslint-disable-next-line react-hooks/exhaustive-deps\nuseEffect(f, []);\n"
        self.assertIn("eslint-disable-next-line", strip_ts(src))

    def test_ts_expect_error_preserved(self):
        self.assertIn("@ts-expect-error", strip_ts("// @ts-expect-error bad types\nx();\n"))

    def test_license_header_preserved(self):
        out = strip_ts(f"{LIC}\n// ordinary\nexport const a = 1;\n")
        self.assertIn(LIC, out)
        self.assertNotIn("ordinary", out)

    def test_jsx_block_comment_removed(self):
        self.assertNotIn("gone", strip_ts("<div>{/* gone */}</div>"))

    def test_apostrophe_in_string_does_not_swallow_code(self):
        src = "const s = 'it\\'s'; const t = 2;"
        self.assertEqual(strip_ts(src), src)


class TestPython(unittest.TestCase):
    def test_module_and_function_docstrings_removed(self):
        src = '"""Module doc."""\n\n\ndef f():\n    """Fn doc."""\n    return 1\n'
        out = strip_python(src)
        self.assertNotIn("Module doc", out)
        self.assertNotIn("Fn doc", out)
        self.assertIn("return 1", out)

    def test_docstring_only_body_gets_pass(self):
        out = strip_python('def f():\n    """Only doc."""\n')
        self.assertIn("pass", out)
        exec(compile(out, "<t>", "exec"), {})

    def test_comment_removed_code_kept(self):
        out = strip_python("x = 1  # note\ny = 2\n")
        self.assertNotIn("note", out)
        self.assertIn("x = 1", out)
        self.assertIn("y = 2", out)

    def test_hash_inside_string_survives(self):
        src = 'colour = "#ff0000"\n'
        self.assertEqual(strip_python(src), src)

    def test_shebang_preserved(self):
        out = strip_python('#!/usr/bin/env python3\n"""Doc."""\nx = 1\n')
        self.assertTrue(out.startswith("#!/usr/bin/env python3"))
        self.assertNotIn("Doc.", out)

    def test_module_docstring_emptied_when_dunder_doc_is_read(self):
        src = (
            '"""Tool that does X.\n\nDetails.\n"""\n'
            "import argparse\n\n\n"
            'def m():\n    return argparse.ArgumentParser(description=__doc__.split("\\n")[0])\n'
        )
        out = strip_python(src)
        self.assertNotIn("Tool that does X", out)
        self.assertNotIn("Details", out)
        ns: dict = {}
        exec(compile(out, "<t>", "exec"), ns)
        self.assertEqual(ns["__doc__"], "")  # not None -- `.split` must not blow up
        self.assertEqual(ns["m"]().description, "")

    def test_module_docstring_fully_removed_when_dunder_doc_unused(self):
        out = strip_python('"""Prose."""\nx = 1\n')
        self.assertNotIn("Prose", out)
        ns: dict = {}
        exec(compile(out, "<t>", "exec"), ns)
        self.assertIsNone(ns.get("__doc__"))

    def test_class_docstring_removed_keeps_body(self):
        out = strip_python('class C:\n    """Doc."""\n\n    x = 1\n')
        self.assertNotIn("Doc.", out)
        exec(compile(out, "<t>", "exec"), {})

    def test_multiline_docstring_fully_removed(self):
        src = 'def f():\n    """Line one.\n\n    Line two.\n    """\n    return 2\n'
        out = strip_python(src)
        self.assertNotIn("Line one", out)
        self.assertNotIn("Line two", out)
        ns: dict = {}
        exec(compile(out, "<t>", "exec"), ns)
        self.assertEqual(ns["f"](), 2)

    def test_nested_defs_stay_valid(self):
        src = 'def outer():\n    """D."""\n\n    def inner():\n        """D2."""\n\n    return inner\n'
        exec(compile(strip_python(src), "<t>", "exec"), {})


class TestMisc(unittest.TestCase):
    def test_collapse_blank_lines(self):
        self.assertEqual(collapse_blank_lines("a\n\n\n\n\nb\n"), "a\n\nb\n")

    def test_strippers_are_idempotent(self):
        rs = "// x\npub fn f() {}\n"
        ts = "// x\nexport const a = 1;\n"
        py = '"""D."""\nx = 1\n'
        self.assertEqual(strip_rust(strip_rust(rs)), strip_rust(rs))
        self.assertEqual(strip_ts(strip_ts(ts)), strip_ts(ts))
        self.assertEqual(strip_python(strip_python(py)), strip_python(py))


if __name__ == "__main__":
    unittest.main()
