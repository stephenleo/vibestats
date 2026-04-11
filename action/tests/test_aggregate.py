"""test_aggregate.py — Unit tests for aggregate.py.

Story 5.1: Implement aggregate.py
GH Issue: #26

Test IDs follow: 5.1-UNIT-{SEQ}
"""

import json
import os
import pathlib
import subprocess
import sys
import tempfile
import unittest

# ---------------------------------------------------------------------------
# Fixture helpers
# ---------------------------------------------------------------------------

FIXTURES_ROOT = pathlib.Path(__file__).parent / "fixtures" / "sample_machine_data"

# Expected days values derived from fixtures (purged machine excluded):
#   2026-04-09: machine-a/claude(3s,45m) + machine-b/claude(2s,30m) + machine-a/codex(1s,10m)
#              → sessions=6, active_minutes=85
#   2026-04-10: machine-a/claude(4s,60m) + machine-b/claude(1s,15m)
#              → sessions=5, active_minutes=75
#   2026-04-11: machine-a/claude(5s,75m) → sessions=5, active_minutes=75
EXPECTED_DAYS = {
    "2026-04-09": {"sessions": 6, "active_minutes": 85},
    "2026-04-10": {"sessions": 5, "active_minutes": 75},
    "2026-04-11": {"sessions": 5, "active_minutes": 75},
}


