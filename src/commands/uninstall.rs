/// Returns the path to ~/.claude/settings.json, or None if HOME is not set.
fn settings_path() -> Option<std::path::PathBuf> {
    std::env::var("HOME").ok().map(|h| {
        std::path::PathBuf::from(h)
            .join(".claude")
            .join("settings.json")
    })
}

/// Returns the path to the installed vibestats binary at ~/.local/bin/vibestats,
/// or None if HOME is not set.
fn binary_path() -> Option<std::path::PathBuf> {
    std::env::var("HOME").ok().map(|h| {
        std::path::PathBuf::from(h)
            .join(".local")
            .join("bin")
            .join("vibestats")
    })
}

/// True if a hook object's "command" field identifies a vibestats command.
/// Matches exactly `"vibestats"` or anything starting with `"vibestats "` (with space).
/// Narrow on purpose: avoids false positives like `"my-tool --flag vibestats-backup"`.
fn is_vibestats_hook(hook_obj: &serde_json::Value) -> bool {
    hook_obj
        .get("command")
        .and_then(|c| c.as_str())
        .map(|cmd| cmd == "vibestats" || cmd.starts_with("vibestats "))
        .unwrap_or(false)
}

/// Remove vibestats hook entries from a settings.json JSON Value in place.
/// Preserves all non-vibestats hooks and all other top-level settings keys.
/// Returns `true` if anything was removed, `false` otherwise.
///
/// Scope: only the `Stop` and `SessionStart` hook types — the only ones the vibestats
/// installer writes to. Cleans up empty groups, empty hook-type arrays, and the
/// top-level `hooks` object if it becomes empty.
fn remove_vibestats_hooks(settings: &mut serde_json::Value) -> bool {
    let mut changed = false;

    // First pass: operate only on Stop and SessionStart under the "hooks" object.
    for hook_type in &["Stop", "SessionStart"] {
        let Some(groups_arr) = settings
            .get_mut("hooks")
            .and_then(|h| h.get_mut(*hook_type))
            .and_then(|h| h.as_array_mut())
        else {
            continue;
        };

        // Filter inner "hooks" arrays inside each group.
        for group in groups_arr.iter_mut() {
            let Some(inner_hooks) = group.get_mut("hooks").and_then(|h| h.as_array_mut()) else {
                continue;
            };
            let before = inner_hooks.len();
            inner_hooks.retain(|hook| !is_vibestats_hook(hook));
            if inner_hooks.len() != before {
                changed = true;
            }
        }

        // Drop groups whose "hooks" array is now empty. Preserve groups without a
        // "hooks" key (unknown format — leave untouched).
        let before = groups_arr.len();
        groups_arr.retain(|group| {
            group
                .get("hooks")
                .and_then(|h| h.as_array())
                .map(|arr| !arr.is_empty())
                .unwrap_or(true)
        });
        if groups_arr.len() != before {
            changed = true;
        }
    }

    if !changed {
        return false;
    }

    // Second pass: drop hook-type keys whose value is now an empty array.
    if let Some(hooks_obj) = settings.get_mut("hooks").and_then(|h| h.as_object_mut()) {
        for hook_type in &["Stop", "SessionStart"] {
            let is_empty = hooks_obj
                .get(*hook_type)
                .and_then(|v| v.as_array())
                .map(|a| a.is_empty())
                .unwrap_or(false);
            if is_empty {
                hooks_obj.remove(*hook_type);
            }
        }
    }

    // Third pass: drop the top-level "hooks" key if the object is now empty.
    let hooks_empty = settings
        .get("hooks")
        .and_then(|h| h.as_object())
        .map(|o| o.is_empty())
        .unwrap_or(false);
    if hooks_empty {
        if let Some(root) = settings.as_object_mut() {
            root.remove("hooks");
        }
    }

    true
}

