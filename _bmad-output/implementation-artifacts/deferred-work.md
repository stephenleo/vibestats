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