def _import_aggregate():
    """Dynamically import aggregate module from action/aggregate.py."""
    import importlib.util

    aggregate_path = pathlib.Path(__file__).parent.parent / "aggregate.py"
    spec = importlib.util.spec_from_file_location("aggregate", aggregate_path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


# ---------------------------------------------------------------------------
# P0 Tests (Critical)
# ---------------------------------------------------------------------------


class TestAggregateSumMultipleMachines(unittest.TestCase):
    """5.1-UNIT-001: Two active machines on same date → values summed (P0, R-001)."""

    def test_two_machines_same_date_sessions_summed(self):
        """Given two active machines on 2026-04-10, when aggregated,
        their sessions must be summed (4+1=5)."""
        agg = _import_aggregate()
        result = agg.aggregate(FIXTURES_ROOT, "testuser")

        self.assertIn("2026-04-10", result["days"])
        self.assertEqual(result["days"]["2026-04-10"]["sessions"], 5)

    def test_two_machines_same_date_active_minutes_summed(self):
        """Given two active machines on 2026-04-10, when aggregated,
        their active_minutes must be summed (60+15=75)."""
        agg = _import_aggregate()
        result = agg.aggregate(FIXTURES_ROOT, "testuser")

        self.assertIn("2026-04-10", result["days"])
        self.assertEqual(result["days"]["2026-04-10"]["active_minutes"], 75)


class TestAggregatePurgedMachineSkipped(unittest.TestCase):
    """5.1-UNIT-002: Purged machine data absent from output (P0, R-001)."""

    def test_purged_machine_sessions_excluded(self):
        """Given machine-purged has status=purged in registry.json,
        its 99 sessions on 2026-04-09 must NOT appear in the output.
        Expected: sessions=6 (not 105=6+99)."""
        agg = _import_aggregate()
        result = agg.aggregate(FIXTURES_ROOT, "testuser")

        self.assertIn("2026-04-09", result["days"])
        # Purged machine had 99 sessions — if included, total would be 105
        self.assertEqual(result["days"]["2026-04-09"]["sessions"], 6)
        self.assertNotEqual(result["days"]["2026-04-09"]["sessions"], 105)

    def test_purged_machine_active_minutes_excluded(self):
        """Given machine-purged has status=purged in registry.json,
        its 999 active_minutes on 2026-04-09 must NOT appear in the output.
        Expected: active_minutes=85 (not 1084=85+999)."""
        agg = _import_aggregate()
        result = agg.aggregate(FIXTURES_ROOT, "testuser")

        self.assertIn("2026-04-09", result["days"])
        # Purged machine had 999 active_minutes — if included, total would be 1084
        self.assertEqual(result["days"]["2026-04-09"]["active_minutes"], 85)
        self.assertNotEqual(result["days"]["2026-04-09"]["active_minutes"], 1084)


class TestAggregateOutputSchema(unittest.TestCase):
    """5.1-UNIT-003: Output schema conforms to public spec — no machine IDs/paths/hostnames (P0, R-002)."""

    def test_output_has_exactly_three_top_level_keys(self):
        """data.json must have exactly: generated_at, username, days. No extras."""
        agg = _import_aggregate()
        result = agg.aggregate(FIXTURES_ROOT, "testuser")

        self.assertEqual(set(result.keys()), {"generated_at", "username", "days"})

    def test_days_values_have_only_numeric_fields(self):
        """Each days entry must have only sessions (int) and active_minutes (int).
        No machine IDs, Hive paths, hostnames, or raw file content."""
        agg = _import_aggregate()
        result = agg.aggregate(FIXTURES_ROOT, "testuser")

        for date_key, day_data in result["days"].items():
            self.assertEqual(
                set(day_data.keys()),
                {"sessions", "active_minutes"},
                msg=f"Day {date_key} has unexpected keys: {set(day_data.keys())}",
            )
            self.assertIsInstance(day_data["sessions"], int, msg=f"sessions must be int on {date_key}")
            self.assertIsInstance(day_data["active_minutes"], int, msg=f"active_minutes must be int on {date_key}")

    def test_days_values_contain_no_string_leakage(self):
        """No string values in days entries — no machine IDs, paths, or hostnames."""
        agg = _import_aggregate()
        result = agg.aggregate(FIXTURES_ROOT, "testuser")

        for date_key, day_data in result["days"].items():
            for field, value in day_data.items():
                self.assertNotIsInstance(
                    value,
                    str,
                    msg=f"String value found in days[{date_key}][{field}] = {value!r} — data boundary violated (NFR8/NFR9)",
                )

    def test_username_set_correctly(self):
        """username field must equal the value passed to aggregate()."""
        agg = _import_aggregate()
        result = agg.aggregate(FIXTURES_ROOT, "testuser")

        self.assertEqual(result["username"], "testuser")

    def test_generated_at_is_iso8601_utc(self):
        """generated_at must be formatted as YYYY-MM-DDTHH:MM:SSZ (ISO 8601 UTC)."""
        import re

        agg = _import_aggregate()
        result = agg.aggregate(FIXTURES_ROOT, "testuser")

        generated_at = result.get("generated_at", "")
        # Pattern: YYYY-MM-DDTHH:MM:SSZ
        pattern = r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$"
        self.assertRegex(
            generated_at,
            pattern,
            msg=f"generated_at={generated_at!r} must match YYYY-MM-DDTHH:MM:SSZ",
        )


class TestAggregateErrorExit(unittest.TestCase):
    """5.1-UNIT-004: Error paths exit non-zero (P0, R-009)."""

    def test_malformed_data_json_causes_nonzero_exit(self):
        """Given a data.json with invalid JSON, running aggregate.py must exit non-zero."""
        aggregate_script = pathlib.Path(__file__).parent.parent / "aggregate.py"

        with tempfile.TemporaryDirectory() as tmpdir:
            tmproot = pathlib.Path(tmpdir)

            # Create a malformed data.json in a Hive partition
            partition = tmproot / "machines" / "year=2026" / "month=04" / "day=09" / "harness=claude" / "machine_id=bad-machine"
            partition.mkdir(parents=True)
            (partition / "data.json").write_text("THIS IS NOT JSON", encoding="utf-8")

            result = subprocess.run(
                [sys.executable, str(aggregate_script)],
                cwd=tmpdir,
                capture_output=True,
                text=True,
                timeout=30,
            )

            self.assertNotEqual(
                result.returncode,
                0,
                msg="aggregate.py must exit non-zero when data.json is malformed",
            )

    def test_missing_sessions_field_causes_nonzero_exit(self):
        """Given a data.json missing the required 'sessions' field,
        running aggregate.py must exit non-zero."""
        aggregate_script = pathlib.Path(__file__).parent.parent / "aggregate.py"

        with tempfile.TemporaryDirectory() as tmpdir:
            tmproot = pathlib.Path(tmpdir)

            partition = tmproot / "machines" / "year=2026" / "month=04" / "day=09" / "harness=claude" / "machine_id=bad-machine"
            partition.mkdir(parents=True)
            # Missing sessions field
            (partition / "data.json").write_text(
                json.dumps({"active_minutes": 30}), encoding="utf-8"
            )

            result = subprocess.run(
                [sys.executable, str(aggregate_script)],
                cwd=tmpdir,
                capture_output=True,
                text=True,
                timeout=30,
            )

            self.assertNotEqual(
                result.returncode,
                0,
                msg="aggregate.py must exit non-zero when sessions field is missing",
            )


# ---------------------------------------------------------------------------
# P1 Tests (High)
# ---------------------------------------------------------------------------


class TestAggregateSingleMachineBaseline(unittest.TestCase):
    """5.1-UNIT-005: Single machine baseline happy path (P1)."""

    def test_single_machine_single_date_correct_values(self):
        """Given a single active machine with one date, output matches exact values."""
        agg = _import_aggregate()

        with tempfile.TemporaryDirectory() as tmpdir:
            tmproot = pathlib.Path(tmpdir)
            partition = tmproot / "machines" / "year=2026" / "month=04" / "day=09" / "harness=claude" / "machine_id=solo-machine"
            partition.mkdir(parents=True)
            (partition / "data.json").write_text(
                json.dumps({"sessions": 7, "active_minutes": 120}), encoding="utf-8"
            )

            result = agg.aggregate(tmproot, "testuser")

        self.assertEqual(result["days"], {"2026-04-09": {"sessions": 7, "active_minutes": 120}})


class TestAggregateMultipleDates(unittest.TestCase):
    """5.1-UNIT-006: All dates aggregated correctly across multiple dates (P1)."""

    def test_all_three_expected_dates_present(self):
        """Given fixtures with data on 2026-04-09, 2026-04-10, and 2026-04-11,
        all three dates appear in the output."""
        agg = _import_aggregate()
        result = agg.aggregate(FIXTURES_ROOT, "testuser")

        self.assertIn("2026-04-09", result["days"])
        self.assertIn("2026-04-10", result["days"])
        self.assertIn("2026-04-11", result["days"])

    def test_all_dates_match_expected_values(self):
        """All date values in days must match the pre-computed expected output."""
        agg = _import_aggregate()
        result = agg.aggregate(FIXTURES_ROOT, "testuser")

        self.assertEqual(result["days"], EXPECTED_DAYS)


# ---------------------------------------------------------------------------
# P2 Tests (Medium)
# ---------------------------------------------------------------------------


class TestAggregateEmptyHiveDirectory(unittest.TestCase):
    """5.1-UNIT-007: Empty Hive directory → days: {} produced, no error (P2)."""

    def test_empty_machines_directory_returns_empty_days(self):
        """Given no Hive partition files exist, output must contain days: {} without error."""
        agg = _import_aggregate()

        with tempfile.TemporaryDirectory() as tmpdir:
            tmproot = pathlib.Path(tmpdir)
            # Create the machines directory but no partition files
            (tmproot / "machines").mkdir()

            result = agg.aggregate(tmproot, "testuser")

        self.assertEqual(result["days"], {})
        self.assertIn("generated_at", result)
        self.assertIn("username", result)

    def test_missing_machines_directory_returns_empty_days(self):
        """Given no machines/ directory at all, output must contain days: {} without error."""
        agg = _import_aggregate()

        with tempfile.TemporaryDirectory() as tmpdir:
            tmproot = pathlib.Path(tmpdir)
            # No machines directory created at all

            result = agg.aggregate(tmproot, "testuser")

        self.assertEqual(result["days"], {})


class TestAggregateMultipleHarnesses(unittest.TestCase):
    """5.1-UNIT-008: Multiple harness dirs (harness=claude, harness=codex) summed correctly (P2)."""

    def test_claude_and_codex_harness_sessions_summed_on_same_date(self):
        """Given machine-a has data in both harness=claude and harness=codex on 2026-04-09,
        their sessions must be summed (3+1=4 from machine-a alone on that date before machine-b)."""
        agg = _import_aggregate()
        result = agg.aggregate(FIXTURES_ROOT, "testuser")

        # 2026-04-09 total: machine-a/claude(3) + machine-b/claude(2) + machine-a/codex(1) = 6
        self.assertEqual(result["days"]["2026-04-09"]["sessions"], 6)

    def test_claude_and_codex_harness_active_minutes_summed_on_same_date(self):
        """Given machine-a has data in both harness=claude(45m) and harness=codex(10m) on 2026-04-09,
        plus machine-b/claude(30m), total active_minutes must be 85."""
        agg = _import_aggregate()
        result = agg.aggregate(FIXTURES_ROOT, "testuser")

        # 2026-04-09 total: machine-a/claude(45) + machine-b/claude(30) + machine-a/codex(10) = 85
        self.assertEqual(result["days"]["2026-04-09"]["active_minutes"], 85)


class TestAggregateIdempotency(unittest.TestCase):
    """5.1-UNIT-009: Idempotency — running twice produces identical days content (P2)."""

    def test_idempotent_days_output(self):
        """Running aggregate() twice on the same fixtures must produce identical days content.
        Only generated_at is allowed to differ between runs."""
        agg = _import_aggregate()

        result1 = agg.aggregate(FIXTURES_ROOT, "testuser")
        result2 = agg.aggregate(FIXTURES_ROOT, "testuser")

        self.assertEqual(
            result1["days"],
            result2["days"],
            msg="days content must be identical across two runs on the same data",
        )
        self.assertEqual(result1["username"], result2["username"])


class TestAggregateRegistryMissingOrMalformed(unittest.TestCase):
    """5.1-UNIT-010: registry.json absent → all machines included (P2)."""

    def test_missing_registry_includes_all_machines(self):
        """Given no registry.json exists, all machines (including former purged ones) are included."""
        agg = _import_aggregate()

        with tempfile.TemporaryDirectory() as tmpdir:
            tmproot = pathlib.Path(tmpdir)
            # Create a single machine partition, no registry.json
            partition = tmproot / "machines" / "year=2026" / "month=04" / "day=09" / "harness=claude" / "machine_id=any-machine"
            partition.mkdir(parents=True)
            (partition / "data.json").write_text(
                json.dumps({"sessions": 5, "active_minutes": 50}), encoding="utf-8"
            )
            # No registry.json created

            result = agg.aggregate(tmproot, "testuser")

        # Machine should be included since no registry says to skip it
        self.assertEqual(result["days"]["2026-04-09"]["sessions"], 5)

    def test_malformed_registry_treated_as_empty_purged_set(self):
        """Given registry.json exists but contains invalid JSON,
        the purged set is treated as empty and all machines are included."""
        agg = _import_aggregate()

        with tempfile.TemporaryDirectory() as tmpdir:
            tmproot = pathlib.Path(tmpdir)
            partition = tmproot / "machines" / "year=2026" / "month=04" / "day=09" / "harness=claude" / "machine_id=any-machine"
            partition.mkdir(parents=True)
            (partition / "data.json").write_text(
                json.dumps({"sessions": 5, "active_minutes": 50}), encoding="utf-8"
            )
            # Malformed registry.json
            (tmproot / "registry.json").write_text("NOT VALID JSON", encoding="utf-8")

            result = agg.aggregate(tmproot, "testuser")

        # Malformed registry → empty purged set → machine included
        self.assertEqual(result["days"]["2026-04-09"]["sessions"], 5)


if __name__ == "__main__":
    unittest.main()
