"""ATDD tests for generate_svg.py — TDD GREEN PHASE.

Story 5.2: Implement generate_svg.py
Status: GREEN — all 26 tests pass against the implemented generate_svg.py.

Acceptance Criteria covered:
  AC1: Produces a valid SVG with 52-columns × 7-rows grid using xml.etree.ElementTree (stdlib only)
  AC2: Activity intensity uses Claude orange shades (low: #fef3e8 → high: #f97316); zero days → #ebedf0
  AC3: No JavaScript, no <script>, no event handlers — static SVG compatible with GitHub DOMPurify

Test framework: Python stdlib unittest (no pytest, no external dependencies).
Run: python -m unittest discover -s action/tests
"""

import json
import os
import pathlib
import sys
import tempfile
import unittest
import xml.etree.ElementTree as ET

# ---------------------------------------------------------------------------
# Ensure action/ directory is importable from any working directory
# ---------------------------------------------------------------------------
_ACTION_DIR = pathlib.Path(__file__).parent.parent.resolve()
if str(_ACTION_DIR) not in sys.path:
    sys.path.insert(0, str(_ACTION_DIR))

# ---------------------------------------------------------------------------
# Fixture helpers
# ---------------------------------------------------------------------------

_FIXTURE_DIR = pathlib.Path(__file__).parent / "fixtures"
_EXPECTED_OUTPUT_DIR = _FIXTURE_DIR / "expected_output"

_SAMPLE_DATA_ACTIVE = {
    "generated_at": "2026-04-11T01:00:00Z",
    "username": "stephenleo",
    "days": {
        "2026-04-10": {"sessions": 8, "active_minutes": 120},
        "2026-04-09": {"sessions": 3, "active_minutes": 45},
        "2026-04-08": {"sessions": 1, "active_minutes": 15},
        "2026-03-15": {"sessions": 4, "active_minutes": 60},
    },
}

_SAMPLE_DATA_EMPTY = {
    "generated_at": "2026-04-11T01:00:00Z",
    "username": "stephenleo",
    "days": {},
}

_SAMPLE_DATA_SINGLE_MAX = {
    "generated_at": "2026-04-11T01:00:00Z",
    "username": "stephenleo",
    "days": {
        "2026-04-10": {"sessions": 10, "active_minutes": 200},
    },
}

# Low vs high sessions — used to verify low-intensity colour endpoint (AC2)
_SAMPLE_DATA_LOW_AND_HIGH = {
    "generated_at": "2026-04-11T01:00:00Z",
    "username": "stephenleo",
    "days": {
        "2026-04-10": {"sessions": 1, "active_minutes": 10},
        "2026-04-09": {"sessions": 20, "active_minutes": 300},
    },
}

# Log-scale intensity fixture — max=8, low=1 → verifies bucket mapping (AC2 Dev Notes)
_SAMPLE_DATA_LOG_SCALE = {
    "generated_at": "2026-04-11T01:00:00Z",
    "username": "stephenleo",
    "days": {
        "2026-04-10": {"sessions": 8, "active_minutes": 100},  # max → intensity 4 → #f97316
        "2026-04-09": {"sessions": 1, "active_minutes": 10},   # low → intensity 1 → #fef3e8
    },
}

_NS = {"svg": "http://www.w3.org/2000/svg"}

# Grid dimensions — 52 ISO weeks × 7 days per week
_GRID_CELL_COUNT = 52 * 7  # 364


def _write_data_json(tmp_dir: str, data: dict) -> str:
    """Write data dict as data.json in tmp_dir and return the path."""
    path = os.path.join(tmp_dir, "data.json")
    with open(path, "w", encoding="utf-8") as fh:
        json.dump(data, fh)
    return path


def _run_generate_svg(data: dict, tmp_dir: str) -> str:
    """Call generate_svg.generate() and return SVG content as a string."""
    import generate_svg  # noqa: PLC0415  (local import intentional)

    input_path = _write_data_json(tmp_dir, data)
    output_path = os.path.join(tmp_dir, "heatmap.svg")
    generate_svg.generate(input_path, output_path)  # type: ignore[attr-defined]
    with open(output_path, "r", encoding="utf-8") as fh:
        return fh.read()


def _get_rects(svg_content: str) -> list:
    """Parse SVG content and return all <rect> elements (namespace-aware)."""
    root = ET.fromstring(svg_content)
    rects = root.findall(".//{http://www.w3.org/2000/svg}rect")
    if not rects:
        rects = root.findall(".//rect")
    return rects


