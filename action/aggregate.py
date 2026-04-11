"""aggregate.py — Aggregates per-machine Hive partition files into daily totals.

Implementation: Epic 5, Story 5.1.
"""

import collections
import datetime
import json
import os
import pathlib
import sys


def load_purged_machines(root: pathlib.Path) -> set:
    """Return set of machine_id strings whose status is 'purged' in registry.json.
    Returns empty set if registry.json is absent or malformed."""
    registry_path = root / "registry.json"
    if not registry_path.exists():
        return set()
    try:
        with open(registry_path, encoding="utf-8") as f:
            registry = json.load(f)
    except (OSError, json.JSONDecodeError):
        # Absent, unreadable, or malformed registry → treat as empty purged set.
        return set()
    purged = set()
    for machine in registry.get("machines", []) or []:
        if not isinstance(machine, dict):
            continue
        if machine.get("status") == "purged":
            machine_id = machine.get("machine_id")
            if isinstance(machine_id, str):
                purged.add(machine_id)
    return purged


def parse_date_from_path(path: pathlib.Path):
    """Extract YYYY-MM-DD from a Hive partition path. Returns str date or None if path is malformed."""
    # Path format:
    # machines/year=YYYY/month=MM/day=DD/harness=.../machine_id=.../data.json
    parts = path.parts
    year_val = None
    month_val = None
    day_val = None
    for part in parts:
        if part.startswith("year="):
            year_val = part[len("year="):]
        elif part.startswith("month="):
            month_val = part[len("month="):]
        elif part.startswith("day="):
            day_val = part[len("day="):]
    if year_val and month_val and day_val:
        return f"{year_val}-{month_val}-{day_val}"
    return None


def _extract_machine_id_from_path(path: pathlib.Path):
    """Extract machine_id value from a Hive partition path. Returns str or None."""
    for part in path.parts:
        if part.startswith("machine_id="):
            return part[len("machine_id="):]
    return None


def aggregate(root: pathlib.Path, username: str) -> dict:
    """Aggregate all Hive partition files under root/machines/. Returns public data.json dict."""
    purged_machines = load_purged_machines(root)

    days = collections.defaultdict(lambda: {"sessions": 0, "active_minutes": 0})

    machines_dir = root / "machines"
    if machines_dir.exists():
        for data_file in machines_dir.glob("year=*/month=*/day=*/harness=*/machine_id=*/data.json"):
            machine_id = _extract_machine_id_from_path(data_file)
            if machine_id in purged_machines:
                continue

            date_key = parse_date_from_path(data_file)
            if date_key is None:
                raise ValueError(f"Cannot parse date from path: {data_file}")

            with open(data_file, encoding="utf-8") as f:
                record = json.load(f)

            days[date_key]["sessions"] += record["sessions"]
            days[date_key]["active_minutes"] += record["active_minutes"]

    generated_at = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

    return {
        "generated_at": generated_at,
        "username": username,
        "days": dict(days),
    }


def main():
    root = pathlib.Path(".")
    username = os.environ.get("GITHUB_REPOSITORY_OWNER") or \
               os.environ.get("GITHUB_REPOSITORY", "/").split("/")[0]
    result = aggregate(root, username)
    with open("data.json", "w", encoding="utf-8") as f:
        json.dump(result, f, indent=2)
    print(f"aggregate.py: wrote data.json with {len(result['days'])} day(s)")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        print(f"aggregate.py: fatal error: {e}", file=sys.stderr)
        sys.exit(1)
