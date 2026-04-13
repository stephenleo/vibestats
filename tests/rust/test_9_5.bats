#!/usr/bin/env bats
# Story 9.5: Rust — Remove dead_code suppressors and verify lint clean
# ATDD Red Phase — tests assert expected behaviour; will fail until suppressors are removed
# and clippy/tests pass clean.
#
# Run: bats tests/rust/test_9_5.bats
#
# Test framework: bats-core
# Stack: Rust (Cargo) — backend-only, no UI
# All tests run from repo root; cargo must be on PATH.
#
# ACs tested:
#   AC #1: No #![allow(dead_code)] exists anywhere in src/
#   AC #2: cargo clippy --all-targets -- -D warnings exits 0 with 0 warnings
#   AC #3: Any residual dead_code warning is resolved per decision tree (no module-level blanket)
#   AC #4: cargo test exits 0 with no regressions

REPO_ROOT="$(cd "$(dirname "$BATS_TEST_FILENAME")/../.." && pwd)"

setup() {
  cd "$REPO_ROOT"
}

# ---------------------------------------------------------------------------
# AC #1 — No module-level #![allow(dead_code)] suppressors remain in src/
# P0 — Story 9.5, AC #1
# ---------------------------------------------------------------------------
@test "[P0] no module-level allow(dead_code) suppressors remain in src/" {
  # Passes when no #![allow(dead_code)] lines exist anywhere in src/.
  # Expected pre-implementation files (all now removed):
  #   src/config.rs:1, src/logger.rs:16, src/checkpoint.rs:1,
  #   src/jsonl_parser.rs:1, src/github_api.rs:22, src/hooks/mod.rs:1
  run grep -rn "#!\[allow(dead_code)\]" src/
  [ "$status" -ne 0 ]  # grep exits 1 when no matches — this is the passing condition
  [ -z "$output" ]      # no output means no suppressors found
}

# ---------------------------------------------------------------------------
# AC #1 — Specifically: src/config.rs has no module-level dead_code suppressor
# P0 — Story 9.5, AC #1
# ---------------------------------------------------------------------------
@test "[P0] src/config.rs does not contain #![allow(dead_code)]" {
  # RED: src/config.rs currently has #![allow(dead_code)] on line 1
  run grep -c "#!\[allow(dead_code)\]" src/config.rs
  [ "$status" -ne 0 ] || [ "$output" = "0" ]
}

# ---------------------------------------------------------------------------
# AC #1 — Specifically: src/logger.rs has no module-level dead_code suppressor
# P0 — Story 9.5, AC #1
# ---------------------------------------------------------------------------
@test "[P0] src/logger.rs does not contain #![allow(dead_code)]" {
  # RED: src/logger.rs currently has #![allow(dead_code)] on line 16
  run grep -c "#!\[allow(dead_code)\]" src/logger.rs
  [ "$status" -ne 0 ] || [ "$output" = "0" ]
}

# ---------------------------------------------------------------------------
# AC #1 — Specifically: src/checkpoint.rs has no module-level dead_code suppressor
# P0 — Story 9.5, AC #1
# ---------------------------------------------------------------------------
@test "[P0] src/checkpoint.rs does not contain #![allow(dead_code)]" {
  # RED: src/checkpoint.rs currently has #![allow(dead_code)] on line 1
  run grep -c "#!\[allow(dead_code)\]" src/checkpoint.rs
  [ "$status" -ne 0 ] || [ "$output" = "0" ]
}

# ---------------------------------------------------------------------------
# AC #1 — Specifically: src/jsonl_parser.rs has no module-level dead_code suppressor
# P0 — Story 9.5, AC #1
# ---------------------------------------------------------------------------
@test "[P0] src/jsonl_parser.rs does not contain #![allow(dead_code)]" {
  # RED: src/jsonl_parser.rs currently has #![allow(dead_code)] on line 1
  run grep -c "#!\[allow(dead_code)\]" src/jsonl_parser.rs
  [ "$status" -ne 0 ] || [ "$output" = "0" ]
}

