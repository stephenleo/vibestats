# Story 8.1: Implement Rust binary release CI

Status: review

<!-- GH Issue: #39 | Epic: #8 | PR must include: Closes #39 -->

## Story

As a vibestats user,
I want pre-compiled binaries for macOS arm64, macOS x86_64, and Linux x86_64 automatically published to GitHub Releases on every tag,
so that install.sh can download the correct binary without requiring Rust to be installed.

## Acceptance Criteria

1. **Given** a git tag matching `v*` is pushed **When** `release.yml` runs **Then** it triggers a matrix build using the `cross` crate for targets: `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu` (FR41)

2. **Given** all three targets compile **When** the release step runs **Then** each binary is archived as `vibestats-<target>.tar.gz` and attached to the GitHub Release (FR41)

3. **Given** any compilation target fails **When** the workflow exits **Then** it exits non-zero and no partial release is published

## Tasks / Subtasks

- [x] Task 1: Create `.github/workflows/release.yml` (AC: #1, #2, #3)
  - [x] Set trigger: `on: push: tags: ['v*']` — no branch or PR triggers
  - [x] Define matrix with `fail-fast: true` and targets: `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`
  - [x] Install `cross` for cross-compilation (use `cross-rs/cross` action or `cargo install cross`)
  - [x] Build step: `cross build --release --target ${{ matrix.target }}`
  - [x] Archive step: `tar czf vibestats-${{ matrix.target }}.tar.gz -C target/${{ matrix.target }}/release vibestats`
  - [x] Upload artifact step: upload `vibestats-${{ matrix.target }}.tar.gz` as a build artifact
  - [x] Create GitHub Release step: triggered by the tag push, attaches all three archives
  - [x] Pin all `uses:` action references to major version tags (e.g., `@v4`) — never `@main` or `@master`
  - [x] Use `${{ github.ref_name }}` for the release tag — no hardcoded version strings

- [x] Task 2: Verify `Cargo.toml` is release-ready (AC: #1)
  - [x] Confirm `Cargo.toml` has `name = "vibestats"` and a valid `version` field (currently `0.1.0`)
  - [x] Confirm binary name resolves to `vibestats` (default for `cargo new vibestats --bin`)
  - [x] No changes needed unless `name` or build settings differ from defaults

## Dev Notes

### File to Create

Single new file: `.github/workflows/release.yml` (relative to repo root).

Do NOT modify:
- `.github/workflows/aggregate.yml` (Epic 5 — user workflow template; independent)
- `action.yml` (Epic 5 — community action; no binary release logic)
- `Cargo.toml` (already set up in Epic 1/2; verify only)

### Required Workflow Structure

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    name: Build ${{ matrix.target }}
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: true
      matrix:
        include:
          - target: aarch64-apple-darwin
            os: macos-latest
          - target: x86_64-apple-darwin
            os: macos-latest
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install cross (Linux target only)
        if: matrix.target == 'x86_64-unknown-linux-gnu'
        run: cargo install cross --locked

      - name: Build (macOS native)
        if: matrix.target != 'x86_64-unknown-linux-gnu'
        run: cargo build --release --target ${{ matrix.target }}

      - name: Build (cross for Linux)
        if: matrix.target == 'x86_64-unknown-linux-gnu'
        run: cross build --release --target ${{ matrix.target }}

      - name: Archive binary
        run: |
          tar czf vibestats-${{ matrix.target }}.tar.gz \
            -C target/${{ matrix.target }}/release vibestats

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: vibestats-${{ matrix.target }}
          path: vibestats-${{ matrix.target }}.tar.gz

  release:
    name: Create GitHub Release
    needs: build
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - name: Download all artifacts
        uses: actions/download-artifact@v4

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          tag_name: ${{ github.ref_name }}
          name: ${{ github.ref_name }}
          files: |
            vibestats-aarch64-apple-darwin/vibestats-aarch64-apple-darwin.tar.gz
            vibestats-x86_64-apple-darwin/vibestats-x86_64-apple-darwin.tar.gz
            vibestats-x86_64-unknown-linux-gnu/vibestats-x86_64-unknown-linux-gnu.tar.gz
```

**Key design decisions:**
- macOS targets use native `cargo build` (no cross-compilation needed — macOS runners support both arm64 and x86_64 natively via `macos-latest`)
- Linux target uses `cross` for cross-compilation reliability
- `fail-fast: true` ensures no partial release if any target fails
- `release` job uses `needs: build` — build job failure prevents this job from running
- `permissions: contents: write` required for `softprops/action-gh-release` to create release

### Archive Naming — Critical for Epic 6

The archive names MUST follow the exact pattern `vibestats-<target>.tar.gz` where `<target>` is one of:
- `vibestats-aarch64-apple-darwin.tar.gz`
- `vibestats-x86_64-apple-darwin.tar.gz`
- `vibestats-x86_64-unknown-linux-gnu.tar.gz`

Epic 6 (`install.sh`) will construct download URLs from these filenames using `uname -s` + `uname -m`. Any deviation breaks the installer. The archive contains a single file named `vibestats` (the binary).

### Cross-Compilation Strategy

Architecture specifies `cross` crate for cross-compilation (architecture.md — Infrastructure & Deployment section). However:
- macOS arm64 (`aarch64-apple-darwin`): use `macos-latest` runner with `cargo build --release --target aarch64-apple-darwin` — macOS runners have universal SDK
- macOS x86_64 (`x86_64-apple-darwin`): use `macos-latest` runner with `cargo build --release --target x86_64-apple-darwin`
- Linux x86_64 (`x86_64-unknown-linux-gnu`): use `cross` — matches architecture spec exactly

Alternative: use `cross` for ALL targets uniformly if native macOS builds fail due to target support. The P0 schema test (R-002) asserts `cross` is used for the Linux target at minimum.

### Cargo Dependencies

Current `Cargo.toml` dependencies (from Epic 1/2 setup):
```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
ureq = "2.10"
toml = "0.8"
```

All dependencies are cross-platform compatible. `ureq` uses native TLS on each platform — no platform-specific feature flags needed for the release build.

### Action Pinning Requirements (P1 Test — R-006)

All `uses:` references must be pinned to major version tags, not `@main` or `@master`:
- `actions/checkout@v4` ✓
- `dtolnay/rust-toolchain@stable` ✓ (uses channel name, not SHA — acceptable)
- `actions/upload-artifact@v4` ✓
- `actions/download-artifact@v4` ✓
- `softprops/action-gh-release@v2` ✓

### Release Tag Convention

- Format: `v<semver>` — e.g., `v0.1.0`, `v1.0.0`
- `Cargo.toml` version `0.1.0` → first release tag `v0.1.0`
- `github.ref_name` resolves to the tag value (e.g., `v0.1.0`) — use this in the release step, not hardcoded strings

### P0 Test Assertions (from test-design-epic-8.md)

The test framework will assert these properties of `release.yml`:
1. Matrix targets exactly: `{aarch64-apple-darwin, x86_64-apple-darwin, x86_64-unknown-linux-gnu}`
2. `fail-fast: true` in matrix strategy
3. Archive name template uses `vibestats-${{ matrix.target }}.tar.gz`
4. Workflow trigger is `on: push: tags: ['v*']` only — no branch/PR triggers
5. `cross` used for Linux target (`x86_64-unknown-linux-gnu`)
6. All `uses:` pinned to version tags (not `@main`/`@master`)
7. Release step uses `${{ github.ref_name }}` — no hardcoded version

Design the `release.yml` to pass all these assertions.

### Architecture References

- **Targets**: architecture.md → Infrastructure & Deployment: `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`
- **Tool**: architecture.md → "Matrix build using `cross` crate for cross-compilation"
- **Archive format**: architecture.md → "Produces `.tar.gz` archives attached to GitHub Release"
- **Trigger**: architecture.md → "Triggered on `git tag v*`"
- **Workflow file location**: architecture.md → `.github/workflows/release.yml`

### What Already Exists

From Epic 5 — do NOT modify:
- `.github/workflows/aggregate.yml` — user's vibestats-data workflow template
- `action.yml` — community GitHub Action (has `branding`, `inputs`, `runs: composite` from Story 5.4)

From Epics 1–4 — Rust binary is fully implemented. The release pipeline packages the already-working binary, it does not change it.

### Repo Permissions

The `release` job needs `permissions: contents: write` to create a GitHub Release and upload assets. This must be set at the job level (not inherited from default, which may be read-only).

### TLS on Linux Cross-Compilation (Potential Issue)

`ureq 2.10` uses `native-tls` by default on Linux. When cross-compiling with `cross` for `x86_64-unknown-linux-gnu`, OpenSSL headers may not be available in the cross Docker image. If the build fails with OpenSSL-related errors, add to `Cargo.toml`:

```toml
[dependencies]
ureq = { version = "2.10", features = ["native-tls"] }
# OR switch to rustls backend:
ureq = { version = "2.10", features = ["tls"], default-features = false }
```

The `cross` Docker image (`ghcr.io/cross-rs/x86_64-unknown-linux-gnu`) includes OpenSSL, so native-tls should work. If it fails, switch to `ureq` with `rustls` feature — pure Rust TLS, no system library dependency. This is the safest option for cross-compilation.

### References

- Story 8.1 definition [Source: _bmad-output/planning-artifacts/epics.md#Story-8.1]
- Architecture CI/CD section [Source: _bmad-output/planning-artifacts/architecture.md — Infrastructure & Deployment]
- Test design risks R-001, R-002, R-006, R-007 [Source: _bmad-output/test-artifacts/test-design-epic-8.md]
- FR41: cross-platform binary distribution [Source: _bmad-output/planning-artifacts/epics.md#Functional-Requirements]
- Existing workflow not to modify [Source: .github/workflows/aggregate.yml]
- Cargo.toml current deps [Source: Cargo.toml]

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None.

### Completion Notes List

- Task 1 complete: Created `.github/workflows/release.yml` with matrix build for three targets (`aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`). macOS targets use native `cargo build`, Linux uses `cross build`. `fail-fast: true` ensures no partial releases. All action references pinned to major version tags. Release job uses `softprops/action-gh-release@v2` with `permissions: contents: write` and `${{ github.ref_name }}` for the tag.
- Task 2 complete: Verified `Cargo.toml` has `name = "vibestats"` and `version = "0.1.0"` — no changes needed.
- All 17 ATDD schema tests pass (GREEN phase). Full test suite: 101 passed, 0 failures, 0 regressions.

### File List

- `.github/workflows/release.yml` (created)
- `action/tests/test_release_yml.py` (modified — removed `@pytest.mark.skip` decorators, TDD green phase)

### Change Log

- 2026-04-12: Implemented Story 8.1 — created `.github/workflows/release.yml` (Rust binary release CI). Activated 17 ATDD schema tests. All 101 tests pass.
