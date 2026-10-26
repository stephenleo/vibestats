<h1 align="center">⚡ VibeStats</h1>

<p align="center">
  <a href="https://github.com/stephenleo/vibestats/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/stephenleo/vibestats/ci.yml?branch=main&label=CI&style=flat-square" alt="CI"></a>
  <a href="https://crates.io/crates/vibestats"><img src="https://img.shields.io/crates/v/vibestats?style=flat-square" alt="Crates.io version"></a>
  <a href="https://crates.io/crates/vibestats"><img src="https://img.shields.io/crates/d/vibestats?label=crates%20downloads&style=flat-square" alt="Crates downloads"></a>
  <a href="https://github.com/stephenleo/vibestats/releases/latest"><img src="https://img.shields.io/github/v/release/stephenleo/vibestats?style=flat-square" alt="GitHub release"></a>
  <a href="https://github.com/stephenleo/vibestats/releases"><img src="https://img.shields.io/github/downloads/stephenleo/vibestats/total?label=github%20downloads&style=flat-square" alt="GitHub downloads"></a>
  <a href="https://github.com/stephenleo/vibestats/blob/main/LICENSE"><img src="https://img.shields.io/github/license/stephenleo/vibestats?style=flat-square" alt="License"></a>
</p>

<p align="center">
  Track your Claude Code and Codex sessions. Get a GitHub-style heatmap on your profile and a full analytics dashboard at <strong>vibestats.dev/&lt;username&gt;</strong>.
</p>

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/dashboard-hero-dark.png">
  <img src="docs/images/dashboard-hero-light.png" alt="VibeStats dashboard — full view">
</picture>

<!-- vibestats-start -->
<!-- vibestats-end -->

## Why VibeStats?

