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
