# Story 9.4: Bash installer — Refactor EXIT trap to composable cleanup

Status: backlog

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

Source: `deferred-work.md` (Story 6.1 review), Epic 6 retrospective Technical Debt #2.

## Acceptance Criteria

1. **Given** `install.sh` currently overwrites the EXIT trap in `download_and_install_binary()` **When** this story is complete **Then** the EXIT trap is replaced with a named `cleanup()` function that accumulates cleanup tasks and is registered once.

2. **Given** the `cleanup()` function is registered via `trap cleanup EXIT` **When** `install.sh` terminates (normally or via `set -e` error) **Then** all accumulated cleanup tasks execute (e.g., temp directory removal).

3. **Given** the refactored cleanup mechanism **When** `bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats tests/installer/test_6_3.bats tests/installer/test_6_4.bats` is run **Then** all tests still pass with 0 failures.

4. **Given** a future developer needs to add a new cleanup action **When** they follow the established pattern **Then** they can add to the `cleanup()` function body without risk of dropping existing cleanup tasks.

## Tasks / Subtasks

- [ ] Task 1: Read current `install.sh` exit trap usage
  - [ ] Search for all `trap` calls in `install.sh`
  - [ ] Identify every location where `trap ... EXIT` is used
  - [ ] Note what each trap does (temp dir removal, etc.)

- [ ] Task 2: Implement the composable cleanup pattern
  - [ ] Add a `_CLEANUP_TMPDIR=""` global variable at the top of `install.sh` (or appropriate scope)
  - [ ] Add a `cleanup()` function that performs all cleanup actions using the accumulated state variables:
    ```bash
    cleanup() {
      [ -n "$_CLEANUP_TMPDIR" ] && rm -rf "$_CLEANUP_TMPDIR"
    }
    trap cleanup EXIT
    ```
  - [ ] Register `trap cleanup EXIT` once, early in `install.sh` (before any function that could set a trap)
  - [ ] In `download_and_install_binary()`: replace `trap 'rm -rf "$TMPDIR_WORK"' EXIT` with `_CLEANUP_TMPDIR="$TMPDIR_WORK"` (sets the variable for cleanup() to use)
  - [ ] Remove the inline trap override from `download_and_install_binary()`

- [ ] Task 3: Verify Bash 3.2 compatibility
  - [ ] Confirm the pattern uses only Bash 3.2-compatible syntax (no `declare -A`, no `+=` on arrays)
  - [ ] The global variable approach (`_CLEANUP_TMPDIR`) is Bash 3.2-compatible
  - [ ] Test with `bash --version` check if available

- [ ] Task 4: Run full bats test suite and confirm 0 regressions
  - [ ] Run `bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats tests/installer/test_6_3.bats tests/installer/test_6_4.bats`
  - [ ] All must pass (this story depends on Story 9.3 having resolved any pre-existing failures first)

## Dev Notes

**Bash 3.2 constraints (macOS default shell):**
- No `declare -A` associative arrays
- No `mapfile`/`readarray`
- No `[[ =~ ]]` regex capture groups
- No `+=` for array append
- The simple variable accumulation approach (`_CLEANUP_TMPDIR=""` set before use) is compatible.

**Alternative pattern if multiple independent cleanup actions are needed:**
```bash
_CLEANUP_ACTIONS=""
add_cleanup() { _CLEANUP_ACTIONS="${_CLEANUP_ACTIONS:+$_CLEANUP_ACTIONS;}$1"; }
cleanup() { eval "$_CLEANUP_ACTIONS"; }
trap cleanup EXIT
```
This is more flexible but adds complexity. Use the simple global variable approach unless more than one cleanup action needs to be accumulated.

**Do NOT use `trap "$(trap -p EXIT)..."` pattern** — it's Bash 4+ behavior and doesn't work reliably on Bash 3.2.

**Note on Story 9.3 dependency:** This story modifies `install.sh`. If Story 9.3 (fix test_6_2.bats failures) has not been completed first, there may be pre-existing test failures that make it hard to confirm whether this story's changes introduced regressions. Recommend completing Story 9.3 before this story.

## Review Criteria

- `install.sh` has no inline `trap ... EXIT` call inside `download_and_install_binary()`
- A `cleanup()` function exists and is registered with `trap cleanup EXIT`
- Full bats suite passes with 0 failures
- Pattern is Bash 3.2 compatible
