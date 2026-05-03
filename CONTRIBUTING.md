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
