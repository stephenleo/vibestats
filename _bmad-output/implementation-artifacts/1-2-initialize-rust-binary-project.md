# Story 1.2: Initialize Rust Binary Project

Status: review

<!-- GH Issue: #10 | Epic: #1 | PR must include: Closes #10 -->

## Story

As a developer,
I want the Rust binary project initialized with all required crates declared,
so that the project compiles with a working CLI skeleton before any feature code is written.

## Acceptance Criteria

1. **Given** the Rust binary source and dependencies are in place **When** `cargo build` is executed **Then** the project compiles without errors

2. **Given** the project is initialized **When** `Cargo.toml` is reviewed **Then** `clap` (with `derive` feature), `serde` + `serde_json` (with `derive` feature), `ureq`, and `toml` are declared with pinned versions

3. **Given** the project is initialized **When** `src/main.rs` is reviewed **Then** a `clap` CLI skeleton defines `sync`, `status`, `machines`, `auth`, and `uninstall` subcommands, each with a stub handler that prints "not yet implemented"

## Tasks / Subtasks

- [x] Task 1: Replace stub `src/.gitkeep` with a working Rust binary entry point (AC: #1)
  - [x] Remove `src/.gitkeep` (do NOT run `cargo new` — `Cargo.toml` and repo already exist from Story 1.1)
  - [x] Create `src/main.rs` with `clap` CLI skeleton (see Dev Notes for exact structure)
  - [x] Ensure `src/main.rs` compiles with `cargo build` without errors or warnings

- [x] Task 2: Update `Cargo.toml` with pinned dependency versions (AC: #2)
  - [x] Add `clap` with `derive` feature (pin to latest stable — see Dev Notes for version guidance)
  - [x] Add `serde` with `derive` feature
  - [x] Add `serde_json`
  - [x] Add `ureq`
  - [x] Add `toml`
  - [x] Confirm all versions are pinned exactly (not `"*"` or range specifiers)

- [x] Task 3: Verify compilation and CLI skeleton (AC: #3)
  - [x] Run `cargo build` — must succeed with 0 errors
  - [x] Run `cargo clippy` — must produce 0 warnings
  - [x] Run `./target/debug/vibestats sync` — must print "not yet implemented" and exit 0
  - [x] Run `./target/debug/vibestats status` — must print "not yet implemented" and exit 0
  - [x] Run `./target/debug/vibestats machines list` — must print "not yet implemented" and exit 0
  - [x] Run `./target/debug/vibestats machines remove test-id` — must print "not yet implemented" and exit 0
  - [x] Run `./target/debug/vibestats auth` — must print "not yet implemented" and exit 0
  - [x] Run `./target/debug/vibestats uninstall` — must print "not yet implemented" and exit 0
  - [x] Confirm `Cargo.lock` is present and committed (binary project — lockfile is kept in VCS)

## Dev Notes

### Context: What Story 1.1 Created

Story 1.1 created a minimal stub `Cargo.toml` at the repo root with no dependencies:

```toml
[package]
name = "vibestats"
version = "0.1.0"
edition = "2021"
```

It also created `src/.gitkeep` as a placeholder. This story replaces the placeholder with real source code and adds all required dependencies to `Cargo.toml`.

**IMPORTANT: Do NOT run `cargo new`** — the repo and `Cargo.toml` stub already exist from Story 1.1. Running `cargo new` would overwrite the existing `Cargo.toml` and create a nested package. Simply edit the existing files and create `src/main.rs` by hand.

**Files to touch:**
- `Cargo.toml` — add `[dependencies]` section (do NOT change `[package]` section)
- `src/.gitkeep` — delete this file
- `src/main.rs` — create with CLI skeleton

### Required Dependencies and Versions

Use these crates with minimum-version specifiers in `Cargo.toml`. Exact pinning for a binary is achieved via `Cargo.lock` (which must be committed). Do NOT use `"*"` or open-ended ranges like `">=4"`. The versions below are minimum compatible versions:

| Crate | Version (latest stable as of 2026-04) | Features |
|---|---|---|
| `clap` | `"4.5"` | `features = ["derive"]` |
| `serde` | `"1.0"` | `features = ["derive"]` |
| `serde_json` | `"1.0"` | none |
| `ureq` | `"2.10"` | none |
| `toml` | `"0.8"` | none |

**Why these crates (from architecture.md):**
- `clap` — CLI argument parsing with derived subcommands (`status`, `sync`, `machines`, `auth`, `uninstall`)
- `serde` + `serde_json` — JSONL parsing with `#[serde(default)]` for schema tolerance (NFR14)
- `ureq` — minimal sync HTTP client for GitHub Contents API; chosen over `reqwest` because no async runtime needed (hook async is handled at the Claude Code level, not the binary level)
- `toml` — config file read/write (`~/.config/vibestats/config.toml`, `checkpoint.toml`)
- **No `tokio` or async runtime** — this is an intentional architectural decision (ADR)

### `src/main.rs` Required Structure

The CLI must define exactly these subcommands matching the architecture spec:

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "vibestats", version, about = "Track your Claude Code session activity")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Sync session data to vibestats-data
    Sync {
        /// Run a full historical backfill
        #[arg(long)]
        backfill: bool,
    },
    /// Show current sync status and last sync time
    Status,
    /// Manage registered machines
    Machines {
        #[command(subcommand)]
        subcommand: MachinesSubcommand,
    },
    /// Authenticate with GitHub
    Auth,
    /// Uninstall vibestats
    Uninstall,
}

#[derive(Subcommand)]
enum MachinesSubcommand {
    /// List all registered machines
    List,
    /// Remove a machine by ID
    Remove {
        /// Machine ID to remove
        machine_id: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Sync { backfill: _ } => println!("not yet implemented"),
        Commands::Status => println!("not yet implemented"),
        Commands::Machines { subcommand } => match subcommand {
            MachinesSubcommand::List => println!("not yet implemented"),
            MachinesSubcommand::Remove { machine_id: _ } => println!("not yet implemented"),
        },
        Commands::Auth => println!("not yet implemented"),
        Commands::Uninstall => println!("not yet implemented"),
    }
}
```

**CLI naming conventions (from architecture.md):**
- Subcommands: `kebab-case` (e.g., `machines list`, `machines remove`, `sync --backfill`)
- Flags: `--kebab-case` (e.g., `--backfill`)

### Module Files — Do NOT Create Yet

The architecture defines these future files in `src/`:

```
src/
├── main.rs              ← THIS STORY
├── config.rs            ← Story 2.1
├── checkpoint.rs        ← Story 2.3
├── logger.rs            ← Story 2.2
├── jsonl_parser.rs      ← Story 2.4
├── github_api.rs        ← Story 2.5
├── sync.rs              ← Story 3.1
└── commands/
    ├── mod.rs           ← Epic 4
    ├── sync.rs          ← Epic 4 / Story 3.4
    ├── status.rs        ← Epic 4 / Story 4.1
    ├── machines.rs      ← Epic 4 / Story 4.2
    ├── auth.rs          ← Epic 4 / Story 4.3
    └── uninstall.rs     ← Epic 4 / Story 4.4
```

**Do NOT create module files** (`config.rs`, `logger.rs`, etc.) in this story. Those are fully owned by Epic 2 stories. Creating stubs now will cause confusion when later stories expect those modules to not exist yet.

### `Cargo.toml` Final State

```toml
[package]
name = "vibestats"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
ureq = "2.10"
toml = "0.8"
```

### `Cargo.lock` Must Be Committed

Binary projects commit `Cargo.lock` — it is NOT in `.gitignore` (which only ignores `/target/`). The story 1.1 `.gitignore` already correctly excludes `/target/` only, not `Cargo.lock`. Confirm `Cargo.lock` appears as a new tracked file in `git status` after `cargo build`.

### No Tests Required

This story sets up project scaffolding only — no business logic to test. Verification is via `cargo build` + manual `./target/debug/vibestats <subcommand>` invocations as listed in Task 3. The architecture spec calls for co-located `#[cfg(test)]` modules in each `.rs` file, but `main.rs` CLI routing stub requires no unit tests.

### Previous Story Review Learnings

From Story 1.1 review:
- The `.gitignore` was patched post-review to restore `_bmad/`, `.claude/`, `.agents/` and secret-file entries. Confirm those entries remain untouched in this story (this story does not modify `.gitignore`).
- `Cargo.lock` was deliberately absent in story 1.1 (no `cargo build` ran yet). This story is the first time `cargo build` runs and generates `Cargo.lock`. It must be committed.

### Architecture Constraints Summary

| Constraint | Source | Impact on This Story |
|---|---|---|
| No async runtime (tokio) | architecture.md#Selected Starters | Do NOT add `tokio` to `Cargo.toml` |
| `ureq` over `reqwest` | architecture.md#Rust Sync Binary | Use `ureq` for HTTP, not `reqwest` |
| `snake_case` Rust filenames | architecture.md#Naming Patterns | `main.rs`, not `Main.rs` |
| CLI: `kebab-case` subcommands | architecture.md#Naming Patterns | `machines list`, `machines remove` |
| Binary project: commit lockfile | architecture.md#Complete Project Directory | `Cargo.lock` in VCS |
| All errors exit 0 | architecture.md#Process Patterns | Not applicable yet in stub |

### References

- Rust crate decisions: [Source: architecture.md#Selected Starters by Component - Rust Sync Binary]
- Module file layout: [Source: architecture.md#Complete Project Directory Structure]
- CLI subcommand names: [Source: architecture.md#Naming Patterns]
- No async runtime: [Source: architecture.md#Rust Sync Binary — "No async runtime (tokio)"]
- Story 1.2 ACs: [Source: epics.md#Story 1.2: Initialize Rust binary project]
- Previous story file list: [Source: implementation-artifacts/1-1-initialize-monorepo-directory-structure.md#File List]

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

- Encountered cargo workspace traversal issue: the worktree directory `.worktrees/story-1.2-...` is nested inside the main repo, so cargo walked up and found the parent `Cargo.toml` (which has no `src/main.rs`). Fixed by adding `[workspace]` section to the worktree's `Cargo.toml` to prevent upward traversal. This is standard practice for standalone Rust packages nested within a monorepo or worktree hierarchy.

### Completion Notes List

- Deleted `src/.gitkeep` and created `src/main.rs` with exact clap CLI skeleton from Dev Notes
- Updated `Cargo.toml` with `[workspace]` section (worktree isolation) and all 5 required dependencies at specified minimum versions: clap 4.5 (derive), serde 1.0 (derive), serde_json 1.0, ureq 2.10, toml 0.8
- `cargo build` succeeded with 0 errors; `cargo clippy` produced 0 warnings
- All 6 subcommands (sync, status, machines list, machines remove, auth, uninstall) print "not yet implemented" and exit 0
- `Cargo.lock` generated and staged for commit (100 packages locked)
- `.gitignore` not modified — confirmed untouched per story requirements
- No async runtime (tokio) added — intentional architectural decision maintained

### File List

- Cargo.toml (modified — added [workspace] section and [dependencies])
- Cargo.lock (new — generated by cargo build, must be committed)
- src/main.rs (new — clap CLI skeleton with 5 subcommands)
- src/.gitkeep (deleted)
- _bmad-output/implementation-artifacts/1-2-initialize-rust-binary-project.md (modified — story file updates)
- _bmad-output/implementation-artifacts/sprint-status.yaml (modified — status: in-progress)

## Change Log

- 2026-04-11: Implemented story 1.2 — Rust binary project initialized with clap CLI skeleton and all required dependencies. cargo build and clippy pass with 0 errors/warnings. All 6 CLI subcommands verified functional.
