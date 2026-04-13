# Story 9.4: Bash installer — Refactor EXIT trap to composable cleanup

Status: done

<!-- GH Issue: #84 | Epic: #80 | PR must include: Closes #84 -->

## Story

As a developer maintaining `install.sh`,
I want the EXIT trap to use a composable cleanup function rather than a single override,
So that any future addition to `install.sh` can register cleanup tasks without silently discarding earlier ones.

## Background

Identified in Story 6.1's code review and deferred four consecutive times (Stories 6.1, 6.2, 6.3, 6.4). The current pattern in `download_and_install_binary()`:

```bash
trap 'rm -rf "$TMPDIR_WORK"' EXIT
```

This **replaces any previously registered EXIT trap**. If a future story adds another `trap ... EXIT` elsewhere in `install.sh`, one of them will be silently dropped. The Epic 6 retrospective's "last-story resolution policy" states that items deferred to the final story of an epic must become explicit post-epic tech debt — which is this story.

Source: Story 6.1 Dev Agent Record (Review Findings), Epic 6 retrospective Technical Debt #2, `deferred-work.md`.

## Acceptance Criteria

1. **Given** `install.sh` currently overwrites the EXIT trap inside `download_and_install_binary()` **When** this story is complete **Then** the inline `trap 'rm -rf "$TMPDIR_WORK"' EXIT` is removed from that function and replaced by assigning `TMPDIR_WORK` to a script-level variable that the shared `cleanup()` function uses.

2. **Given** a `cleanup()` function registered via `trap cleanup EXIT` at the top of `install.sh` **When** `install.sh` terminates (normally or via `set -e` error) **Then** the temp directory created by `download_and_install_binary()` is removed.

3. **Given** the refactored cleanup mechanism **When** `bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats tests/installer/test_6_3.bats tests/installer/test_6_4.bats` is run **Then** all tests still pass with 0 failures.

4. **Given** a future developer needs to add a new cleanup action **When** they follow the established pattern **Then** they can add a new guarded `rm -rf` (or equivalent) to `cleanup()` without touching the `trap` registration or any other function.

## Tasks / Subtasks

