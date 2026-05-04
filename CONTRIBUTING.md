# Contributing to vibestats

Thank you for your interest in contributing to vibestats!

## Getting Started

1. Fork the repository and clone your fork.
2. Create a feature branch: `git checkout -b feature/your-feature-name`
3. Make your changes following the conventions below.
4. Run tests and linting before submitting.
5. Open a pull request against `main`.

## Code Conventions

- **Rust**: `snake_case` for files and identifiers; run `cargo fmt` and `cargo clippy` before committing.
- **Python**: `snake_case` for files and identifiers; follow PEP 8.
- **Shell**: `kebab-case` for script names; use `shellcheck` for linting.
- **YAML**: `kebab-case` for file names.

## Commit Messages

Use clear, imperative commit messages (e.g., `add aggregate.py stub`, `fix config parsing`).

## Running checks before submitting a PR

CI enforces four checks. Run them locally before pushing:

```sh
# 1. Format - CI runs `cargo fmt --check`; fix formatting first
cargo fmt

# 2. Lint - all Clippy warnings are treated as errors
cargo clippy -- -D warnings

# 3. Tests
cargo test

# 4. Release build
cargo build --release
```

## Adding a new harness

vibestats supports multiple AI coding harnesses (Claude Code, Codex, …) via a
trait + registry under `src/harnesses/`. Each harness lives in exactly one file
and must implement the [`Harness`](src/harnesses/mod.rs) trait. End to end,
adding a harness has two parts: the **Rust parser** (which reads the tool's
local session files) and the **installer hook wiring** (which makes the tool
call `vibestats sync` after each session).

### Part 1 — Rust parser

The `Harness` trait:

```rust
pub trait Harness: Sync {
    /// Stable id used in CLI args, checkpoint keys, and hive paths.
    /// MUST be lowercase ASCII with no spaces. Changing this is a breaking
    /// change to the on-disk and on-remote data format.
    fn id(&self) -> &'static str;

    /// Human-readable name for log/error messages.
    fn display_name(&self) -> &'static str;

    /// Returns true if this harness is installed on the local machine.
    fn is_installed(&self) -> bool;

    /// Walk local session files and aggregate per-day activity for dates in
    /// `[start, end]` inclusive (YYYY-MM-DD strings). Returns an empty map
    /// when the harness is not installed or has no data.
    fn parse_date_range(&self, start: &str, end: &str)
        -> std::collections::HashMap<String, crate::harnesses::DailyActivity>;
}
```

Steps:

1. **Create `src/harnesses/<name>.rs`.** Define `pub struct <Name>;` and
   `impl Harness for <Name>`. Keep all parser internals (serde structs, file
   walking, date parsing) private to this module. Use
   `crate::harnesses::DailyActivity` as the per-day output type — do not
   introduce a parallel struct. The existing `claude.rs` and `codex.rs` are
   the reference implementations.
2. **Register the harness** in `src/harnesses/mod.rs`:
   - add `pub mod <name>;` next to the other module declarations,
   - add `&<name>::<Name>,` to the `REGISTRY` array.

After these two changes the new id is automatically available to the CLI,
sync, backfill, and checkpoint subsystems — no other Rust files need editing.

### Part 2 — Installer hook wiring (`install.sh`)

vibestats relies on each harness firing `vibestats sync --quiet` after every
session via that tool's native hooks mechanism. Because every tool stores hook
configuration in its own format and location, each harness needs its own
`configure_<name>_hooks` shell function in `install.sh`. Skip this part only
if the new tool has no hooks system.

The pattern, mirrored in the existing `configure_hooks` (Claude) and
`configure_codex_hooks` (Codex):

```bash
configure_<name>_hooks() {
  TOOL_DIR="${HOME}/.<tool>"
  if [ ! -d "${TOOL_DIR}" ]; then
    return 0    # tool not installed on this machine — silently skip
  fi

  TOOL_HOOKS="${TOOL_DIR}/<hooks-file>"          # e.g. hooks.json, settings.json
  python3 - "$TOOL_HOOKS" <<'PYEOF'
import json, sys
from pathlib import Path

hooks_path = Path(sys.argv[1])
try:
    with hooks_path.open("r") as f:
        doc = json.load(f)
except (FileNotFoundError, json.JSONDecodeError):
    doc = {}

# … idempotently insert Stop and SessionStart hooks that run
#   "vibestats sync --quiet" without duplicating existing entries.
#   See configure_codex_hooks in install.sh for a complete example.

hooks_path.parent.mkdir(parents=True, exist_ok=True)
with hooks_path.open("w") as f:
    json.dump(doc, f, indent=2)
    f.write("\n")
PYEOF
  echo "<Name> hooks configured in ${TOOL_HOOKS}"
}
```

Then call it from `configure_hooks` so the installer wires every present tool
in one pass:

```bash
configure_hooks() {
  # … existing Claude wiring …
  configure_codex_hooks
  configure_<name>_hooks    # ← add this line
}
```

Constraints to respect:

- **Idempotent.** Re-running the installer must not duplicate hook entries.
  The existing functions check for the presence of a `vibestats sync` command
  before appending.
- **Python 3 stdlib only** — no `jq`, no third-party libraries. macOS and
  Linux both ship Python 3 by default.
- **Use `vibestats sync --quiet`** as the hook command. The `--quiet` flag
  suppresses human-readable output that would otherwise pollute hook stdout
  (which some harnesses parse).
- **Wire both Stop and SessionStart hooks** if the tool exposes them. Stop
  triggers an after-session sync; SessionStart catches up on missed days from
  prior sessions.

### Checklist before opening a PR

- [ ] Unit tests for the new harness are in a `#[cfg(test)] mod tests` block
      at the bottom of `src/harnesses/<name>.rs`.
- [ ] `cargo run -- sync --help` lists the new id under `possible values:`.
- [ ] `cargo run -- sync --harness <new-id>` runs end-to-end on a machine
      with that tool installed (try both `--backfill` and the default).
- [ ] The id matches the `harness=<id>` segment used in hive paths and the
      `<id>:date` checkpoint keys (lowercase, no spaces).
- [ ] `bash install.sh` (or just sourcing and calling `configure_hooks`)
      writes the new hook entries on a machine where the tool is installed,
      and is a no-op when re-run.
- [ ] `cargo fmt && cargo clippy -- -D warnings && cargo test` all pass.

## Reporting Issues

Please open a GitHub issue with a clear description of the problem and steps to reproduce it.

## Release Versioning

This project follows semantic versioning: `vMAJOR.MINOR.PATCH` tags (e.g., `v1.0.0`).

### Floating Major Tag Pattern

The current major-version tag is kept in sync with the latest release in that major line. Users pinned to
`uses: stephenleo/vibestats@v2` automatically receive patch and minor updates within the `v2` line.

When a new major is released, older major tags are **not** deleted or force-updated — users pinned to
`@v1` continue to receive the last `v1.x.x` release unchanged. This is the same versioning contract used by
`actions/checkout@v4`, `actions/setup-python@v5`, and other community actions.

### Maintainer Checklist (After Every Release)

Before publishing, ensure the Cargo package version matches the release tag without the leading `v`
(for example, `version = "2.0.0"` for tag `v2.0.0`). The release workflow enforces this.

After publishing a new `v2.x.x` release, maintainers must update the floating major tag:

```bash
git tag -f v2 v2.x.x          # replace v2.x.x with the new version
git push --force origin v2
```

The GitHub Actions Marketplace listing always references the latest stable major tag. Never force-update
a major tag across a breaking-change boundary — cut a new major instead.
