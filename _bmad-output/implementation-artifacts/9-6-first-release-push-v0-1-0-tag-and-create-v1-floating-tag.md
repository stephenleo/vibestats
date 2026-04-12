# Story 9.6: First release — push v0.1.0 tag and create v1 floating tag

Status: backlog

<!-- GH Issue: #86 | Epic: #80 | PR must include: Closes #86 -->

## Story

As a user who wants to install vibestats,
I want a published GitHub Release with downloadable binaries and a working Marketplace action reference,
So that `curl -sSf https://vibestats.dev/install.sh | bash` and `uses: stephenleo/vibestats@v1` both work end-to-end.

## Background

Epic 8 (CI/CD & Distribution) delivered the release workflow and Marketplace prerequisites, but the actual release was never triggered. Two outstanding items from the Epic 8 retrospective:
1. The `release.yml` workflow has been schema-validated but **not runtime-validated** — the first tag push is the real test
2. `uses: stephenleo/vibestats@v1` requires a `v1` floating tag; without it, Marketplace users will get errors
3. There is a documented TLS/cross-compilation risk for the Linux build: `ureq` may fail with OpenSSL when using `cross` — the `rustls` fallback is pre-documented

Additionally, Story 8.3 marked all Marketplace prerequisites as met but the actual UI submission step is a manual action.

Source: Epic 8 retrospective Technical Debt #1 (TLS validation), #2 (v1 floating tag), Action Item (Marketplace submission).

## Acceptance Criteria

1. **Given** `git push origin v0.1.0` is executed **When** the `release.yml` workflow completes **Then** a GitHub Release exists at `https://github.com/stephenleo/vibestats/releases/tag/v0.1.0` with three binary assets: `vibestats-aarch64-apple-darwin.tar.gz`, `vibestats-x86_64-apple-darwin.tar.gz`, `vibestats-x86_64-unknown-linux-gnu.tar.gz`.

2. **Given** the release exists **When** the `v1` floating tag is created **Then** `git tag v1 v0.1.0 && git push origin v1` succeeds and `uses: stephenleo/vibestats@v1` resolves to the `v0.1.0` commit.

3. **Given** the `release.yml` Linux build uses `cross` for cross-compilation **When** the workflow runs **Then** it either succeeds natively OR the `rustls` fallback (documented in Story 8.1 Dev Notes) is applied and the build succeeds.

4. **Given** all Marketplace prerequisites were verified in Story 8.3 **When** the Marketplace submission UI is navigated **Then** the action is submitted for review (manual step — cannot be automated).

5. **Given** the Cloudflare Pages site is ready to deploy **When** the `deploy-site.yml` workflow is triggered via `workflow_dispatch` **Then** `vibestats.dev` serves the landing page from the current main branch.

## Tasks / Subtasks

- [ ] Task 1: Pre-release checklist — verify everything is green before tagging
  - [ ] Run `cargo test` from repo root — must be 0 failures
  - [ ] Run `cargo clippy --all-targets -- -D warnings` — must be 0 warnings (Story 9.5 should be done first)
  - [ ] Run `bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats tests/installer/test_6_3.bats tests/installer/test_6_4.bats` — must be 0 failures (Story 9.3 should be done first)
  - [ ] Confirm `action.yml` has `branding.icon`, `branding.color`, `name`, `description` (Story 5.4 deliverables)
  - [ ] Confirm `CONTRIBUTING.md` has the Release Versioning section (Story 8.3 deliverable)

- [ ] Task 2: Tag and push v0.1.0
  - [ ] Ensure working tree is clean on main: `git status`
  - [ ] Create annotated tag: `git tag -a v0.1.0 -m "Initial release — vibestats v0.1.0"`
  - [ ] Push tag: `git push origin v0.1.0`
  - [ ] Monitor the `release.yml` Actions run on GitHub

- [ ] Task 3: Handle TLS/cross-compilation result
  - [ ] If the Linux build succeeds: document in Dev Agent Record that `cross` + OpenSSL worked as-is
  - [ ] If the Linux build fails with an OpenSSL/TLS error: apply the `rustls` fallback documented in Story 8.1 Dev Notes
    - Add `features = ["rustls"]` to the `ureq` entry in `Cargo.toml`
    - Add `default-features = false` to disable the `native-tls` feature
    - Re-run: delete the failed tag with `git tag -d v0.1.0 && git push origin :refs/tags/v0.1.0`, fix `Cargo.toml`, commit, re-tag, re-push
  - [ ] Verify all three platform binaries are present in the GitHub Release once the workflow passes

- [ ] Task 4: Create v1 floating tag
  - [ ] After v0.1.0 release is confirmed green:
  - [ ] `git tag v1 v0.1.0`
  - [ ] `git push origin v1`
  - [ ] Verify `uses: stephenleo/vibestats@v1` resolves correctly by checking the tag on GitHub

- [ ] Task 5: Trigger Cloudflare Pages deployment
  - [ ] Navigate to the `deploy-site.yml` workflow in the GitHub Actions UI
  - [ ] Trigger via `workflow_dispatch` with `ref: main`
  - [ ] Confirm `vibestats.dev` loads correctly after deployment

- [ ] Task 6: Submit to GitHub Actions Marketplace (manual UI step)
  - [ ] Follow the Marketplace submission process documented in Story 8.3 Dev Notes
  - [ ] Submit the action for review
  - [ ] Document the submission status in this story's Dev Agent Record

## Dev Notes

**IMPORTANT — Story ordering:** This story should only run after:
- Story 9.3 (test_6_2.bats failures resolved) — broken installer tests should not ship with v0.1.0
- Story 9.5 (dead_code suppressors removed) — clean lint is part of release quality

**rustls fallback (if needed):**
In `Cargo.toml`, change the `ureq` entry from:
```toml
ureq = { version = "2.x", features = [] }
```
to:
```toml
ureq = { version = "2.x", default-features = false, features = ["rustls"] }
```
This avoids the OpenSSL system dependency that `cross` can't link when cross-compiling for Linux. `rustls` is a pure-Rust TLS implementation.

**v1 floating tag convention (from CONTRIBUTING.md):**
Per the release versioning section added in Story 8.3, `v1` is a floating tag that should always point to the latest v1.x.x release. After each patch: `git tag -f v1 <new-tag> && git push origin v1 --force`.

**Tag deletion if needed:**
If a tag was pushed with an error, delete it with:
```bash
git tag -d v0.1.0 && git push origin :refs/tags/v0.1.0
```

## Review Criteria

- GitHub Release at `github.com/stephenleo/vibestats/releases/tag/v0.1.0` exists with 3 binary assets
- `git tag --list v1` shows the v1 tag exists and points to v0.1.0
- Cloudflare Pages deployment succeeded (`vibestats.dev` loads)
- Marketplace submission initiated (manual — document status)
- All binaries downloadable and `.tar.gz` archives extract to a valid `vibestats` binary
