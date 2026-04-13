"""generate_svg.py — Generates an SVG heatmap from aggregated session activity data.

Implementation: Epic 5, Story 5.2.

CLI usage:
    python generate_svg.py --input vibestats/data.json --output vibestats/heatmap.svg

The script reads data.json conforming to:
    {
        "generated_at": "YYYY-MM-DDTHH:MM:SSZ",
        "username": "str",
        "days": {
            "YYYY-MM-DD": {"sessions": int, "active_minutes": int},
            ...
        }
    }

And writes a static SVG heatmap (GitHub-contributions-style, Claude orange palette)
to the specified output path. No JavaScript, no third-party dependencies — stdlib only.
"""

from __future__ import annotations

import argparse
import io
import json
import math
import pathlib
import sys
import xml.etree.ElementTree as ET
from datetime import date, timedelta


# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

# Claude orange colour palettes — intensities 0–4
# Intensity 0 and 4 are fixed by AC2; 1–3 are implementation-defined orange family.
COLOUR_PALETTES = {
    "light": {
        0: "#ebedf0",  # neutral/zero
        1: "#fef3e8",  # low
        2: "#fed7aa",  # orange-200
        3: "#fb923c",  # orange-400
        4: "#f97316",  # high
    },
    "dark": {
        0: "#2a3346",  # neutral/zero — visible against dark bg
        1: "#fef3e8",  # low
        2: "#fed7aa",  # orange-200
        3: "#fb923c",  # orange-400
        4: "#f97316",  # high
    },
}

# Theme-aware text/label colours
TEXT_COLOURS = {
    "light": "#586069",
    "dark": "#8b949e",
}

# SVG layout constants
CELL_SIZE = 10       # px
CELL_GAP = 2         # px
CELL_STRIDE = CELL_SIZE + CELL_GAP  # 12 px
NUM_COLS = 52        # ISO weeks
NUM_ROWS = 7         # days per week (0=Sun … 6=Sat)

LEFT_MARGIN = 30     # space for weekday labels
TOP_MARGIN = 20      # space for month labels
RIGHT_MARGIN = 10
BOTTOM_MARGIN = 10

SVG_WIDTH = NUM_COLS * CELL_STRIDE + LEFT_MARGIN + RIGHT_MARGIN   # 52*12+30+10 = 664
SVG_HEIGHT = NUM_ROWS * CELL_STRIDE + TOP_MARGIN + BOTTOM_MARGIN  # 7*12+20+10 = 114

# Weekday labels — only Mon, Wed, Fri shown (matching GitHub)
# Row index: 0=Sun, 1=Mon, 2=Tue, 3=Wed, 4=Thu, 5=Fri, 6=Sat
WEEKDAY_LABELS = {1: "Mon", 3: "Wed", 5: "Fri"}

MONTH_ABBREVS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun",
                 "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]


# ---------------------------------------------------------------------------
# Grid computation
# ---------------------------------------------------------------------------

def _compute_grid_dates(last_date: date) -> list[list[date]]:
    """Return a 52-column × 7-row grid of dates.

    Column 0 is the oldest week; column 51 is the week containing last_date.
    Row 0 = Sunday, Row 6 = Saturday (matching GitHub's day ordering).

    Every cell is populated with a real date — the 52 × 7 window spans exactly
    364 consecutive days (52 weeks) ending on the Saturday of the week that
    contains last_date.
    """
    # Python weekday(): Mon=0 … Sun=6.  Convert to GitHub row index (Sun=0 … Sat=6)
    # so the Sunday that starts last_date's week sits at row 0 of that column.
    github_row_of_last = (last_date.weekday() + 1) % 7  # Python Sun=6 → GitHub 0

    # Sunday that starts the week containing last_date (column 51, row 0).
    sunday_of_last_week = last_date - timedelta(days=github_row_of_last)

    # Sunday of week 0 (first column) is 51 weeks before sunday_of_last_week.
    sunday_of_first_week = sunday_of_last_week - timedelta(weeks=NUM_COLS - 1)

    grid: list[list[date]] = []
    for col in range(NUM_COLS):
        week_sunday = sunday_of_first_week + timedelta(weeks=col)
        column = [week_sunday + timedelta(days=row) for row in range(NUM_ROWS)]
        grid.append(column)

    return grid


