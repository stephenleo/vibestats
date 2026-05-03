# vibestats

Track your Claude Code and Codex session activity and display a GitHub contributions-style heatmap on your profile.

<!-- vibestats heatmap screenshot placeholder -->

<!-- vibestats-start -->
<!-- vibestats-end -->

## How it works

vibestats has three components that work together:

1. **CLI** — installed locally, hooks into Claude Code and Codex to record session activity into a private `vibestats-data` GitHub repo after each session.
2. **GitHub Action** (`stephenleo/vibestats@v2`) — runs daily in your `vibestats-data` repo, aggregates the recorded sessions, and pushes an SVG heatmap to your GitHub profile repo.
3. **Profile heatmap** — the SVG is embedded in your profile `README.md` between marker comments, updated automatically.

## Long-term retention

Claude Code [clears local session transcripts after 30 days by default](https://code.claude.com/docs/en/data-usage) (configurable via `cleanupPeriodDays` in `~/.claude/settings.json`). vibestats syncs aggregated daily stats — tokens, sessions, minutes, model breakdown — to your private `vibestats-data` GitHub repo on every session, *before* that cleanup fires. The sync is non-destructive by design: once a day's stats are uploaded, they stay there indefinitely.

What that gets you:

- **History past 30 days** — months and years of usage stats, without changing Claude Code's defaults.
- **Privacy by default** — Claude Code's transcript cleanup keeps doing its thing; only small JSON aggregates ever leave your machine. No prompt or response content is stored or synced.
- **Survives machine wipes and reinstalls** — your archive lives in your private GitHub repo, not on your laptop.
- **Per-machine breakdown** — every machine writes its own slice; the aggregation step combines them into one heatmap.

## Quickstart

```bash
curl -fsSL https://vibestats.dev/install.sh | bash
```

The installer handles everything in one step:

- Creates a private `vibestats-data` repo in your GitHub account
- Installs the daily aggregation workflow
- Configures Claude Code and Codex hooks to sync after each session when those tools are installed
- Adds the heatmap to your profile `README.md`
- Runs an initial backfill of existing session data

### Adding a second machine

On any additional machine, re-run the same install command. The installer detects the existing `vibestats-data` repo and registers the new machine without overwriting anything.

## CLI reference

```
vibestats <COMMAND>

Commands:
  auth           Authenticate with GitHub
  sync           Sync all supported session data to vibestats-data
  status         Show current sync status and last sync time
  machines       Manage registered machines
  uninstall      Uninstall vibestats
```

## GitHub Action inputs

| Input | Required | Description |
|---|---|---|
| `token` | Yes | Fine-grained PAT with Contents write access to `profile-repo` |
| `profile-repo` | Yes | Your GitHub profile repo in `username/username` format |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, code conventions, and the release process (including how floating major tags work).

## License

[MIT](LICENSE)
