# vibestats

Track your Claude Code session activity and display a GitHub contributions-style heatmap on your profile.

<!-- vibestats heatmap screenshot placeholder -->

<!-- vibestats-start -->
<!-- vibestats-end -->

## How it works

vibestats has three components that work together:

1. **CLI** — installed locally, hooks into Claude Code to record session activity into a private `vibestats-data` GitHub repo after each session.
2. **GitHub Action** (`stephenleo/vibestats@v1`) — runs daily in your `vibestats-data` repo, aggregates the recorded sessions, and pushes an SVG heatmap to your GitHub profile repo.
3. **Profile heatmap** — the SVG is embedded in your profile `README.md` between marker comments, updated automatically.

## Quickstart

```bash
curl -fsSL https://vibestats.dev/install.sh | bash
```

The installer handles everything in one step:

- Creates a private `vibestats-data` repo in your GitHub account
- Installs the daily aggregation workflow
- Configures Claude Code hooks to sync after each session
- Adds the heatmap to your profile `README.md`
- Runs an initial backfill of existing session data

### Adding a second machine

On any additional machine, re-run the same install command. The installer detects the existing `vibestats-data` repo and registers the new machine without overwriting anything.

## CLI reference

```
vibestats <COMMAND>

Commands:
  auth           Authenticate with GitHub
  sync           Sync session data to vibestats-data
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

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, code conventions, and the release process (including how the floating `v1` tag works).

## License

[MIT](LICENSE)
