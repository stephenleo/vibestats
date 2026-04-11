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
import json
import math
import pathlib
import sys
import xml.etree.ElementTree as ET
from datetime import date, timedelta
from typing import Optional


# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

# Claude orange colour palette — intensities 0–4
# Intensity 0 and 4 are fixed by AC2; 1–3 are implementation-defined orange family.
COLOUR_PALETTE = {
    0: "#ebedf0",  # neutral/zero — required by AC2
    1: "#fef3e8",  # low — required by AC2
    2: "#fed7aa",  # orange-200
    3: "#fb923c",  # orange-400
    4: "#f97316",  # high — required by AC2
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

def _compute_grid_dates(last_date: date) -> list[list[date | None]]:
    """Return a 52-column × 7-row grid of dates.

    Column 0 is the oldest week; column 51 is the week containing last_date.
    Row 0 = Sunday, Row 6 = Saturday (matching GitHub's day ordering).

    Cells before the first valid date in column 0 are None (padding).
    """
    # Find the Sunday that starts the current (last) week
    # last_date.isoweekday(): 1=Mon … 7=Sun
    day_of_week = last_date.weekday()  # 0=Mon … 6=Sun
    # Convert to GitHub day index (0=Sun … 6=Sat)
    # Python weekday: Mon=0…Sat=5, Sun=6
    # GitHub row:     Sun=0…Sat=6
    github_row_of_last = (day_of_week + 1) % 7  # Python Sun=6 → GitHub 0

    # The last cell in the grid is (col=51, row=github_row_of_last)
    # The Sunday of week 51 is:
    sunday_of_last_week = last_date - timedelta(days=github_row_of_last)

    # Sunday of week 0 (first column) is 51 weeks before sunday_of_last_week
    sunday_of_first_week = sunday_of_last_week - timedelta(weeks=NUM_COLS - 1)

    grid: list[list[date | None]] = []
    for col in range(NUM_COLS):
        column: list[date | None] = []
        week_sunday = sunday_of_first_week + timedelta(weeks=col)
        for row in range(NUM_ROWS):
            d = week_sunday + timedelta(days=row)
            column.append(d)
        grid.append(column)

    return grid


def _compute_intensity(sessions: int, max_sessions: int) -> int:
    """Return intensity bucket 0–4 using log scale.

    intensity = min(4, int(log(1 + sessions) / log(1 + max_sessions) * 4))
    where max_sessions is the max across all days with activity (>0).
    If max_sessions == 0, returns 0. Any non-zero sessions always yields >= 1
    to ensure visual distinction from zero-activity days.
    """
    if max_sessions == 0 or sessions <= 0:
        return 0
    raw = math.log(1 + sessions) / math.log(1 + max_sessions) * 4
    # Clamp to [1, 4] for non-zero activity days so they are always visually
    # distinct from zero-activity days (which are intensity 0 / #ebedf0).
    return max(1, min(4, int(raw)))


# ---------------------------------------------------------------------------
# SVG rendering
# ---------------------------------------------------------------------------

def _build_svg(grid: list[list[date | None]], days: dict[str, dict]) -> ET.Element:
    """Build and return the SVG ElementTree root element.

    grid: list of 52 columns, each a list of 7 date-or-None values.
    days: mapping from "YYYY-MM-DD" → {"sessions": int, "active_minutes": int}
    """
    # Compute max_sessions across all activity days
    max_sessions = max(
        (v["sessions"] for v in days.values() if v.get("sessions", 0) > 0),
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
        y = TOP_MARGIN + row_idx * CELL_STRIDE + CELL_SIZE - 1  # baseline align
        text = ET.SubElement(svg, "text")
        text.set("x", str(LEFT_MARGIN - 4))
        text.set("y", str(y))
        text.set("font-size", "9")
        text.set("text-anchor", "end")
        text.set("fill", "#586069")
        text.set("font-family", "sans-serif")
        text.text = label

    # --- Month labels (abbreviated) ---
    # Determine which column starts a new month and label it once
    prev_month = None
    for col, column in enumerate(grid):
        # Use the first non-None date in the column to determine its month
        col_date = next((d for d in column if d is not None), None)
        if col_date is None:
            continue
        month = col_date.month
        if month != prev_month:
            x = LEFT_MARGIN + col * CELL_STRIDE
            text = ET.SubElement(svg, "text")
            text.set("x", str(x))
            text.set("y", str(TOP_MARGIN - 6))
            text.set("font-size", "9")
            text.set("fill", "#586069")
            text.set("font-family", "sans-serif")
            text.text = MONTH_ABBREVS[month - 1]
            prev_month = month

    # --- Day cells (rect elements) ---
    for col, column in enumerate(grid):
        for row, cell_date in enumerate(column):
            x = LEFT_MARGIN + col * CELL_STRIDE
            y = TOP_MARGIN + row * CELL_STRIDE

            if cell_date is None:
                fill = COLOUR_PALETTE[0]
            else:
                date_str = cell_date.strftime("%Y-%m-%d")
                day_data = days.get(date_str, {})
                sessions = day_data.get("sessions", 0)
                intensity = _compute_intensity(sessions, max_sessions)
                fill = COLOUR_PALETTE[intensity]

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
    import io
    tree = ET.ElementTree(svg_root)
    buf = io.StringIO()
    tree.write(buf, encoding="unicode", xml_declaration=True)
    return buf.getvalue()


# ---------------------------------------------------------------------------
# Main public API
# ---------------------------------------------------------------------------

def generate(input_path: str, output_path: str) -> None:
    """Read data.json at input_path, write heatmap.svg to output_path.

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

    # Determine last_date: use the latest date in the dataset or today if empty
    if days:
        try:
            last_date = max(date.fromisoformat(d) for d in days.keys())
        except ValueError as exc:
            print(f"ERROR: Invalid date key in days: {exc}", file=sys.stderr)
            sys.exit(1)
    else:
        # Use a fixed reference date for idempotency: parse from generated_at if available
        try:
            # generated_at: "2026-04-11T01:00:00Z" → take date part
            last_date = date.fromisoformat(data["generated_at"][:10])
        except (ValueError, KeyError):
            last_date = date.today()

    # Build grid and SVG
    grid = _compute_grid_dates(last_date)
    svg_root = _build_svg(grid, days)
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
    return parser.parse_args(argv)


if __name__ == "__main__":
    args = _parse_args()
    generate(args.input, args.output)
