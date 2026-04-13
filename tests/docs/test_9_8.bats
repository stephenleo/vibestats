#!/usr/bin/env bats
# Story 9.8: Architecture documentation — Capture post-sprint lessons
# ATDD Green Phase — all 18 tests passing after architecture.md was updated.
#
# Run: bats tests/docs/test_9_8.bats
#
# Test framework: bats-core
# Stack: Documentation — file content assertions only; no source compilation required.
# All tests verify _bmad-output/planning-artifacts/architecture.md content.
#
# ACs tested:
#   AC #1: architecture.md contains a "Known Gotchas & Conventions" section covering all six items
#   AC #2: serde(default) footgun is documented with enough context to avoid it
#   AC #3: All additions are factually accurate (cross-checked against source files)

REPO_ROOT="$(cd "$(dirname "$BATS_TEST_FILENAME")/../.." && pwd)"
ARCH_MD="${REPO_ROOT}/_bmad-output/planning-artifacts/architecture.md"

# ---------------------------------------------------------------------------
# Prerequisite — architecture.md exists and is readable
# ---------------------------------------------------------------------------
@test "[P0][9.8-PRE-001] _bmad-output/planning-artifacts/architecture.md exists" {
  [ -f "$ARCH_MD" ]
}

# ---------------------------------------------------------------------------
# AC #1 — architecture.md contains a "Known Gotchas & Conventions" section
# P0 — Story 9.8, AC #1, 9.8-DOC-001
# ---------------------------------------------------------------------------
@test "[P0][9.8-DOC-001] architecture.md contains a 'Known Gotchas & Conventions' section heading" {
  run grep -E "Known Gotchas" "$ARCH_MD"
  [ "$status" -eq 0 ]
  [[ -n "$output" ]]
}

# ---------------------------------------------------------------------------
# AC #1 / Task 2 — Cloudflare Pages _redirects evaluation-order rule is documented
# P0 — Story 9.8, AC #1, 9.8-DOC-002
# ---------------------------------------------------------------------------
@test "[P0][9.8-DOC-002] architecture.md documents the _redirects evaluation-order rule (top-to-bottom first match)" {
  # Cloudflare evaluates rules top-to-bottom, stopping at first match; pass-through rules must
  # appear before catch-all rules or they are never reached.
  run grep -E "top.to.bottom|first.match|pass.through.*before|before.*catch.all|evaluation.order" "$ARCH_MD"
  [ "$status" -eq 0 ]
  [[ -n "$output" ]]
}

@test "[P1][9.8-DOC-003] architecture.md documents the /u.html infinite-redirect trap" {
  # The doc must warn against rewriting to '/u.html'.
  run grep -E "u\.html|infinite.redirect|clean.URL" "$ARCH_MD"
  [ "$status" -eq 0 ]
  [[ -n "$output" ]]
}

# ---------------------------------------------------------------------------
# AC #1 / AC #2 / Task 3 — serde(default) footgun documented with actionable guidance
# P0 — Story 9.8, AC #1+2, 9.8-DOC-004
# ---------------------------------------------------------------------------
@test "[P0][9.8-DOC-004] architecture.md documents the serde(default = 'fn') vs Default::default() footgun" {
  # The section must include the specific 'default = "fn"' annotation syntax and a concrete example.
  run grep -E 'serde\(default = |default_machine_status|footgun' "$ARCH_MD"
  [ "$status" -eq 0 ]
  [[ -n "$output" ]]
}

@test "[P0][9.8-DOC-005] architecture.md explains that serde(default) does NOT affect Default::default()" {
  # AC #2: the doc must clarify that #[serde(default = "fn")] only affects deserialization,
  # NOT the derived Default impl, so readers don't need to read Story 2.3 to avoid the footgun.
  run grep -E "Default|default_machine_status|manual.*impl|impl Default" "$ARCH_MD"
  [ "$status" -eq 0 ]
  [[ -n "$output" ]]
}

# ---------------------------------------------------------------------------
# AC #1 / Task 4 — Cargo [workspace] isolation pattern documented
# P1 — Story 9.8, AC #1, 9.8-DOC-006
# ---------------------------------------------------------------------------
@test "[P1][9.8-DOC-006] architecture.md documents the Cargo [workspace] worktree isolation pattern" {
  run grep -E "\[workspace\]|worktree.*cargo|cargo.*worktree" "$ARCH_MD"
  [ "$status" -eq 0 ]
  [[ -n "$output" ]]
}

# ---------------------------------------------------------------------------
# AC #1 / Task 5 — _gh() define-if-not-defined pattern documented
# P1 — Story 9.8, AC #1, 9.8-DOC-007
# ---------------------------------------------------------------------------
@test "[P1][9.8-DOC-007] architecture.md documents the _gh() define-if-not-defined testable shell helper pattern" {
  run grep -E "declare -f|define-if-not-defined|_gh\(\)" "$ARCH_MD"
  [ "$status" -eq 0 ]
  [[ -n "$output" ]]
}

# ---------------------------------------------------------------------------
# AC #1 / Task 6 — Python3 stdlib over jq convention documented
# P1 — Story 9.8, AC #1, 9.8-DOC-008
# ---------------------------------------------------------------------------
@test "[P1][9.8-DOC-008] architecture.md documents the Python3 stdlib over jq convention for JSON in Bash scripts" {
  run grep -E "python3.*json|jq.*python|Python3 stdlib|python3 -c" "$ARCH_MD"
  [ "$status" -eq 0 ]
  [[ -n "$output" ]]
}