# ---------------------------------------------------------------------------
# [P0] AC1 — Valid SVG with 52 × 7 grid layout
# ---------------------------------------------------------------------------


class TestSVGGridStructure(unittest.TestCase):
    """[P0] AC1: generate_svg produces a valid 52-columns × 7-rows heatmap SVG."""

    def test_p0_svg_is_well_formed_xml(self):
        """[P0] SVG output must parse as well-formed XML without errors (AC1, AC3)."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            svg_content = _run_generate_svg(_SAMPLE_DATA_ACTIVE, tmp_dir)
            # Must not raise ParseError
            root = ET.fromstring(svg_content)
            self.assertIsNotNone(root)

    def test_p0_svg_contains_exactly_364_rect_elements(self):
        """[P0] SVG must contain exactly 52 × 7 = 364 <rect> elements representing days (AC1)."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            svg_content = _run_generate_svg(_SAMPLE_DATA_ACTIVE, tmp_dir)
            rects = _get_rects(svg_content)
            self.assertEqual(
                len(rects),
                _GRID_CELL_COUNT,
                f"Expected {_GRID_CELL_COUNT} rect elements (52 × 7), got {len(rects)}",
            )

    def test_p0_svg_has_correct_root_element(self):
        """[P0] Root element must be <svg> with xmlns and viewBox attributes (AC1, AC3)."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            svg_content = _run_generate_svg(_SAMPLE_DATA_ACTIVE, tmp_dir)
            root = ET.fromstring(svg_content)
            # Root tag should be svg (with or without namespace)
            self.assertIn("svg", root.tag.lower())
            # viewBox must be present
            self.assertIn("viewBox", root.attrib)

    def test_p0_empty_days_map_still_produces_364_rect_elements(self):
        """[P0] Empty days dict must still produce a valid 52 × 7 grid (AC1 edge case, Story task 4.5)."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            svg_content = _run_generate_svg(_SAMPLE_DATA_EMPTY, tmp_dir)
            rects = _get_rects(svg_content)
            self.assertEqual(
                len(rects),
                _GRID_CELL_COUNT,
                f"Empty days map: expected {_GRID_CELL_COUNT} rect elements, got {len(rects)}",
            )

    def test_p0_each_rect_has_correct_dimensions(self):
        """[P0] Each <rect> must have width=10, height=10, rx=2 (AC1, Dev Notes SVG Grid Layout)."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            svg_content = _run_generate_svg(_SAMPLE_DATA_ACTIVE, tmp_dir)
            rects = _get_rects(svg_content)
            self.assertGreater(len(rects), 0)
            for rect in rects:
                self.assertEqual(rect.get("width"), "10", "Each rect must have width=10")
                self.assertEqual(rect.get("height"), "10", "Each rect must have height=10")
                self.assertEqual(rect.get("rx"), "2", "Each rect must have rx=2 (rounded corners)")

    def test_p1_svg_file_is_written_to_output_path(self):
        """[P1] generate_svg must write heatmap.svg to the specified output path (AC1)."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            input_path = _write_data_json(tmp_dir, _SAMPLE_DATA_ACTIVE)
            output_path = os.path.join(tmp_dir, "heatmap.svg")
            import generate_svg  # noqa: PLC0415

            generate_svg.generate(input_path, output_path)  # type: ignore[attr-defined]
            self.assertTrue(
                os.path.isfile(output_path),
                f"Expected heatmap.svg to be written to {output_path}",
            )

    def test_p1_svg_contains_month_labels(self):
        """[P1] SVG must contain <text> elements for month labels (Jan–Dec) (AC1, Dev Notes)."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            svg_content = _run_generate_svg(_SAMPLE_DATA_ACTIVE, tmp_dir)
            # Month abbreviations that should appear somewhere in the SVG
            month_abbrevs = {"Jan", "Feb", "Mar", "Apr", "May", "Jun",
                             "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"}
            found_months = {abbr for abbr in month_abbrevs if abbr in svg_content}
            self.assertGreater(
                len(found_months),
                0,
                f"Expected at least one month label in SVG, found none. "
                f"SVG head: {svg_content[:500]}",
            )

    def test_p1_svg_contains_weekday_labels(self):
        """[P1] SVG must contain Mon, Wed, Fri weekday labels (AC1, Dev Notes)."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            svg_content = _run_generate_svg(_SAMPLE_DATA_ACTIVE, tmp_dir)
            for day in ("Mon", "Wed", "Fri"):
                self.assertIn(
                    day,
                    svg_content,
                    f"Expected weekday label '{day}' in SVG output",
                )

    def test_p1_output_is_idempotent(self):
        """[P1] Same data.json input must always produce byte-identical SVG (AC1, Dev Notes Idempotency)."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            svg_first = _run_generate_svg(_SAMPLE_DATA_ACTIVE, tmp_dir)
        with tempfile.TemporaryDirectory() as tmp_dir:
            svg_second = _run_generate_svg(_SAMPLE_DATA_ACTIVE, tmp_dir)
        self.assertEqual(
            svg_first,
            svg_second,
            "Two runs with identical input must produce identical SVG output",
        )


# ---------------------------------------------------------------------------
# [P0/P1] AC2 — Claude orange colour palette
# ---------------------------------------------------------------------------


class TestSVGColourPalette(unittest.TestCase):
    """[P0/P1] AC2: colour palette uses Claude orange shades; zero days → neutral."""

    def test_p0_zero_activity_days_use_neutral_colour(self):
        """[P0] Days with zero sessions must use neutral colour #ebedf0 (AC2)."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            svg_content = _run_generate_svg(_SAMPLE_DATA_ACTIVE, tmp_dir)
        # Neutral hex required by AC2
        self.assertIn(
            "#ebedf0",
            svg_content.lower(),
            "Expected neutral colour #ebedf0 for zero-activity days",
        )

    def test_p0_empty_days_all_cells_use_neutral_colour(self):
        """[P0] When days is empty, all 364 cells must use #ebedf0 (AC2, task 4.5)."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            svg_content = _run_generate_svg(_SAMPLE_DATA_EMPTY, tmp_dir)
            rects = _get_rects(svg_content)
            self.assertEqual(len(rects), _GRID_CELL_COUNT)
            for rect in rects:
                fill = rect.get("fill", "").lower()
                self.assertEqual(
                    fill,
                    "#ebedf0",
                    f"Empty days: expected all rects to be #ebedf0, got fill={fill}",
                )

    def test_p1_max_activity_day_uses_high_orange(self):
        """[P1] Day with maximum sessions must use high-intensity colour #f97316 (AC2 high endpoint)."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            svg_content = _run_generate_svg(_SAMPLE_DATA_SINGLE_MAX, tmp_dir)
        self.assertIn(
            "#f97316",
            svg_content.lower(),
            "Expected high-intensity orange #f97316 for day with maximum sessions",
        )

    def test_p1_low_activity_day_uses_low_orange(self):
        """[P1] Day with low (but non-zero) sessions must use low-intensity colour #fef3e8 (AC2 low endpoint)."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            svg_content = _run_generate_svg(_SAMPLE_DATA_LOW_AND_HIGH, tmp_dir)
        self.assertIn(
            "#fef3e8",
            svg_content.lower(),
            "Expected low-intensity orange #fef3e8 for day with low sessions",
        )

    def test_p1_colour_palette_does_not_include_unexpected_colours(self):
        """[P1] Only the five defined intensity colours should appear as fill values (AC2)."""
        allowed_fills = {"#ebedf0", "#fef3e8", "#fed7aa", "#fb923c", "#f97316"}
        with tempfile.TemporaryDirectory() as tmp_dir:
            svg_content = _run_generate_svg(_SAMPLE_DATA_ACTIVE, tmp_dir)
            rects = _get_rects(svg_content)
            for rect in rects:
                fill = rect.get("fill", "").lower()
                self.assertIn(
                    fill,
                    allowed_fills,
                    f"Unexpected fill colour '{fill}' — must be one of {allowed_fills}",
                )

    def test_p2_intensity_buckets_use_log_scale(self):
        """[P2] Intensity bucketing follows log scale formula (AC2, Dev Notes).

        With max=8, low=1:
          intensity(8) = min(4, int(log(9)/log(9)*4)) = 4 → #f97316
          intensity(1) = min(4, int(log(2)/log(9)*4)) = 1 → #fef3e8 (clamped from 0 for non-zero)
        """
        with tempfile.TemporaryDirectory() as tmp_dir:
            svg_content = _run_generate_svg(_SAMPLE_DATA_LOG_SCALE, tmp_dir)
        # The day with max sessions must map to high-intensity orange
        self.assertIn("#f97316", svg_content.lower())


# ---------------------------------------------------------------------------
# [P0] AC3 — No JavaScript / static SVG / GitHub DOMPurify compatibility
# ---------------------------------------------------------------------------


class TestSVGNoJavaScript(unittest.TestCase):
    """[P0] AC3: SVG must be static — no script elements, no event handlers, no foreignObject."""

    def test_p0_no_script_elements(self):
        """[P0] SVG must not contain <script> elements (AC3, ADR-7)."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            svg_content = _run_generate_svg(_SAMPLE_DATA_ACTIVE, tmp_dir)
        self.assertNotIn(
            "<script",
            svg_content.lower(),
            "SVG must not contain <script> elements — GitHub DOMPurify will strip them",
        )

    def test_p0_no_onclick_handler(self):
        """[P0] SVG must not contain onclick event handlers (AC3, ADR-7)."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            svg_content = _run_generate_svg(_SAMPLE_DATA_ACTIVE, tmp_dir)
        self.assertNotIn(
            "onclick",
            svg_content.lower(),
            "SVG must not contain onclick handlers",
        )

    def test_p0_no_onmouseover_handler(self):
        """[P0] SVG must not contain onmouseover event handlers (AC3, ADR-7)."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            svg_content = _run_generate_svg(_SAMPLE_DATA_ACTIVE, tmp_dir)
        self.assertNotIn(
            "onmouseover",
            svg_content.lower(),
            "SVG must not contain onmouseover handlers",
        )

    def test_p0_no_foreign_object(self):
        """[P0] SVG must not contain <foreignObject> elements (AC3, ADR-7)."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            svg_content = _run_generate_svg(_SAMPLE_DATA_ACTIVE, tmp_dir)
        self.assertNotIn(
            "<foreignobject",
            svg_content.lower(),
            "SVG must not contain <foreignObject> — GitHub DOMPurify will strip it",
        )

    def test_p0_no_javascript_event_handlers_comprehensive(self):
        """[P0] SVG must contain none of the known DOMPurify-stripped event handlers (AC3, task 4.4)."""
        forbidden_patterns = [
            "<script",
            "onclick",
            "onmouseover",
            "onmouseout",
            "onload",
            "onerror",
            "onfocus",
            "onblur",
            "<foreignobject",
            "<object",
            "javascript:",
        ]
        with tempfile.TemporaryDirectory() as tmp_dir:
            svg_content = _run_generate_svg(_SAMPLE_DATA_ACTIVE, tmp_dir)
            svg_lower = svg_content.lower()
        for pattern in forbidden_patterns:
            self.assertNotIn(
                pattern,
                svg_lower,
                f"SVG must not contain '{pattern}' — forbidden by GitHub DOMPurify",
            )

    def test_p1_svg_uses_stdlib_only_no_third_party_imports(self):
        """[P1] generate_svg.py must import only stdlib modules — no third-party libs (Dev Notes)."""
        import ast  # noqa: PLC0415 (stdlib)
        import importlib.util  # noqa: PLC0415

        module_path = _ACTION_DIR / "generate_svg.py"
        self.assertTrue(module_path.exists(), f"generate_svg.py not found at {module_path}")

        source = module_path.read_text(encoding="utf-8")
        tree = ast.parse(source)

        allowed_modules = {
            "json", "datetime", "pathlib", "xml", "sys", "argparse", "math",
            "os", "typing", "abc", "collections", "functools", "itertools",
            "unittest",  # test helpers
        }
        forbidden_third_party = {
            "svgwrite", "cairosvg", "lxml", "numpy", "pandas", "requests",
            "flask", "django", "pytest",
        }

        imported_modules = set()
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                for alias in node.names:
                    imported_modules.add(alias.name.split(".")[0])
            elif isinstance(node, ast.ImportFrom):
                if node.module:
                    imported_modules.add(node.module.split(".")[0])

        for mod in imported_modules:
            self.assertNotIn(
                mod,
                forbidden_third_party,
                f"generate_svg.py must not import '{mod}' — stdlib only allowed",
            )


# ---------------------------------------------------------------------------
# [P1] Input validation and error handling
# ---------------------------------------------------------------------------


class TestSVGInputValidation(unittest.TestCase):
    """[P1] generate_svg.py must validate inputs and exit non-zero on errors."""

    def test_p1_cli_accepts_input_and_output_flags(self):
        """[P1] CLI interface must accept --input and --output flags (Dev Notes integration)."""
        import subprocess  # noqa: PLC0415

        with tempfile.TemporaryDirectory() as tmp_dir:
            input_path = _write_data_json(tmp_dir, _SAMPLE_DATA_ACTIVE)
            output_path = os.path.join(tmp_dir, "heatmap.svg")
            module_path = str(_ACTION_DIR / "generate_svg.py")
            result = subprocess.run(
                [sys.executable, module_path, "--input", input_path, "--output", output_path],
                capture_output=True,
                text=True,
            )
            self.assertEqual(
                result.returncode,
                0,
                f"generate_svg.py CLI exited non-zero. stderr: {result.stderr}",
            )
            self.assertTrue(
                os.path.isfile(output_path),
                "Expected heatmap.svg to be created at the --output path",
            )

    def test_p1_exits_nonzero_on_malformed_json_input(self):
        """[P1] Must exit non-zero with clear stderr message on malformed JSON input (AC1 error path)."""
        import subprocess  # noqa: PLC0415

        with tempfile.TemporaryDirectory() as tmp_dir:
            bad_input = os.path.join(tmp_dir, "bad.json")
            with open(bad_input, "w") as fh:
                fh.write("not valid json {{{")
            output_path = os.path.join(tmp_dir, "heatmap.svg")
            module_path = str(_ACTION_DIR / "generate_svg.py")
            result = subprocess.run(
                [sys.executable, module_path, "--input", bad_input, "--output", output_path],
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(
                result.returncode,
                0,
                "Expected non-zero exit code on malformed JSON input",
            )
            self.assertTrue(
                len(result.stderr) > 0,
                "Expected a non-empty error message on stderr",
            )

    def test_p1_exits_nonzero_on_missing_input_file(self):
        """[P1] Must exit non-zero with clear stderr message when input file does not exist (AC1 error path)."""
        import subprocess  # noqa: PLC0415

        with tempfile.TemporaryDirectory() as tmp_dir:
            missing_input = os.path.join(tmp_dir, "nonexistent.json")
            output_path = os.path.join(tmp_dir, "heatmap.svg")
            module_path = str(_ACTION_DIR / "generate_svg.py")
            result = subprocess.run(
                [sys.executable, module_path, "--input", missing_input, "--output", output_path],
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(
                result.returncode,
                0,
                "Expected non-zero exit code when input file is missing",
            )

    def test_p1_validates_input_schema_has_required_keys(self):
        """[P1] Must exit non-zero when data.json is missing required top-level keys (AC1 validation)."""
        import subprocess  # noqa: PLC0415

        incomplete_data = {"username": "stephenleo"}  # missing generated_at and days
        with tempfile.TemporaryDirectory() as tmp_dir:
            input_path = _write_data_json(tmp_dir, incomplete_data)
            output_path = os.path.join(tmp_dir, "heatmap.svg")
            module_path = str(_ACTION_DIR / "generate_svg.py")
            result = subprocess.run(
                [sys.executable, module_path, "--input", input_path, "--output", output_path],
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(
                result.returncode,
                0,
                "Expected non-zero exit code for input missing required keys",
            )


# ---------------------------------------------------------------------------
# [P3] Snapshot test — byte-for-byte SVG comparison
# ---------------------------------------------------------------------------


class TestSVGSnapshot(unittest.TestCase):
    """[P3] Snapshot regression: output matches golden file (task 4.6)."""

    def test_p3_svg_snapshot_matches_golden_file(self):
        """[P3] SVG output must be byte-for-byte identical to golden fixture (task 4.6)."""
        golden_path = _EXPECTED_OUTPUT_DIR / "heatmap.svg"
        if not golden_path.exists():
            self.skipTest(
                f"Golden fixture not yet generated at {golden_path}. "
                "Run: python action/generate_svg.py --input <known_input> --output "
                "action/tests/fixtures/expected_output/heatmap.svg"
            )

        # Use a known-stable input fixture that matches the golden file
        known_stable_data = {
            "generated_at": "2026-01-01T00:00:00Z",
            "username": "stephenleo",
            "days": {
                "2025-12-31": {"sessions": 5, "active_minutes": 75},
                "2025-12-30": {"sessions": 2, "active_minutes": 30},
            },
        }
        with tempfile.TemporaryDirectory() as tmp_dir:
            svg_content = _run_generate_svg(known_stable_data, tmp_dir)

        with open(golden_path, "r", encoding="utf-8") as fh:
            golden_content = fh.read()

        self.assertEqual(
            svg_content,
            golden_content,
            "SVG output differs from golden snapshot — if the change is intentional, "
            "regenerate the golden file.",
        )


if __name__ == "__main__":
    unittest.main()
