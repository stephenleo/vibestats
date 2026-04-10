# Contributing to vibestats

Thank you for your interest in contributing!

## Running Locally

1. Clone the repository:
   ```sh
   git clone https://github.com/stephenleo/vibestats.git
   cd vibestats
   ```

2. Build the Rust binary:
   ```sh
   cargo build
   ```

3. Run the Astro site (after Story 1.3 initialization):
   ```sh
   cd site
   npm install
   npm run dev
   ```

## Pull Request Process

1. Fork the repository and create a feature branch from `main`.
2. Make your changes with clear, focused commits.
3. Ensure all tests pass before submitting.
4. Open a pull request against `main` with a clear description of your changes.
5. Reference any related GitHub issues in the PR description.

## Code Style

- Rust: follow `rustfmt` defaults (`cargo fmt`)
- Python: follow PEP 8
- Shell: use `#!/usr/bin/env bash` and `set -euo pipefail`
