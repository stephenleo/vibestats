/// Returns the path to `~/.config/vibestats/checkpoint.toml`, or `None` if `HOME` is not set.
fn checkpoint_path() -> Option<std::path::PathBuf> {
    std::env::var("HOME").ok().map(|h| {
        std::path::PathBuf::from(h)
            .join(".config")
            .join("vibestats")
            .join("checkpoint.toml")
    })
}

/// Entry point called from `main.rs` for the `vibestats auth` command.
///
/// Orchestrates: gh auth token → config.save() → gh secret set → checkpoint.clear_auth_error()
///
/// NEVER calls `std::process::exit` — `main.rs` handles exit.
pub fn run() {
    // Step 1 — obtain fresh token via `gh auth token`
    let new_token = match std::process::Command::new("gh")
        .args(["auth", "token"])
        .output()
    {
        Err(e) => {
            println!("vibestats: auth failed — could not run 'gh': {e}");
            println!("Ensure 'gh' CLI is installed and accessible in PATH.");
            return;
        }
        Ok(out) if !out.status.success() => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            println!(
                "vibestats: auth failed — 'gh auth token' returned non-zero: {}",
                stderr.trim()
            );
            println!("Run 'gh auth login' first, then retry 'vibestats auth'.");
            return;
        }
        Ok(out) => {
            let token = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if token.is_empty() {
                println!("vibestats: auth failed — 'gh auth token' returned empty token.");
                println!("Run 'gh auth login' first, then retry 'vibestats auth'.");
                return;
            }
            token
        }
    };

    // Step 2 — update config.toml with new token (Config::save enforces 600 perms)
    let mut config = match crate::config::Config::load() {
        Ok(c) => c,
        Err(e) => {
            println!("vibestats: auth failed — could not load config: {e}");
            return;
        }
    };
    config.oauth_token = new_token;
    if let Err(e) = config.save() {
        println!("vibestats: auth failed — could not save config: {e}");
        return;
    }

    // Step 3 — update VIBESTATS_TOKEN Actions secret (non-fatal if it fails).
    //
    // SECURITY: we pass the token via stdin using `--body-file -` rather than
    // `--body <token>` so the token never appears in this process's argv (and
    // therefore never shows up in `ps` / `/proc/<pid>/cmdline`). NFR6.
    let secret_result = (|| -> std::io::Result<std::process::Output> {
        use std::io::Write;
        let mut child = std::process::Command::new("gh")
            .args([
                "secret",
                "set",
                "VIBESTATS_TOKEN",
                "--repo",
                &config.vibestats_data_repo,
                "--body-file",
                "-",
            ])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;
        // stdin is piped — convert None to io::Error so the closure can use ?
        // without resorting to expect() or unwrap() in non-test code.
        child
            .stdin
            .as_mut()
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::BrokenPipe, "stdin not piped")
            })?
            .write_all(config.oauth_token.as_bytes())?;
        // Drop stdin to close the pipe and signal EOF to the child process.
        // Without this, `gh secret set --body-file -` would block waiting for
        // more input, and wait_with_output() would deadlock.
        drop(child.stdin.take());
        child.wait_with_output()
    })();

    match secret_result {
        Err(e) => {
            println!(
                "vibestats: token saved locally but could not update VIBESTATS_TOKEN secret: {e}"
            );
            println!("Run manually (feeds token via stdin to avoid leaking it in argv):");
            println!(
                "  gh auth token | gh secret set VIBESTATS_TOKEN --repo {} --body-file -",
                config.vibestats_data_repo
            );
            // Continue — local token is updated; don't abort checkpoint clear
        }
        Ok(out) if !out.status.success() => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            println!("vibestats: token saved locally but 'gh secret set' failed: {stderr}");
            println!("Run manually (feeds token via stdin to avoid leaking it in argv):");
            println!(
                "  gh auth token | gh secret set VIBESTATS_TOKEN --repo {} --body-file -",
                config.vibestats_data_repo
            );
            // Continue — local token is updated
        }
        Ok(_) => {} // success
    }

    // Step 4 — clear auth_error in checkpoint.toml (non-fatal if it fails)
    if let Some(cp_path) = checkpoint_path() {
        let mut cp = crate::checkpoint::Checkpoint::load(&cp_path);
        cp.clear_auth_error();
        if let Err(e) = cp.save(&cp_path) {
            // Non-fatal — auth is refreshed; log but do not abort
            println!("vibestats: auth complete (note: could not clear auth_error flag: {e})");
            return;
        }
    }
    println!("vibestats: auth complete");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkpoint_path_returns_some_when_home_is_set() {
        // HOME should be set in a normal test environment
        let result = checkpoint_path();
        assert!(result.is_some(), "checkpoint_path() should return Some when HOME is set");
        let path = result.unwrap();
        let path_str = path.to_string_lossy();
        assert!(
            path_str.ends_with(".config/vibestats/checkpoint.toml"),
            "path should end with .config/vibestats/checkpoint.toml, got: {path_str}"
        );
    }

    #[test]
    fn checkpoint_path_returns_none_when_home_unset() {
        // Temporarily unset HOME, capture result, then restore
        let original_home = std::env::var("HOME").ok();

        // SAFETY: modifying environment in tests; this test must run in isolation
        // (single-threaded or with env var manipulation not racing)
        unsafe {
            std::env::remove_var("HOME");
        }
        let result = checkpoint_path();

        // Restore HOME
        if let Some(home) = original_home {
            unsafe {
                std::env::set_var("HOME", home);
            }
        }

        assert!(result.is_none(), "checkpoint_path() should return None when HOME is unset");
    }

    #[test]
    fn nonexistent_gh_binary_causes_error() {
        // Verify that invoking a non-existent binary path produces an Err from .output()
        let result = std::process::Command::new("/nonexistent/gh")
            .args(["auth", "token"])
            .output();
        assert!(result.is_err(), "Expected Err for non-existent binary, got Ok");
    }
}