def _compute_intensity(sessions: int, max_sessions: int) -> int:
    """Return intensity bucket 0–4 using log scale.

    intensity = min(4, int(log(1 + sessions) / log(1 + max_sessions) * 4))
    where max_sessions is the max across all days with activity (>0).
    If max_sessions == 0 or sessions <= 0, returns 0. Any non-zero sessions
    always yields >= 1 to ensure visual distinction from zero-activity days.

    Negative session counts are treated as zero activity rather than raising
    (the aggregator should never emit them, but belt-and-braces keeps this
    helper pure and crash-free on malformed upstream data).
    """
    if max_sessions <= 0 or sessions <= 0:
        return 0
    raw = math.log(1 + sessions) / math.log(1 + max_sessions) * 4
    # Clamp to [1, 4] for non-zero activity days so they are always visually
    # distinct from zero-activity days (which are intensity 0 / #ebedf0).
    return max(1, min(4, int(raw)))


# ---------------------------------------------------------------------------
# SVG rendering
# ---------------------------------------------------------------------------

def _build_svg(grid: list[list[date]], days: dict[str, dict], theme: str = "light") -> ET.Element:
    """Build and return the SVG ElementTree root element.

    grid: list of 52 columns, each a list of 7 dates.
    days: mapping from "YYYY-MM-DD" → {"sessions": int, "active_minutes": int}
    theme: "light" or "dark" — controls palette and label colours.
    """
    # Compute max_sessions across all activity days (validated to be ints
    # upstream in generate(), so .get() default of 0 is purely defensive).
    max_sessions = max(
        (v.get("sessions", 0) for v in days.values() if v.get("sessions", 0) > 0),
        default=0,
    )

    # Use plain tag names (no namespace prefix) so ElementTree doesn't add
    # "ns0:" prefixes or duplicate xmlns declarations in the output.
    # We'll inject the xmlns attribute on the root element directly.
    svg = ET.Element("svg")
    svg.set("xmlns", "http://www.w3.org/2000/svg")
    svg.set("width", str(SVG_WIDTH))
    svg.set("height", str(SVG_HEIGHT))
    # viewBox must be camelCase for valid SVG — ElementTree preserves attribute
    # names exactly as set.
    svg.set("viewBox", f"0 0 {SVG_WIDTH} {SVG_HEIGHT}")

    # --- Weekday labels (Mon, Wed, Fri) ---
    for row_idx, label in WEEKDAY_LABELS.items():
        # SVG <text> y anchors at the font baseline.  Offset by CELL_SIZE-1
        # so the label sits vertically centred against its matching row of
        # rects rather than hovering above them.
        y = TOP_MARGIN + row_idx * CELL_STRIDE + CELL_SIZE - 1
        text = ET.SubElement(svg, "text")
        text.set("x", str(LEFT_MARGIN - 4))
        text.set("y", str(y))
        text.set("font-size", "9")
        text.set("text-anchor", "end")
        text.set("fill", TEXT_COLOURS[theme])
        text.set("font-family", "sans-serif")
        text.text = label

    # --- Month labels (abbreviated) ---
    # Determine which column starts a new month and label it once.
    # The grid is dense (every column has 7 real dates), so we use the
    # first row's date as the column's representative.
    prev_month = None
    for col, column in enumerate(grid):
        col_date = column[0]
        month = col_date.month
        if month != prev_month:
            x = LEFT_MARGIN + col * CELL_STRIDE
            text = ET.SubElement(svg, "text")
            text.set("x", str(x))
            text.set("y", str(TOP_MARGIN - 6))
            text.set("font-size", "9")
            text.set("fill", TEXT_COLOURS[theme])
            text.set("font-family", "sans-serif")
            text.text = MONTH_ABBREVS[month - 1]
            prev_month = month

    # --- Day cells (rect elements) ---
    for col, column in enumerate(grid):
        for row, cell_date in enumerate(column):
            x = LEFT_MARGIN + col * CELL_STRIDE
            y = TOP_MARGIN + row * CELL_STRIDE

            date_str = cell_date.strftime("%Y-%m-%d")
            day_data = days.get(date_str, {})
            sessions = day_data.get("sessions", 0)
            intensity = _compute_intensity(sessions, max_sessions)
            fill = COLOUR_PALETTES[theme][intensity]

            rect = ET.SubElement(svg, "rect")
            rect.set("x", str(x))
            rect.set("y", str(y))
            rect.set("width", str(CELL_SIZE))
            rect.set("height", str(CELL_SIZE))
            rect.set("rx", "2")
            rect.set("fill", fill)

    return svg


def _svg_to_string(svg_root: ET.Element) -> str:
    """Serialise SVG element to a Unicode string with XML declaration."""
    tree = ET.ElementTree(svg_root)
    buf = io.StringIO()
    tree.write(buf, encoding="unicode", xml_declaration=True)
    return buf.getvalue()


