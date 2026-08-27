"""Regression cases for the Python local-variable renamer."""

from __future__ import annotations

import ast
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from rename_py import (  # noqa: E402
    Counter,
    _collect_scope,
    apply_edits,
    plan_file,
    plan_items,
    undo,
    undo_items,
)


def run(src: str) -> str:
    tree = ast.parse(src)
    edits, inverse = plan_file(tree, Counter())
    out = apply_edits(src, edits)
    # Same guard the driver applies: a pure rename and nothing else.
    assert undo(ast.parse(out), inverse) == ast.dump(ast.parse(src))
    return out


class TestRenamePy(unittest.TestCase):
    def test_renames_a_plain_local(self):
        out = run("def f():\n    total = 1\n    return total\n")
        self.assertNotIn("total", out)
        self.assertIn("p_0", out)

    def test_non_ascii_lines_are_not_corrupted(self):
        # ast reports col_offset in UTF-8 BYTES. Slicing the str by that index
        # shreds any line with Vietnamese text -- most of this codebase.
        src = 'def f():\n    tong = 1\n    ghi_chu = "Hồ sơ đã ký số — xong"\n    return tong, ghi_chu\n'
        out = run(src)
        self.assertIn('"Hồ sơ đã ký số — xong"', out)
        ast.parse(out)

    def test_non_ascii_identifier_reference_survives(self):
        src = 'def f():\n    x = "→ ✓ ≥"\n    y = 2\n    return x, y\n'
        out = run(src)
        self.assertIn('"→ ✓ ≥"', out)

    def test_parameters_are_never_renamed(self):
        # Callers may pass them by keyword; nothing in the AST proves otherwise.
        out = run("def f(count):\n    n = count + 1\n    return n\n")
        self.assertIn("def f(count):", out)
        self.assertIn("count + 1", out)

    def test_module_level_names_untouched(self):
        out = run("TOTAL = 1\n\n\ndef f():\n    return TOTAL\n")
        self.assertIn("TOTAL = 1", out)
        self.assertIn("return TOTAL", out)

    def test_functions_using_dynamic_lookup_are_skipped(self):
        out = run("def f():\n    total = 1\n    return locals()\n")
        self.assertIn("total = 1", out)

    def test_names_quoted_anywhere_in_the_file_are_skipped(self):
        # Covers getattr(o, "total") and other string-keyed access.
        out = run('KEY = "total"\n\n\ndef f():\n    total = 1\n    return total\n')
        self.assertIn("total = 1", out)

    def test_fstring_references_are_renamed_together(self):
        out = run('def f():\n    tong = 1\n    return f"{tong}"\n')
        self.assertNotIn("tong", out)
        ast.parse(out)

    def test_nested_function_closure_reference_renamed(self):
        src = "def f():\n    acc = 1\n\n    def g():\n        return acc\n\n    return g\n"
        out = run(src)
        self.assertNotIn("acc", out)
        ns: dict = {}
        exec(compile(out, "<t>", "exec"), ns)
        self.assertEqual(ns["f"]()(), 1)

    def test_comprehension_target_renamed_consistently(self):
        out = run("def f(xs):\n    doubled = [n * 2 for n in xs]\n    return doubled\n")
        ns: dict = {}
        exec(compile(out, "<t>", "exec"), ns)
        self.assertEqual(ns["f"]([1, 2]), [2, 4])

    def test_generated_name_never_collides_with_an_existing_one(self):
        out = run("def f():\n    p_0 = 1\n    other = 2\n    return p_0 + other\n")
        ns: dict = {}
        exec(compile(out, "<t>", "exec"), ns)
        self.assertEqual(ns["f"](), 3)

    def test_behaviour_is_preserved(self):
        src = (
            "def f(items):\n"
            "    seen = set()\n"
            "    kept = []\n"
            "    for it in items:\n"
            "        if it not in seen:\n"
            "            seen.add(it)\n"
            "            kept.append(it)\n"
            "    return kept\n"
        )
        out = run(src)
        ns: dict = {}
        exec(compile(out, "<t>", "exec"), ns)
        self.assertEqual(ns["f"]([1, 2, 1, 3]), [1, 2, 3])

    def test_is_idempotent(self):
        once = run("def f():\n    total = 1\n    return total\n")
        self.assertEqual(run(once), once)


if __name__ == "__main__":
    unittest.main()


class TestRenameItems(unittest.TestCase):
    """Module-level def/class renaming across files."""

    def plan(self, sources: dict[str, str]):
        trees = {Path(k): ast.parse(v) for k, v in sources.items()}
        eligible, _ = _collect_scope(trees, {"a", "b"})
        mapping = {n: f"q_{i:x}" for i, n in enumerate(sorted(eligible))}
        inverse = {v: k for k, v in mapping.items()}
        out = {}
        for k, v in sources.items():
            edits = plan_items(trees[Path(k)], v, mapping)
            text = apply_edits(v, edits)
            assert undo_items(ast.parse(text), inverse) == ast.dump(ast.parse(v))
            out[k] = text
        return out

    def test_renames_a_unique_module_function(self):
        out = self.plan({"a.py": "def only_helper():\n    return 1\n\n\nonly_helper()\n"})
        self.assertNotIn("only_helper", out["a.py"])
        self.assertIn("def q_0", out["a.py"])

    def test_renames_consistently_across_files(self):
        out = self.plan(
            {
                "a.py": "def shared_fn():\n    return 1\n",
                "b.py": "from a import shared_fn\n\nshared_fn()\n",
            }
        )
        self.assertNotIn("shared_fn", out["a.py"])
        self.assertNotIn("shared_fn", out["b.py"])
        self.assertIn("from a import q_0", out["b.py"])

    def test_leaves_attribute_reachable_names_alone(self):
        # `mod.helper()` is resolved at run time; renaming the def breaks it.
        out = self.plan({"a.py": "def helper():\n    return 1\n", "b.py": "import a\n\na.helper()\n"})
        self.assertIn("def helper", out["a.py"])

    def test_leaves_duplicated_names_alone(self):
        out = self.plan({"a.py": "def parse():\n    pass\n", "b.py": "def parse():\n    pass\n"})
        self.assertIn("def parse", out["a.py"])
        self.assertIn("def parse", out["b.py"])

    def test_leaves_quoted_names_alone(self):
        out = self.plan({"a.py": 'def handler():\n    pass\n\n\nNAME = "handler"\n'})
        self.assertIn("def handler", out["a.py"])

    def test_non_ascii_definition_line_survives(self):
        src = 'def bao_cao():\n    return "Báo cáo — hồ sơ đã ký"\n\n\nbao_cao()\n'
        out = self.plan({"a.py": src})
        self.assertIn('"Báo cáo — hồ sơ đã ký"', out["a.py"])
        ast.parse(out["a.py"])

    def test_class_names_renamed_and_still_run(self):
        out = self.plan({"a.py": "class WidgetBox:\n    x = 1\n\n\nW = WidgetBox()\n"})
        self.assertNotIn("WidgetBox", out["a.py"])
        ns: dict = {}
        exec(compile(out["a.py"], "<t>", "exec"), ns)
        self.assertEqual(ns["W"].x, 1)