Claude Code [clears local session transcripts after 30 days by default](https://code.claude.com/docs/en/data-usage) (configurable via `cleanupPeriodDays` in `~/.claude/settings.json`). Tools that read those JSONL files can only show what's still on disk — your stats silently shrink as old sessions age out.

VibeStats syncs aggregated daily stats — tokens, sessions, minutes, model breakdown, harness mix — to your private `vibestats-data` GitHub repo on every session, *before* that cleanup fires. The sync is non-destructive by design: once a day's stats are uploaded, they stay there indefinitely.

What that gets you:

- **History past 30 days** — months and years of usage stats, without changing Claude Code's defaults.
- **Privacy by default** — Claude Code's transcript cleanup keeps doing its thing; only small JSON aggregates ever leave your machine. No prompt or response content is stored or synced.
- **Survives machine wipes and reinstalls** — your archive lives in your private GitHub repo, not on your laptop.
- **Per-machine breakdown** — every machine writes its own slice; the aggregation step combines them into one heatmap.

## What you get

A live heatmap on your GitHub profile (the embed above), and a full analytics dashboard at `vibestats.dev/<your-username>`:

Weekly model token usage shows the selected period's top six models individually and groups rank seven onward as **Others**. It defaults to the last 12 months, with calendar-year selection available; Weekly Token Consumption and Efficiency share the summary row equally.

<table>
  <tr>
    <td>
      <picture>
        <source media="(prefers-color-scheme: dark)" srcset="docs/images/dashboard-heatmap-dark.png">
        <img src="docs/images/dashboard-heatmap-light.png" alt="Activity heatmap with Sessions / Tokens / Active Time / Tool Uses metric tabs and a range dropdown">
      </picture>
    </td>
    <td>
      <picture>
        <source media="(prefers-color-scheme: dark)" srcset="docs/images/dashboard-kpis-dark.png">
        <img src="docs/images/dashboard-kpis-light.png" alt="KPI tiles: Total Tokens, Sessions, Active Time, Current Streak, Top Harness, Tool Uses">
      </picture>
    </td>
  </tr>
  <tr>
    <td colspan="2">
      <picture>
        <source media="(prefers-color-scheme: dark)" srcset="docs/images/dashboard-summary-charts-dark.png">
        <img src="docs/images/dashboard-summary-charts-light.png" alt="Weekly Token Consumption and Efficiency summary charts in an equal-width row">
      </picture>
    </td>
  </tr>
  <tr>
    <td colspan="2">
      <picture>
        <source media="(prefers-color-scheme: dark)" srcset="docs/images/dashboard-model-usage-dark.png">
        <img src="docs/images/dashboard-model-usage-light.png" alt="Weekly Model Token Usage area chart showing the top six models and Others, with a Last 12 months or calendar-year selector">
      </picture>
    </td>
  </tr>
</table>

## Quickstart

### macOS/Linux: curl installer (recommended)

```bash
curl -fsSL https://vibestats.dev/install.sh | bash
```

The installer handles everything in one step:

- Creates a private `vibestats-data` repo in your GitHub account
- Installs the daily aggregation workflow
- Configures Claude Code and Codex hooks to sync after each session when those tools are installed
- Adds the heatmap to your profile `README.md`
- Runs an initial backfill of existing session data

To sync manually without installing Claude Code or Codex hooks:

```bash
curl -fsSL https://vibestats.dev/install.sh | bash -s -- --skip-hook-configuration
vibestats sync
```

Run `vibestats sync --backfill` whenever you want to rescan all historical sessions.

### Windows: PowerShell installer

On Windows, use PowerShell:

```powershell
irm https://vibestats.dev/install.ps1 | iex
```

The Windows installer performs the same setup as the macOS/Linux installer: it installs the native Windows binary, configures Claude Code and Codex hooks when those tools are present, registers the machine, and runs an initial backfill.

VibeStats uses native Windows paths:

| Data | Windows path |
|---|---|
| Config, checkpoint, and log files | `%LOCALAPPDATA%\vibestats` |
| Binary | `%LOCALAPPDATA%\Programs\vibestats` |

To remove VibeStats later, run:

```powershell
vibestats uninstall
```

### Method 2: cargo install

If you already have a Rust toolchain set up and prefer crates.io:

```bash
# 1. Install the binary from crates.io
cargo install vibestats

# 2. Run the installer with --skip-binary to do the rest of the setup
#    (creates the vibestats-data repo, wires hooks, registers this machine)
curl -fsSL https://vibestats.dev/install.sh | bash -s -- --skip-binary
```

`cargo install` only ships the binary — the per-account setup (private data repo, daily workflow, hook wiring, profile-README markers) still has to run. The `--skip-binary` flag tells the installer to use the `vibestats` already on your `PATH` instead of downloading a release tarball.

### Adding a second machine

On any additional machine, re-run the same install command. The installer detects the existing `vibestats-data` repo and registers the new machine without overwriting anything.

### Claude Code cloud sessions

Sessions on [claude.ai/code](https://claude.ai/code) run in throwaway cloud containers that can't sync themselves: hooks from your user settings don't run there, and their GitHub access is limited to the repositories attached to each session. Instead, one daily [scheduled task](https://code.claude.com/docs/en/routines) records all of them. It lists your cloud sessions with Claude Code's built-in session tools and writes one file per session to your `vibestats-data` repo, under the machine `claude-cloud`. Nothing leaves Anthropic and your GitHub account.

Set it up once, after a normal install has created your `vibestats-data` repo:

1. Make sure the [Claude GitHub App](https://github.com/settings/installations) can access `vibestats-data`. If the scheduled task's repository picker can't find it, add it under the app's **Repository access**.
2. Go to [claude.ai/code/routines](https://claude.ai/code/routines) and create a new scheduled task.
3. **Repository:** select only `<your-username>/vibestats-data`. **Environment:** Default. **Trigger:** daily. **Connectors:** remove them all; the task needs none. Any model works, including the smallest.
4. Paste this prompt, with your username filled in:

   ```text
   Record my Claude Code cloud session usage in vibestats.

   1. Install the vibestats CLI:
      mkdir -p ~/.local/bin && curl -fsSL https://github.com/stephenleo/vibestats/releases/latest/download/vibestats-x86_64-unknown-linux-gnu.tar.gz | tar xz -C ~/.local/bin vibestats
   2. Call the list_sessions tool with limit 100 and mine set to true. While the response has has_more true, call it again with after_id set to that response's last_id. Don't copy the results anywhere: vibestats reads them from this session's transcript.
   3. Run: ~/.local/bin/vibestats ingest-cloud --repo <your-username>/vibestats-data
   4. If the command says the listing is incomplete, finish step 2 and run step 3 again.
   5. Reply with the command's output. Don't change files, branches or pull requests in the repository.
   ```
5. Click **Create**, then **Run now** to check it. The run's last message should end with a line like `3 session(s) uploaded, 0 unchanged, 0 failed`.

Each session is counted on the day it started, and its file is updated on every run for as long as you keep using it. A run only uploads sessions whose totals changed since the last run, tracked in `claude-cloud/index.json`. Cloud sessions report tokens and models only, so they add no active minutes, message counts or tool-use counts. Sessions you drive from your own machine with Remote Control are skipped here, since that machine's own sync already counts them.

## How it works

VibeStats has three components that work together:

1. **CLI** — installed locally, hooks into Claude Code and Codex to record session activity into a private `vibestats-data` GitHub repo after each session.
2. **GitHub Action** (`stephenleo/vibestats@v2`) — runs daily in your `vibestats-data` repo, aggregates the recorded sessions, and pushes an SVG heatmap to your GitHub profile repo.
3. **Profile heatmap** — the SVG is embedded in your profile `README.md` between marker comments, updated automatically.

## CLI reference

```
vibestats <COMMAND>

Commands:
  auth           Authenticate with GitHub
  sync           Sync all supported session data to vibestats-data
  status         Show current sync status and last sync time
  machines       Manage registered machines
  uninstall      Uninstall vibestats
  ingest-cloud   Record Claude Code cloud sessions (run by a scheduled task)
```

## GitHub Action inputs

| Input | Required | Description |
|---|---|---|
| `token` | Yes | Token with Contents write access to `profile-repo`. The installer defaults this to your `gh` OAuth token; opt in during install to supply a narrower fine-grained PAT scoped to just `profile-repo` instead. |
| `profile-repo` | Yes | Your GitHub profile repo in `username/username` format |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, code conventions, and the release process (including how floating major tags work).

## License

[MIT](LICENSE)