/// Entry point for `vibestats uninstall`.
///
/// Steps:
/// 1. Remove vibestats hook entries from ~/.claude/settings.json
/// 2. Delete the vibestats binary from ~/.local/bin/vibestats
/// 3. Print manual cleanup instructions
///
/// NEVER calls std::process::exit — main.rs handles exit.
pub fn run() {
    // Step 1: Remove vibestats hooks from ~/.claude/settings.json
    match settings_path() {
        None => {
            println!("vibestats: HOME not set — skipping hook removal");
        }
        Some(path) => {
            if !path.exists() {
                println!(
                    "vibestats: no vibestats hooks found in settings.json (already clean)"
                );
            } else {
                match std::fs::read_to_string(&path) {
                    Err(e) => {
                        println!("vibestats: could not read ~/.claude/settings.json: {e}");
                        println!("Hook removal skipped.");
                    }
                    Ok(contents) => match serde_json::from_str::<serde_json::Value>(&contents) {
                        Err(e) => {
                            println!("vibestats: could not parse ~/.claude/settings.json: {e}");
                            println!("Hook removal skipped.");
                        }
                        Ok(mut settings) => {
                            if remove_vibestats_hooks(&mut settings) {
                                match serde_json::to_string_pretty(&settings) {
                                    Err(e) => {
                                        println!(
                                            "vibestats: could not serialize ~/.claude/settings.json: {e}"
                                        );
                                    }
                                    Ok(updated) => {
                                        // Write atomically via a sibling .tmp file then
                                        // rename, so a crash or disk-full mid-write can
                                        // never leave settings.json truncated/corrupt.
                                        let mut tmp_path = path.clone();
                                        let mut tmp_name = path
                                            .file_name()
                                            .unwrap_or_default()
                                            .to_os_string();
                                        tmp_name.push(".tmp");
                                        tmp_path.set_file_name(tmp_name);
                                        let write_result = std::fs::write(&tmp_path, updated + "\n")
                                            .and_then(|()| std::fs::rename(&tmp_path, &path));
                                        match write_result {
                                            Err(e) => {
                                                // Clean up the temp file on failure (best effort).
                                                let _ = std::fs::remove_file(&tmp_path);
                                                println!(
                                                    "vibestats: could not write ~/.claude/settings.json: {e}"
                                                );
                                            }
                                            Ok(()) => {
                                                println!(
                                                    "vibestats: removed Stop and SessionStart hooks from ~/.claude/settings.json"
                                                );
                                            }
                                        }
                                    }
                                }
                            } else {
                                println!(
                                    "vibestats: no vibestats hooks found in settings.json (already clean)"
                                );
                            }
                        }
                    },
                }
            }
        }
    }

    // Step 2: Delete the vibestats binary from ~/.local/bin/vibestats (AC #4).
    // This is the installer's target location; it is deliberately NOT `current_exe()`
    // so a developer running `cargo run -- uninstall` does not nuke their dev build.
    match binary_path() {
        None => {
            println!("vibestats: HOME not set — skipping binary deletion");
        }
        Some(path) => match std::fs::remove_file(&path) {
            Ok(()) => println!("vibestats: deleted binary at {}", path.display()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!(
                    "vibestats: binary not found at {} (already removed?)",
                    path.display()
                );
            }
            Err(e) => {
                println!(
                    "vibestats: could not delete binary at {}: {e}",
                    path.display()
                );
                println!("Delete it manually: rm \"{}\"", path.display());
            }
        },
    }

    // Step 3: Print manual cleanup instructions
    println!();
    println!("vibestats: uninstall complete.");
    println!();
    println!("Optional manual cleanup (not done automatically):");
    println!("  1. Delete your vibestats-data repo if you no longer want the data:");
    println!("       gh repo delete <username>/vibestats-data --yes");
    println!("  2. Remove the heatmap markers from your profile README:");
    println!("       Delete the lines between <!-- vibestats-start --> and <!-- vibestats-end -->");
    println!("  3. Remove vibestats config and logs:");
    println!("       rm -rf ~/.config/vibestats");
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Combined test for settings_path() under varying HOME state.
    /// Mutating env vars is process-global, so both branches are exercised
    /// serially inside one test to avoid races with parallel test execution.
    #[test]
    fn settings_path_reflects_home_state() {
        let saved = std::env::var("HOME").ok();

        // Case 1: HOME set
        std::env::set_var("HOME", "/tmp/vibestats-test-home");
        let path = settings_path().expect("must return Some when HOME is set");
        assert!(
            path.to_string_lossy().ends_with(".claude/settings.json"),
            "path must end with .claude/settings.json, got: {}",
            path.display()
        );

        // Case 2: HOME unset
        std::env::remove_var("HOME");
        assert!(
            settings_path().is_none(),
            "settings_path must return None when HOME is unset"
        );

        // Restore HOME
        if let Some(h) = saved {
            std::env::set_var("HOME", h);
        }
    }

    #[test]
    fn binary_path_builds_local_bin_path() {
        let saved = std::env::var("HOME").ok();
        std::env::set_var("HOME", "/tmp/vibestats-test-home");
        let path = binary_path().expect("must return Some when HOME is set");
        assert!(
            path.to_string_lossy().ends_with(".local/bin/vibestats"),
            "binary path must end with .local/bin/vibestats, got: {}",
            path.display()
        );
        if let Some(h) = saved {
            std::env::set_var("HOME", h);
        } else {
            std::env::remove_var("HOME");
        }
    }

    #[test]
    fn is_vibestats_hook_matches_only_exact_and_prefixed_commands() {
        assert!(is_vibestats_hook(&json!({ "command": "vibestats" })));
        assert!(is_vibestats_hook(&json!({ "command": "vibestats sync" })));
        assert!(is_vibestats_hook(
            &json!({ "command": "vibestats session-start" })
        ));
        // False positives the old `.contains()` implementation would have matched:
        assert!(!is_vibestats_hook(
            &json!({ "command": "my-tool --arg vibestats-fake" })
        ));
        assert!(!is_vibestats_hook(&json!({ "command": "vibestats-killer" })));
        assert!(!is_vibestats_hook(&json!({ "command": "not-vibestats" })));
        // Missing / non-string command:
        assert!(!is_vibestats_hook(&json!({})));
        assert!(!is_vibestats_hook(&json!({ "command": 42 })));
    }

    #[test]
    fn hook_filtering_removes_vibestats_commands() {
        let mut settings = json!({
            "hooks": {
                "Stop": [{
                    "hooks": [
                        { "type": "command", "command": "vibestats sync", "async": true },
                        { "type": "command", "command": "other-tool run" }
                    ]
                }]
            }
        });

        let changed = remove_vibestats_hooks(&mut settings);
        assert!(changed, "must report changed=true when hooks removed");

        let stop_hooks = &settings["hooks"]["Stop"][0]["hooks"];
        let arr = stop_hooks.as_array().expect("must be array");
        assert_eq!(arr.len(), 1, "only non-vibestats hook should remain");
        assert_eq!(arr[0]["command"], "other-tool run");
    }

    #[test]
    fn hook_filtering_preserves_non_vibestats_hooks() {
        let mut settings = json!({
            "hooks": {
                "Stop": [{
                    "hooks": [
                        { "type": "command", "command": "my-tool --do-thing" }
                    ]
                }],
                "SessionStart": [{
                    "hooks": [
                        { "type": "command", "command": "another-tool start" }
                    ]
                }]
            },
            "other_setting": "should remain"
        });

        let changed = remove_vibestats_hooks(&mut settings);
        assert!(
            !changed,
            "must report changed=false when nothing vibestats-related is found"
        );

        // Non-vibestats hooks must be preserved
        let stop_hooks = &settings["hooks"]["Stop"][0]["hooks"];
        assert_eq!(
            stop_hooks.as_array().unwrap().len(),
            1,
            "non-vibestats Stop hook must remain"
        );
        let session_hooks = &settings["hooks"]["SessionStart"][0]["hooks"];
        assert_eq!(
            session_hooks.as_array().unwrap().len(),
            1,
            "non-vibestats SessionStart hook must remain"
        );
        // Other top-level settings preserved
        assert_eq!(settings["other_setting"], "should remain");
    }

    #[test]
    fn hook_filtering_handles_missing_hook_keys_gracefully() {
        // No "hooks" key at all
        let mut settings = json!({ "some_other_key": 42 });
        let changed = remove_vibestats_hooks(&mut settings);
        assert!(!changed);
        assert_eq!(settings["some_other_key"], 42);
    }

    #[test]
    fn hook_filtering_removes_group_when_all_hooks_are_vibestats() {
        let mut settings = json!({
            "hooks": {
                "Stop": [
                    {
                        "hooks": [
                            { "type": "command", "command": "vibestats sync", "async": true }
                        ]
                    },
                    {
                        "hooks": [
                            { "type": "command", "command": "other-tool" }
                        ]
                    }
                ]
            }
        });

        let changed = remove_vibestats_hooks(&mut settings);
        assert!(changed);

        let stop_groups = settings["hooks"]["Stop"].as_array().expect("must be array");
        assert_eq!(
            stop_groups.len(),
            1,
            "empty vibestats group should be removed; only other-tool group remains"
        );
        assert_eq!(stop_groups[0]["hooks"][0]["command"], "other-tool");
    }

    #[test]
    fn hook_filtering_preserves_unknown_hook_types() {
        // Settings has PreToolUse which vibestats never writes — must be untouched.
        let mut settings = json!({
            "hooks": {
                "PreToolUse": [{
                    "hooks": [{ "type": "command", "command": "some-tool" }]
                }]
            }
        });

        let changed = remove_vibestats_hooks(&mut settings);
        assert!(!changed);

        let pre_hooks = &settings["hooks"]["PreToolUse"][0]["hooks"];
        assert_eq!(
            pre_hooks.as_array().unwrap().len(),
            1,
            "unknown hook type must be preserved"
        );
    }

    #[test]
    fn hook_filtering_strips_empty_hook_type_and_hooks_key_when_only_vibestats_present() {
        let mut settings = json!({
            "hooks": {
                "Stop": [{
                    "hooks": [
                        { "type": "command", "command": "vibestats sync", "async": true }
                    ]
                }],
                "SessionStart": [{
                    "hooks": [
                        { "type": "command", "command": "vibestats session-start" }
                    ]
                }]
            },
            "model": "sonnet"
        });

        let changed = remove_vibestats_hooks(&mut settings);
        assert!(changed, "must report changed=true");

        // hooks object must be gone entirely (it was only vibestats).
        assert!(
            settings.get("hooks").is_none(),
            "top-level hooks key must be removed when empty"
        );
        // Unrelated top-level settings must be preserved.
        assert_eq!(settings["model"], "sonnet");
    }

    #[test]
    fn hook_filtering_does_not_touch_non_stop_sessionstart_types_even_if_vibestats_command() {
        // Spec scope: removal is limited to Stop/SessionStart. A vibestats command
        // placed under an unexpected hook type must be preserved (out of scope).
        let mut settings = json!({
            "hooks": {
                "PreToolUse": [{
                    "hooks": [
                        { "type": "command", "command": "vibestats weird-hook" }
                    ]
                }]
            }
        });

        let changed = remove_vibestats_hooks(&mut settings);
        assert!(
            !changed,
            "must not touch hook types outside Stop/SessionStart"
        );
        assert_eq!(
            settings["hooks"]["PreToolUse"][0]["hooks"][0]["command"],
            "vibestats weird-hook"
        );
    }
}
