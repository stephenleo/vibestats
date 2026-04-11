/// Returns the path to ~/.claude/settings.json, or None if HOME is not set.
fn settings_path() -> Option<std::path::PathBuf> {
    std::env::var("HOME").ok().map(|h| {
        std::path::PathBuf::from(h)
            .join(".claude")
            .join("settings.json")
    })
}

/// Remove all hook entries whose "command" field contains "vibestats".
/// Operates on the mutable JSON Value in-place.
/// Preserves all other hooks and top-level settings keys.
fn remove_vibestats_hooks(settings: &mut serde_json::Value) {
    // Get the "hooks" object — if absent, nothing to do
    let Some(hooks_map) = settings.get_mut("hooks").and_then(|h| h.as_object_mut()) else {
        return;
    };

    // For each hook type ("Stop", "SessionStart", etc.)
    for (_event, groups) in hooks_map.iter_mut() {
        let Some(groups_arr) = groups.as_array_mut() else {
            continue;
        };
        // Each group has a "hooks" array of individual hook entries
        for group in groups_arr.iter_mut() {
            let Some(inner_hooks) = group.get_mut("hooks").and_then(|h| h.as_array_mut()) else {
                continue;
            };
            inner_hooks.retain(|hook| {
                // Keep hooks whose "command" does NOT contain "vibestats"
                let cmd = hook.get("command").and_then(|c| c.as_str()).unwrap_or("");
                !cmd.contains("vibestats")
            });
        }
        // Remove groups that now have an empty "hooks" array
        groups_arr.retain(|group| {
            group
                .get("hooks")
                .and_then(|h| h.as_array())
                .map(|arr| !arr.is_empty())
                .unwrap_or(true) // Keep groups without a "hooks" key (unknown format)
        });
    }
}

/// Entry point for `vibestats uninstall`.
///
/// Steps:
/// 1. Remove vibestats hook entries from ~/.claude/settings.json
/// 2. Delete the vibestats binary
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
                println!("vibestats: ~/.claude/settings.json not found — skipping hook removal");
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
                            remove_vibestats_hooks(&mut settings);
                            match serde_json::to_string_pretty(&settings) {
                                Err(e) => {
                                    println!(
                                        "vibestats: could not serialize ~/.claude/settings.json: {e}"
                                    );
                                }
                                Ok(updated) => match std::fs::write(&path, updated) {
                                    Err(e) => {
                                        println!(
                                            "vibestats: could not write ~/.claude/settings.json: {e}"
                                        );
                                    }
                                    Ok(()) => {
                                        println!(
                                            "vibestats: removed hooks from ~/.claude/settings.json"
                                        );
                                    }
                                },
                            }
                        }
                    },
                }
            }
        }
    }

    // Step 2: Delete the vibestats binary
    match std::env::current_exe() {
        Err(e) => {
            println!("vibestats: could not determine binary path: {e}");
            println!("Delete the vibestats binary manually.");
        }
        Ok(exe_path) => match std::fs::remove_file(&exe_path) {
            Ok(()) => println!("vibestats: deleted binary at {}", exe_path.display()),
            Err(e) => {
                println!(
                    "vibestats: could not delete binary at {}: {e}",
                    exe_path.display()
                );
                println!("Delete it manually: rm \"{}\"", exe_path.display());
            }
        },
    }

    // Step 3: Print manual cleanup instructions
    println!();
    println!("vibestats: uninstall complete.");
    println!();
    println!("Optional manual cleanup (not done automatically):");
    println!("  - Delete your vibestats-data repo if you no longer want the data:");
    println!("      gh repo delete <username>/vibestats-data --yes");
    println!(
        "  - Remove the <!-- vibestats-start --> and <!-- vibestats-end --> markers"
    );
    println!("    from your profile README at <username>/<username>/README.md");
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn settings_path_returns_some_when_home_is_set() {
        // HOME must be set in a normal test environment
        if std::env::var("HOME").is_ok() {
            let path = settings_path();
            assert!(path.is_some(), "settings_path must return Some when HOME is set");
            let p = path.unwrap();
            assert!(
                p.to_string_lossy().ends_with(".claude/settings.json"),
                "path must end with .claude/settings.json, got: {}",
                p.display()
            );
        }
    }

    #[test]
    fn settings_path_returns_none_when_home_unset() {
        // Temporarily remove HOME
        let saved = std::env::var("HOME").ok();
        std::env::remove_var("HOME");

        let path = settings_path();
        assert!(path.is_none(), "settings_path must return None when HOME is unset");

        // Restore HOME
        if let Some(h) = saved {
            std::env::set_var("HOME", h);
        }
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

        remove_vibestats_hooks(&mut settings);

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

        remove_vibestats_hooks(&mut settings);

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
        // Must not panic
        remove_vibestats_hooks(&mut settings);
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

        remove_vibestats_hooks(&mut settings);

        let stop_groups = settings["hooks"]["Stop"].as_array().expect("must be array");
        assert_eq!(
            stop_groups.len(),
            1,
            "empty vibestats group should be removed; only other-tool group remains"
        );
        assert_eq!(stop_groups[0]["hooks"][0]["command"], "other-tool");
    }

    #[test]
    fn hook_filtering_handles_missing_stop_and_session_start_gracefully() {
        // Settings exist but have no Stop or SessionStart keys
        let mut settings = json!({
            "hooks": {
                "PreToolUse": [{
                    "hooks": [{ "type": "command", "command": "some-tool" }]
                }]
            }
        });

        // Must not panic, must preserve unknown hook types
        remove_vibestats_hooks(&mut settings);

        let pre_hooks = &settings["hooks"]["PreToolUse"][0]["hooks"];
        assert_eq!(
            pre_hooks.as_array().unwrap().len(),
            1,
            "unknown hook type must be preserved"
        );
    }
}