# ---------------------------------------------------------------------------
# Main public API
# ---------------------------------------------------------------------------

def generate(input_path: str, output_path: str, theme: str = "light") -> None:
    """Read data.json at input_path, write heatmap.svg to output_path.

    theme: "light" or "dark" — controls palette and label colours.
    Raises SystemExit with non-zero code on any error.
    """
    # Load and parse input
    try:
        with open(input_path, "r", encoding="utf-8") as fh:
            data = json.load(fh)
    except FileNotFoundError:
        print(f"ERROR: Input file not found: {input_path}", file=sys.stderr)
        sys.exit(1)
    except json.JSONDecodeError as exc:
        print(f"ERROR: Malformed JSON in {input_path}: {exc}", file=sys.stderr)
        sys.exit(1)
    except OSError as exc:
        print(f"ERROR: Cannot read {input_path}: {exc}", file=sys.stderr)
        sys.exit(1)

    # Validate schema
    required_keys = {"generated_at", "username", "days"}
    missing = required_keys - set(data.keys())
    if missing:
        print(
            f"ERROR: data.json is missing required keys: {sorted(missing)}",
            file=sys.stderr,
        )
        sys.exit(1)

    if not isinstance(data["days"], dict):
        print("ERROR: data.json 'days' field must be a JSON object", file=sys.stderr)
        sys.exit(1)

    days: dict[str, dict] = data["days"]

    # Validate each day entry — sessions must be a non-negative int.  Booleans
    # are an int subclass in Python, so we reject them explicitly.
    for date_str, entry in days.items():
        if not isinstance(entry, dict):
            print(
                f"ERROR: data.json days['{date_str}'] must be an object, got "
                f"{type(entry).__name__}",
                file=sys.stderr,
            )
            sys.exit(1)
        sessions = entry.get("sessions", 0)
        if isinstance(sessions, bool) or not isinstance(sessions, int) or sessions < 0:
            print(
                f"ERROR: data.json days['{date_str}'].sessions must be a "
                f"non-negative integer, got {sessions!r}",
                file=sys.stderr,
            )
            sys.exit(1)

    # Determine last_date: use the latest date in the dataset or fall back to
    # generated_at for empty datasets.  We NEVER use date.today() here — that
    # would violate the idempotency contract (same input → same output).
    if days:
        try:
            last_date = max(date.fromisoformat(d) for d in days.keys())
        except ValueError as exc:
            print(f"ERROR: Invalid date key in days: {exc}", file=sys.stderr)
            sys.exit(1)
    else:
        # Empty days: parse date from generated_at.  Required for idempotency.
        generated_at = data.get("generated_at")
        if not isinstance(generated_at, str) or len(generated_at) < 10:
            print(
                "ERROR: data.json 'generated_at' must be an ISO-8601 string "
                "(e.g., '2026-04-11T01:00:00Z') when 'days' is empty",
                file=sys.stderr,
            )
            sys.exit(1)
        try:
            last_date = date.fromisoformat(generated_at[:10])
        except ValueError as exc:
            print(
                f"ERROR: data.json 'generated_at' is not a valid ISO-8601 "
                f"date: {exc}",
                file=sys.stderr,
            )
            sys.exit(1)

    # Build grid and SVG
    grid = _compute_grid_dates(last_date)
    svg_root = _build_svg(grid, days, theme=theme)
    svg_content = _svg_to_string(svg_root)

    # Write output
    try:
        out_path = pathlib.Path(output_path)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(svg_content, encoding="utf-8")
    except OSError as exc:
        print(f"ERROR: Cannot write SVG to {output_path}: {exc}", file=sys.stderr)
        sys.exit(1)


# ---------------------------------------------------------------------------
# CLI entry point
# ---------------------------------------------------------------------------

def _parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate a GitHub-contributions-style SVG heatmap from vibestats data.json."
    )
    parser.add_argument(
        "--input",
        default="vibestats/data.json",
        help="Path to data.json (default: vibestats/data.json)",
    )
    parser.add_argument(
        "--output",
        default="vibestats/heatmap.svg",
        help="Path to write heatmap.svg (default: vibestats/heatmap.svg)",
    )
    parser.add_argument(
        "--theme",
        choices=["light", "dark"],
        default="light",
        help="Colour theme: 'light' or 'dark' (default: light)",
    )
    return parser.parse_args(argv)


if __name__ == "__main__":
    args = _parse_args()
    generate(args.input, args.output, theme=args.theme)
