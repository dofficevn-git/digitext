"""Adversarial cases for the config-format strippers."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

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


def clean(fn, src: str) -> str:
    return drop_emptied_lines(src, fn(src))


class TestYaml(unittest.TestCase):
    def test_comment_removed(self):
        self.assertNotIn("gone", clean(strip_yaml, "a: 1 # gone\n# gone\nb: 2\n"))

    def test_hash_inside_quoted_scalar_survives(self):
        src = 'colour: "#ff0000"\ntag: \'v1#2\'\n'
        self.assertEqual(clean(strip_yaml, src), src)

    def test_hash_without_leading_space_is_not_a_comment(self):
        src = "ref: refs/tags#v1\n"
        self.assertEqual(clean(strip_yaml, src), src)

    def test_shell_block_scalar_has_its_comments_stripped(self):
        src = 'jobs:\n  run: |\n    set -e\n    # gone\n    echo "hi"\n'
        out = clean(strip_yaml, src)
        self.assertNotIn("gone", out)
        self.assertIn("set -e", out)
        self.assertIn('echo "hi"', out)

    def test_sequence_item_block_scalar_is_handled(self):
        src = 'command:\n  - |\n    set -e\n    # gone\n    exec x\n'
        out = clean(strip_yaml, src)
        self.assertNotIn("gone", out)
        self.assertIn("exec x", out)

    def test_non_shell_block_scalar_left_verbatim(self):
        src = "notes: |\n  # A markdown heading\n  body text\n"
        self.assertEqual(clean(strip_yaml, src), src)


class TestToml(unittest.TestCase):
    def test_comment_removed(self):
        self.assertNotIn("gone", clean(strip_toml, '# gone\nname = "x" # gone\n'))

    def test_hash_inside_string_survives(self):
        src = 'url = "https://x/y#frag"\n'
        self.assertEqual(clean(strip_toml, src), src)


class TestShell(unittest.TestCase):
    def test_shebang_preserved_comment_removed(self):
        out = clean(strip_sh, "#!/usr/bin/env bash\n# gone\nset -e\n")
        self.assertTrue(out.startswith("#!/usr/bin/env bash"))
        self.assertNotIn("gone", out)

    def test_hash_inside_string_survives(self):
        src = 'echo "colour #ff0000"\n'
        self.assertEqual(clean(strip_sh, src), src)

    def test_parameter_expansion_length_survives(self):
        src = 'n=${#arr[@]}\nm=$#\n'
        self.assertEqual(clean(strip_sh, src), src)

    def test_heredoc_body_preserved(self):
        src = "cat <<'EOF'\n# not a comment, it is data\nEOF\n"
        self.assertEqual(clean(strip_sh, src), src)

    def test_trailing_comment_removed_keeps_command(self):
        out = clean(strip_sh, "set -e  # gone\n")
        self.assertEqual(out, "set -e\n")


class TestSql(unittest.TestCase):
    def test_line_comment_removed(self):
        self.assertNotIn("gone", clean(strip_sql, "-- gone\nSELECT 1; -- gone\n"))

    def test_double_dash_inside_string_survives(self):
        src = "INSERT INTO t VALUES ('a--b');\n"
        self.assertEqual(clean(strip_sql, src), src)

    def test_doubled_quote_escape_survives(self):
        src = "SELECT 'it''s -- fine';\n"
        self.assertEqual(clean(strip_sql, src), src)

    def test_dollar_quoted_body_survives(self):
        src = "CREATE FUNCTION f() RETURNS int AS $$ -- kept\n  SELECT 1;\n$$ LANGUAGE sql;\n"
        self.assertEqual(clean(strip_sql, src), src)

    def test_block_comment_removed(self):
        self.assertNotIn("gone", clean(strip_sql, "/* gone */ SELECT 1;\n"))


class TestCss(unittest.TestCase):
    def test_block_comment_removed(self):
        self.assertNotIn("gone", clean(strip_css, "/* gone */\n.a { color: red; }\n"))

    def test_double_slash_is_not_a_comment_in_css(self):
        src = '.a { background: url("//cdn/x.png"); }\n'
        self.assertEqual(clean(strip_css, src), src)


class TestTerraformAndPbtxt(unittest.TestCase):
    def test_tf_all_three_comment_forms(self):
        out = clean(strip_tf, '# gone\n// gone\n/* gone */\nx = "keep"\n')
        self.assertNotIn("gone", out)
        self.assertIn('x = "keep"', out)

    def test_tf_hash_in_string_survives(self):
        src = 'x = "a#b"\n'
        self.assertEqual(clean(strip_tf, src), src)

    def test_pbtxt_comment_removed(self):
        out = clean(strip_pbtxt, '# gone\nname: "det_dbnet"\n')
        self.assertNotIn("gone", out)
        self.assertIn('name: "det_dbnet"', out)


class TestDropEmptiedLines(unittest.TestCase):
    def test_removes_only_lines_the_strip_blanked(self):
        before = "a\n# c\n\nb\n"
        after = "a\n\n\nb\n"
        self.assertEqual(drop_emptied_lines(before, after), "a\n\nb\n")

    def test_no_op_when_line_count_changed(self):
        self.assertEqual(drop_emptied_lines("a\nb\n", "a\n"), "a\n")


if __name__ == "__main__":
    unittest.main()
