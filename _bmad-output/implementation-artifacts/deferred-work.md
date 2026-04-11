# Deferred Work

Items surfaced during reviews that are intentionally not acted on in the story
that raised them. Revisit when the blocking rationale no longer applies.

## Deferred from: code review of story 2-3-implement-checkpoint-module (2026-04-11)

- **No concurrent-writer lock on checkpoint.toml** [src/checkpoint.rs:105] — Two
  simultaneous Stop hook invocations could race on the checkpoint file. In
  practice Claude Code serializes Stop hooks per session so this is currently
  unobservable, but if we ever fan out parallel sync workers we will need a
  file lock (POSIX `flock`, `fs2`, or a `.lock` sentinel file). Revisit when
  story 3.x wires up the real sync loop.
- **`Box<dyn std::error::Error>` is not `Send + Sync`** [src/checkpoint.rs:112]
  — `Checkpoint::save` returns a boxed error that cannot cross thread
  boundaries. The architecture explicitly prohibits async runtimes (no tokio,
  no reqwest), so this is not currently a problem. If we ever introduce
  threaded sync we should switch to `Box<dyn Error + Send + Sync>` or a
  concrete error enum.

## Deferred from: code review of story 2-5-implement-github-api-module (2026-04-11)

- **No percent-encoding of URL path/repo components in Contents API calls**
  [src/github_api.rs:187, 236] — `get_file_sha_inner` and `put_file_inner`
  interpolate `repo` and `path` directly into the URL with `format!`. Today's
  callers (future `sync.rs`) compose Hive paths from alphanumerics, `=`,
  `/`, and `-`, and the repo slug is `owner/name` — all URL-safe in practice.
  A defensive percent-encoding pass (stdlib-only, no `percent-encoding`
  crate) would harden the module against future callers that happen to pass
  a space, `#`, or `?`. Revisit when any other module needs to compose paths
  dynamically.
- **`with_retry` backoff tests sleep real wall-clock seconds** [src/github_api.rs:84]
  — `test_retry_transport_error_exhausts_3_attempts` and
  `test_retry_succeeds_after_two_transport_errors` each sleep the full 1s+2s
  backoff sequence because `delays_secs` is a hardcoded constant. The suite
  still finishes in ~3s total, but if test-suite runtime becomes a concern we
  can make the delay array an injectable parameter (or compile-time `cfg!(test)`
  override) so retry tests become effectively free.

## Deferred from: code review of story 3-4-implement-vibestats-sync-and-vibestats-sync-backfill-commands (2026-04-11)

- **`vibestats sync --backfill` stdout does not report actual synced/failed
  counts** [src/commands/sync.rs:44-69] — AC #2 wording asks for "count of dates
  synced and any failures", but the implementation reports only the count of
  dates **found in JSONL** (since `sync::run` returns `()` and hides the
  changed-vs-skipped split from its caller). Errors are routed to
  `vibestats.log` via `logger::error`, never to stdout. This design is
  **explicitly documented and justified** in the story Dev Notes ("the CLI is
  user-initiated; errors belong in the log"). To truly satisfy AC #2's literal
  wording, `sync::run` would need a richer return type (e.g.
  `struct SyncReport { synced: u32, skipped: u32, failed: u32 }`), which is
  out of scope for this story. Revisit if/when a future story wants to surface
  sync outcomes on stdout or in `vibestats status`.

## Deferred from: code review of story 4-2-implement-vibestats-machines-list-and-machines-remove (2026-04-11)

- **Remote purge walks entire `machines/` tree for every target machine**
  [src/commands/machines.rs:279] — `purge_remote` does a depth-first walk of
  `machines/year=*/month=*/day=*/harness=*/machine_id=*` and only filters by
  the target machine_id at the leaf level. For a multi-year dataset this can
  easily produce hundreds of GET calls against the 5000/hour GitHub rate
  limit. This is architecturally intended per the story Dev Notes ("more
  network-intensive but necessary for remote purge"); a future optimization
  could use the Git Trees API (GET /repos/{owner}/{repo}/git/trees/{sha}?recursive=1)
  to fetch the entire repo tree in a single call, then filter client-side.
- **No unit tests for `purge_self` / `purge_remote` / `update_local_checkpoint`**
  [src/commands/machines.rs:231-385] — These helpers call through `GithubApi`
  and `Checkpoint::save`, both of which hit real I/O. Testing them requires
  either a mocking framework (forbidden by story "no new crates" constraint)
  or extracting the business logic into pure functions that take trait objects.
  Revisit if/when the project adopts a `GithubApi` trait for testability.

## Deferred from: code review of story 5-1-implement-aggregate-py (2026-04-11)

- **`action/tests/fixtures/expected_output/data.json` is not referenced by any
  test** [action/tests/fixtures/expected_output/data.json] — Tests assert
  against the in-file `EXPECTED_DAYS` constant instead of loading the fixture.
  The fixture is kept because story 5.1 task list explicitly requires it to
  exist, but future work could either (a) wire the fixture into a test that
  parses it and compares `days`, or (b) remove the dead file once the story
  constraint is revisited.
- **No file-size cap when reading per-machine `data.json` files**
  [action/aggregate.py:73] — `aggregate()` reads every matched Hive partition
  file into memory via `json.load`. In the production Actions context files
  are machine-written and tiny (<1 KB), but a hostile write to
  `vibestats-data` could OOM the runner. The repo is owner-controlled so this
  is low risk; revisit only if the vibestats-data repo ever accepts writes
  from unverified sources.

## Deferred from: code review of story 5-3-implement-update-readme-py (2026-04-11)

- **No validation for empty `--username`** [action/update_readme.py:20-24] —
  `argparse` currently accepts `--username ""`, which would produce broken URLs
  (`https://raw.githubusercontent.com///main/vibestats/heatmap.svg`) and a
  bogus dashboard link without surfacing an error. In practice the GitHub
  Actions workflow always passes a real login, so this is not exploitable
  today. Revisit if the script ever becomes callable from user-facing tooling
  or if we want defence-in-depth against workflow misconfiguration.

## Deferred from: code review of story 5-4-implement-action-yml (2026-04-12)

- **Push retry loop does not `git pull --rebase` between retries**
  [action.yml:70-74] — The 3-retry loop in the "Push to profile-repo" step
  only mitigates transient network errors. If a push fails because the remote
  moved (race with another concurrent vibestats run against the same
  profile-repo), every retry will also fail because the local branch is still
  behind. The architecture spec calls for a simple retry, not rebase-retry,
  so this matches the design. Revisit if we ever support multiple machines
  writing to the same profile-repo concurrently — story 5.5 (`aggregate.yml`)
  should set a `concurrency:` group on the calling workflow as the primary
  mitigation, with rebase-on-conflict as a fallback inside this action.
- **`update_readme.py` and `git add README.md` fail loudly if profile-repo
  has no `README.md`** [action.yml:54, action.yml:62] — A brand-new
  `username/username` repo without a README would crash the action at the
  `update_readme.py` step (file-not-found) or, if somehow that step were
  skipped, at the `git add README.md` step. Profile READMEs always exist by
  GitHub convention (the `username/username` repo's README is the user-visible
  GitHub profile), so this is the user-visible failure we want per NFR13.
  Revisit only if onboarding ever needs to support an empty profile-repo —
  in that case, install.sh (Epic 6) should create a stub README before the
  first action run.