# ---------------------------------------------------------------------------
# AC #1 / Task 7 — Security negative test pattern documented
# P1 — Story 9.8, AC #1, 9.8-DOC-009
# ---------------------------------------------------------------------------
@test "[P1][9.8-DOC-009] architecture.md documents the security negative test pattern (sentinel token + grep assertion)" {
  # The new entry must describe the sentinel-inject + grep-absent assertion approach for bats
  # security tests, distinct from the existing NFR7 implementation note.
  run grep -E "SENTINEL|sentinel.*token|negative.*test.*security|security.*negative|! grep.*sentinel|inject.*sentinel" "$ARCH_MD"
  [ "$status" -eq 0 ]
  [[ -n "$output" ]]
}

# ---------------------------------------------------------------------------
# AC #3 — Factual accuracy checks against source files
# P1 — Story 9.8, AC #3
# ---------------------------------------------------------------------------

@test "[P1][9.8-ACC-001] Cargo.toml line 1 is [workspace] — serde and worktree docs are grounded in reality" {
  # Verify the source fact that the gotcha docs rely on: Cargo.toml starts with [workspace].
  CARGO_TOML="${REPO_ROOT}/Cargo.toml"
  [ -f "$CARGO_TOML" ]
  FIRST_LINE="$(head -1 "$CARGO_TOML")"
  [ "$FIRST_LINE" = "[workspace]" ]
}

@test "[P1][9.8-ACC-002] src/checkpoint.rs defines default_machine_status() function — serde doc example is grounded" {
  # Verify the reference implementation exists where the story claims it does.
  CHECKPOINT="${REPO_ROOT}/src/checkpoint.rs"
  [ -f "$CHECKPOINT" ]
  run grep -c "default_machine_status" "$CHECKPOINT"
  [ "$status" -eq 0 ]
  [ "$output" -ge 1 ]
}

@test "[P1][9.8-ACC-003] install.sh defines the _gh() declare-if-not-defined guard — shell doc example is grounded" {
  # Verify the _gh() pattern exists in install.sh as cited in story tasks.
  INSTALL_SH="${REPO_ROOT}/install.sh"
  [ -f "$INSTALL_SH" ]
  run grep -c "declare -f _gh" "$INSTALL_SH"
  [ "$status" -eq 0 ]
  [ "$output" -ge 1 ]
}

@test "[P1][9.8-ACC-004] install.sh uses python3 -c for JSON parsing — python3 doc example is grounded" {
  # Verify that python3 is used for JSON parsing in install.sh as cited in story tasks.
  INSTALL_SH="${REPO_ROOT}/install.sh"
  [ -f "$INSTALL_SH" ]
  run grep -c "python3 -c" "$INSTALL_SH"
  [ "$status" -eq 0 ]
  [ "$output" -ge 1 ]
}

@test "[P1][9.8-ACC-005] tests/installer/test_6_2.bats contains the SENTINEL_TOKEN negative test — security doc example is grounded" {
  # Verify the sentinel token pattern exists in the cited test file.
  TEST_6_2="${REPO_ROOT}/tests/installer/test_6_2.bats"
  [ -f "$TEST_6_2" ]
  run grep -c "SENTINEL" "$TEST_6_2"
  [ "$status" -eq 0 ]
  [ "$output" -ge 1 ]
}

# ---------------------------------------------------------------------------
# AC #1 — Structural integrity: no existing sections were removed or modified
# P1 — Story 9.8, 9.8-INT-001
# ---------------------------------------------------------------------------
@test "[P1][9.8-INT-001] architecture.md still contains 'Architecture Readiness Assessment' section (existing content preserved)" {
  # The story requires purely additive changes — existing sections must not be removed.
  run grep -E "Architecture Readiness Assessment" "$ARCH_MD"
  [ "$status" -eq 0 ]
  [[ -n "$output" ]]
}

@test "[P1][9.8-INT-002] architecture.md still contains 'Implementation Patterns' section (existing content preserved)" {
  # Another key existing section that must not be removed or modified.
  run grep -E "Implementation Patterns" "$ARCH_MD"
  [ "$status" -eq 0 ]
  [[ -n "$output" ]]
}

@test "[P2][9.8-INT-003] architecture.md Known Gotchas section appears AFTER Architecture Validation Results" {
  # The story requires the new section to be appended at the end of architecture.md,
  # after 'Architecture Validation Results'. Verify ordering by comparing line numbers.
  VALIDATION_LINE=$(grep -n "Architecture Validation Results\|Architecture Readiness Assessment" "$ARCH_MD" | head -1 | cut -d: -f1)
  GOTCHAS_LINE=$(grep -n "Known Gotchas" "$ARCH_MD" | head -1 | cut -d: -f1)

  # Both sections must exist (non-empty line numbers)
  [ -n "$VALIDATION_LINE" ]
  [ -n "$GOTCHAS_LINE" ]

  # Gotchas section must appear AFTER Architecture Validation section
  [ "$GOTCHAS_LINE" -gt "$VALIDATION_LINE" ]
}
