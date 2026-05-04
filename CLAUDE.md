# CLAUDE.md

This is the **vibestats** CLI repo (Rust). General contributor docs are in
[`CONTRIBUTING.md`](CONTRIBUTING.md).

## Adding or modifying a harness

When asked to add a new harness or change an existing one, **read the
[Adding a new harness](CONTRIBUTING.md#adding-a-new-harness) section in
`CONTRIBUTING.md` first.** That section defines the `Harness` trait contract,
the file layout under `src/harnesses/`, and the registry pattern. Do not
introduce parallel data types or dispatch sites — the trait + registry is the
single point of extension.