- [x] Task 1: Locate all `trap` calls in `install.sh` (AC: #1)
  - [x] Run `grep -n 'trap' install.sh` and identify every line with a `trap` call
  - [x] Confirm there is exactly one relevant `trap ... EXIT` inside `download_and_install_binary()` at approximately line 137
  - [x] Note: there are no other EXIT traps to worry about in the current `install.sh` (as of Epic 6 close)

- [x] Task 2: Implement the composable cleanup pattern in `install.sh` (AC: #1, #2, #4)
  - [x] Add `_CLEANUP_TMPDIR=""` as a script-level variable immediately after the `_gh()` helper block (before any function definitions), so it is always initialized when the script is sourced
  - [x] Add a `cleanup()` function immediately after the `_CLEANUP_TMPDIR` declaration:
    ```bash
    cleanup() {
      if [ -n "$_CLEANUP_TMPDIR" ]; then
        rm -rf "$_CLEANUP_TMPDIR"
      fi
    }
    trap cleanup EXIT
    ```
  - [x] In `download_and_install_binary()`: remove the line `trap 'rm -rf "$TMPDIR_WORK"' EXIT`
  - [x] In `download_and_install_binary()`: replace the removed trap line with `_CLEANUP_TMPDIR="$TMPDIR_WORK"` (placed immediately after the `mktemp -d` call that creates `TMPDIR_WORK`)
  - [x] Verify `download_and_install_binary()` no longer contains any `trap` call

- [x] Task 3: Verify Bash 3.2 compatibility (AC: #1, #2, #4)
  - [x] Confirm `_CLEANUP_TMPDIR=""` initialization is a simple string variable (Bash 3.2 compatible)
  - [x] Confirm `cleanup()` uses only `[ -n ... ]` and `rm -rf` (POSIX-safe)
  - [x] Confirm no `+=`, `declare -A`, `mapfile`, or `[[ =~ ]]` regex captures are introduced
  - [x] Run `bash -n install.sh` — must exit 0 with no syntax errors

- [x] Task 4: Run full bats test suite and confirm 0 regressions (AC: #3)
  - [x] Run `bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats tests/installer/test_6_3.bats tests/installer/test_6_4.bats`
  - [x] All tests must pass (0 failures) — this story depends on Story 9.3 having resolved the pre-existing `test_6_2.bats` failures first
  - [x] If failures are found: check whether they are pre-existing (Story 9.3 not yet merged) or introduced by this change

## Dev Notes

### File to Modify

**Single file: `install.sh` at repo root.**

Do NOT modify:
- Any `.bats` test files — no test changes required for this refactor
- Any files in `src/`, `action/`, `.github/workflows/`, `vibestats-site/`

### Current State of `install.sh` (Post-Story 9.3)

The file has 618 lines total. The only `trap` call is at line 137 inside `download_and_install_binary()`:

```bash
TMPDIR_WORK="$(mktemp -d)"
trap 'rm -rf "$TMPDIR_WORK"' EXIT    # <-- THIS IS THE LINE TO REFACTOR
```

There are no other `trap` calls anywhere in `install.sh`. This refactor adds one new `trap cleanup EXIT` at script level and removes the inline one from `download_and_install_binary()`.

### Exact Implementation

**Step 1:** After the `_gh()` define-if-not-defined block (ends around line 16), add:

```bash
# ---------------------------------------------------------------------------
# Composable cleanup — accumulates cleanup tasks; registered once via trap.
# Add new cleanup tasks by extending cleanup() body with guarded rm/unset calls.
# ---------------------------------------------------------------------------
_CLEANUP_TMPDIR=""
cleanup() {
  [ -n "$_CLEANUP_TMPDIR" ] && rm -rf "$_CLEANUP_TMPDIR"
}
trap cleanup EXIT
```

**Step 2:** In `download_and_install_binary()`, replace the two-line block:

```bash
# BEFORE:
TMPDIR_WORK="$(mktemp -d)"
trap 'rm -rf "$TMPDIR_WORK"' EXIT

# AFTER:
TMPDIR_WORK="$(mktemp -d)"
_CLEANUP_TMPDIR="$TMPDIR_WORK"
```

That is the complete change. No other modifications to `install.sh` are needed.

### Why This Pattern

The Epic 6 retrospective explicitly documented the fix pattern:
> "Fix pattern: use `trap 'cleanup' EXIT` with a named `cleanup()` function that accumulates cleanup tasks, or append to the trap handler."
> — Epic 6 retrospective Technical Debt #2

The simple global-variable approach (`_CLEANUP_TMPDIR`) is chosen over the append-to-trap approach (`trap "$(trap -p EXIT); new_cmd" EXIT`) because:
1. `trap -p` behavior differs between Bash versions and is unreliable on Bash 3.2 (macOS default)
2. The global variable is easier to read, test, and extend
3. Only one cleanup action exists in the current installer; the extensible pattern is `cleanup()` body

The alternative `eval`-based pattern from the draft story notes is intentionally NOT used — `eval` is a security risk in a script that handles external inputs (e.g., user-provided paths can be injected via `$HOME`).

### Bash 3.2 Constraints (macOS default shell)

This refactor must remain Bash 3.2-compatible:

| Construct | Status |
|-----------|--------|
| `_CLEANUP_TMPDIR=""` string variable | SAFE — basic string assignment |
| `[ -n "$_CLEANUP_TMPDIR" ]` | SAFE — POSIX `[` test |
| `trap cleanup EXIT` | SAFE — named function trap works in Bash 3.2 |
| `trap -p EXIT` | UNSAFE — do NOT use; unreliable in Bash 3.2 |
| `declare -A` associative arrays | UNSAFE — Bash 4+ only |
| `+=` array append | UNSAFE — Bash 4+ only |
| `mapfile` / `readarray` | UNSAFE — Bash 4+ only |
| `[[ =~ ]]` regex captures | UNSAFE — Bash 4+ behavior |

### Test Impact Analysis

This change does NOT affect test behavior because:

1. **`test_6_1.bats`**: tests call functions by sourcing `install.sh`. The `trap cleanup EXIT` at script level will be registered when sourced, but in bats tests functions are called individually inside `bash --noprofile --norc -c "..."` subshells. Each subshell exits after the function call, triggering `cleanup()` — which does nothing if `_CLEANUP_TMPDIR` was not set by that test. No test invokes `download_and_install_binary()` directly.

2. **`test_6_2.bats`**, **`test_6_3.bats`**, **`test_6_4.bats`**: same isolation pattern. No test in these files calls `download_and_install_binary()` directly.

3. The `_CLEANUP_TMPDIR=""` initialization at script level means when `install.sh` is sourced in tests, `_CLEANUP_TMPDIR` is an empty string. The `cleanup()` function's `[ -n "$_CLEANUP_TMPDIR" ]` guard is false, so `rm -rf` is NOT called — correct behavior.

### Story 9.3 Dependency

This story modifies `install.sh`. Story 9.3 (fix `test_6_2.bats` failures) modifies `tests/installer/test_6_2.bats` only — it does NOT touch `install.sh`. There is no file conflict between the two stories.

However, to verify AC #3 (0 test regressions), Story 9.3 must be merged first. If working before Story 9.3 is merged, the pre-existing `test_6_2.bats` failures will appear but are unrelated to this change.

**Status as of 2026-04-13:** Story 9.3 is `done` and merged to `main`. The worktree for this story was created from `main` (commit `455756d`) which includes the Story 9.3 fix. All 4 bats test files should pass cleanly.

### Verification Commands

```bash
# Syntax check — must exit 0
bash -n install.sh

# Confirm old inline trap is gone
grep -n 'trap.*TMPDIR_WORK' install.sh
# Expected: no output (no matches)

# Confirm new cleanup function exists
grep -n 'cleanup' install.sh
# Expected: _CLEANUP_TMPDIR="", cleanup() { ..., trap cleanup EXIT, _CLEANUP_TMPDIR="$TMPDIR_WORK"

# Full bats regression suite — must exit 0 with 0 failures
bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats tests/installer/test_6_3.bats tests/installer/test_6_4.bats
```

### Project Structure Notes

- `install.sh` — single file at repo root; all changes are in this file only
- Pattern alignment: the `_gh()` helper already uses a define-if-not-defined guard for testability; the `cleanup()` function follows the same philosophy of "define once at script level, usable by all functions"
- `_CLEANUP_TMPDIR` uses underscore prefix to signal it is an internal/private variable (consistent with `_gh()` naming convention already established in `install.sh`)

### References

- Story 6.1 Dev Agent Record, Review Findings: `[Review][Defer] EXIT trap override in download_and_install_binary may conflict with future cleanup traps` [Source: `_bmad-output/implementation-artifacts/6-1-implement-dependency-detection-and-gh-authentication.md`]
- Epic 6 retrospective, Technical Debt #2: "Resolve EXIT trap conflict in `download_and_install_binary()`" [Source: `_bmad-output/implementation-artifacts/epic-6-retro-2026-04-12.md`]
- Architecture: Bash installer must work on macOS Bash 3.2 [Source: `_bmad-output/planning-artifacts/architecture.md#bash-installer`]
- Epic 9, Story 9.4: [#84](https://github.com/stephenleo/vibestats/issues/84) [Source: `_bmad-output/planning-artifacts/epic-9.md`]

## Review Criteria

- `install.sh` has no inline `trap ... EXIT` call inside `download_and_install_binary()`
- A `cleanup()` function exists and is registered with `trap cleanup EXIT` at script level (outside any function)
- `_CLEANUP_TMPDIR=""` is initialized at script level before `cleanup()` is defined
- `bash -n install.sh` exits 0 (syntax clean)
- `bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats tests/installer/test_6_3.bats tests/installer/test_6_4.bats` exits 0 with 0 failures
- Pattern is Bash 3.2 compatible (no `declare -A`, no `+=` array append, no `trap -p`, no `eval`)
- No test files were modified

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Agent Actions Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-04-13 | Story created | BMad Create-Story — Story 9.4 |

### Completion Notes List

- Refactored `install.sh`: added `_CLEANUP_TMPDIR=""` global, `cleanup()` function using `if/fi` guard (not `&&`) to ensure zero exit status when variable is unset, and `trap cleanup EXIT` registered once at top level before `_gh()` helper block.
- Removed inline `trap 'rm -rf "$TMPDIR_WORK"' EXIT` from `download_and_install_binary()`; replaced with `_CLEANUP_TMPDIR="$TMPDIR_WORK"`.
- Used `if [ -n "$_CLEANUP_TMPDIR" ]; then rm -rf...; fi` instead of `[ -n "$_CLEANUP_TMPDIR" ] && rm -rf...` — the `&&` form returns exit status 1 when condition is false, propagating as the shell's final exit code when sourced in bats tests.
- All 10 new ATDD tests (test_9_4.bats) pass: 10/10.
- All 42 regression tests (test_6_1 through test_6_4) pass: 42/42. Zero regressions.
- `bash -n install.sh` exits 0. Pattern is Bash 3.2 compatible.

### File List

- `install.sh` (modified — add `_CLEANUP_TMPDIR` global, add `cleanup()` function with `trap cleanup EXIT`, remove inline trap from `download_and_install_binary()`, replace with `_CLEANUP_TMPDIR="$TMPDIR_WORK"`)

### Change Log

- 2026-04-13: Implemented composable EXIT trap cleanup pattern. Added `_CLEANUP_TMPDIR=""` global and `cleanup()` function; registered `trap cleanup EXIT` once at top level; replaced inline trap override in `download_and_install_binary()` with `_CLEANUP_TMPDIR="$TMPDIR_WORK"`. All 52 tests pass (10 new + 42 regression).