# ---------------------------------------------------------------------------
# AC #1 — Specifically: src/github_api.rs has no module-level dead_code suppressor
#          BUT retains #![allow(clippy::result_large_err)]
# P0 — Story 9.5, AC #1
# ---------------------------------------------------------------------------
@test "[P0] src/github_api.rs does not contain #![allow(dead_code)]" {
  # RED: src/github_api.rs currently has #![allow(dead_code)] on line 22
  run grep -c "#!\[allow(dead_code)\]" src/github_api.rs
  [ "$status" -ne 0 ] || [ "$output" = "0" ]
}

@test "[P1] src/github_api.rs still retains #![allow(clippy::result_large_err)]" {
  # This suppressor must NOT be removed — it's a valid permanent suppression
  # (ureq::Error is ~272 bytes, a third-party constraint)
  run grep -c "#!\[allow(clippy::result_large_err)\]" src/github_api.rs
  [ "$status" -eq 0 ]
  [ "$output" = "1" ]
}

# ---------------------------------------------------------------------------
# AC #1 — Specifically: src/hooks/mod.rs has no module-level dead_code suppressor
# P0 — Story 9.5, AC #1
# ---------------------------------------------------------------------------
@test "[P0] src/hooks/mod.rs does not contain #![allow(dead_code)]" {
  # RED: src/hooks/mod.rs currently has #![allow(dead_code)] on line 1
  run grep -c "#!\[allow(dead_code)\]" src/hooks/mod.rs
  [ "$status" -ne 0 ] || [ "$output" = "0" ]
}

# ---------------------------------------------------------------------------
# AC #3 — Any item-level #[allow(dead_code)] that remains must have a comment
# P1 — Story 9.5, AC #3
# ---------------------------------------------------------------------------
@test "[P1] any remaining item-level allow(dead_code) attributes are accompanied by an explanatory comment" {
  # If any #[allow(dead_code)] (item-level, not module-level) remains,
  # it must appear with an inline comment on the same or preceding line.
  # This test finds item-level allows (no #! prefix) and checks each is commented.
  #
  # After implementation: either no item-level allows exist (test trivially passes),
  # or each one is accompanied by "// intentionally kept:" or "// cfg_attr" pattern.
  local item_level_allows
  item_level_allows="$(grep -rn "^[[:space:]]*#\[allow(dead_code)\]" src/ 2>/dev/null || true)"

  if [ -z "$item_level_allows" ]; then
    # No item-level allows at all — pass
    return 0
  fi

  # For each item-level allow found, check for a comment on the same or prior line
  while IFS= read -r line; do
    local file lineno
    file="$(echo "$line" | cut -d: -f1)"
    lineno="$(echo "$line" | cut -d: -f2)"
    local prev_line
    prev_line="$(sed -n "$((lineno - 1))p" "$file")"
    local curr_line
    curr_line="$(sed -n "${lineno}p" "$file")"
    # Acceptable: prev line or curr line contains a // comment or a #[cfg_attr
    if echo "$prev_line $curr_line" | grep -qE "(//|#\[cfg_attr)"; then
      continue
    fi
    echo "FAIL: uncommented item-level allow(dead_code) at $file:$lineno"
    return 1
  done <<< "$item_level_allows"
}

# ---------------------------------------------------------------------------
# AC #2 — cargo clippy exits 0 with -D warnings (zero warnings)
# P0 — Story 9.5, AC #2
# ---------------------------------------------------------------------------
@test "[P0] cargo clippy --all-targets -- -D warnings exits 0 with zero warnings" {
  # RED: clippy will emit dead_code warnings for the six modules once suppressors are removed.
  # This test passes only after all dead code is removed or given targeted allows with comments.
  run cargo clippy --all-targets -- -D warnings
  [ "$status" -eq 0 ]
}

# ---------------------------------------------------------------------------
# AC #4 — cargo test exits 0 (no regressions)
# P0 — Story 9.5, AC #4
# ---------------------------------------------------------------------------
@test "[P0] cargo test exits 0 with no test failures" {
  # RED (partial): cargo test currently passes because suppressors mask dead_code warnings,
  # but after suppressor removal and any code deletion, all tests must still pass.
  run cargo test
  [ "$status" -eq 0 ]
}
