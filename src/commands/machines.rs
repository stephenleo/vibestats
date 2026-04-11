//! machines command handler for vibestats.
//!
//! Implements `vibestats machines list` and `vibestats machines remove`.
//!
//! # Architecture constraints
//! - NEVER calls `std::process::exit` (NFR10)
//! - All GitHub HTTP calls go through `github_api.rs` — no inline HTTP
//! - Errors logged via `logger::error` only (NFR11)
//! - No async runtime — all code synchronous

use crate::checkpoint::Checkpoint;
use crate::config::Config;
use crate::github_api::GithubApi;
use crate::logger;
use std::path::PathBuf;

/// Returns the path to the checkpoint file, or None if HOME is not set.
/// Defined privately here — mirrors the pattern in `session_start.rs`.
fn checkpoint_path() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(|home| {
        PathBuf::from(home)
            .join(".config")
            .join("vibestats")
            .join("checkpoint.toml")
    })
}

/// List all registered machines from `registry.json`.
///
/// Prints one machine per line: `machine_id  hostname  status  last_seen`.
/// Never calls `std::process::exit`.
pub fn list() {
    let config = Config::load_or_exit();
    let api = GithubApi::new(&config.oauth_token, &config.vibestats_data_repo);

    match api.get_file_content("registry.json") {
        Ok(None) => {
            println!("vibestats: no machines registered");
        }
        Ok(Some(content)) => {
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(json) => {
                    match json["machines"].as_array() {
                        None => {
                            println!("vibestats: registry.json is malformed");
                        }
                        Some(machines) => {
                            if machines.is_empty() {
                                println!("vibestats: no machines registered");
                                return;
                            }
                            for m in machines {
                                let machine_id = m["machine_id"].as_str().unwrap_or("");
                                let hostname = m["hostname"].as_str().unwrap_or("");
                                let status = m["status"].as_str().unwrap_or("");
                                let last_seen = m["last_seen"].as_str().unwrap_or("");
                                println!(
                                    "{}  {}  {}  {}",
                                    machine_id, hostname, status, last_seen
                                );
                            }
                        }
                    }
                }
                Err(_) => {
                    println!("vibestats: registry.json is malformed");
                }
            }
        }
        Err(e) => {
            logger::error(&format!("machines: failed to fetch registry.json: {}", e));
            println!("vibestats: failed to fetch registry — check vibestats.log");
        }
    }
}

/// Remove a machine from `registry.json`.
///
/// Default: sets `status = "retired"` (preserves all historical data).
/// With `purge_history = true`: prompts for confirmation, then sets `status = "purged"`
/// and bulk-deletes all Hive partition files for that machine.
///
/// If the machine being removed is the current machine (self-retire/self-purge),
/// also updates local `checkpoint.toml`.
///
/// Never calls `std::process::exit`.
pub fn remove(machine_id: &str, purge_history: bool) {
    let config = Config::load_or_exit();
    let api = GithubApi::new(&config.oauth_token, &config.vibestats_data_repo);

    // Fetch registry
    let content = match api.get_file_content("registry.json") {
        Ok(None) => {
            println!("vibestats: no machines registered");
            return;
        }
        Ok(Some(c)) => c,
        Err(e) => {
            logger::error(&format!("machines: failed to fetch registry.json: {}", e));
            println!("vibestats: failed to fetch registry — check vibestats.log");
            return;
        }
    };

    // Parse registry JSON
    let mut json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            logger::error(&format!("machines: registry.json is malformed: {}", e));
            println!("vibestats: registry.json is malformed");
            return;
        }
    };

    // Find the machine
    let machines = match json["machines"].as_array() {
        Some(m) => m.clone(),
        None => {
            println!("vibestats: registry.json is malformed");
            return;
        }
    };

    let machine_index = machines
        .iter()
        .position(|m| m["machine_id"].as_str() == Some(machine_id));

    let idx = match machine_index {
        Some(i) => i,
        None => {
            println!("vibestats: machine '{}' not found in registry", machine_id);
            return;
        }
    };

    // Extract hostname for confirmation message
    let hostname = machines[idx]["hostname"]
        .as_str()
        .unwrap_or(machine_id)
        .to_string();

    if !purge_history {
        // Default retire path: set status = "retired"
        json["machines"][idx]["status"] = serde_json::Value::String("retired".to_string());
        let updated_json = match serde_json::to_string_pretty(&json) {
            Ok(s) => s,
            Err(e) => {
                logger::error(&format!("machines: failed to serialize registry: {}", e));
                println!("vibestats: failed to update registry — check vibestats.log");
                return;
            }
        };
        if let Err(e) = api.put_file("registry.json", &updated_json) {
            logger::error(&format!("machines: failed to update registry.json: {}", e));
            println!("vibestats: failed to update registry — check vibestats.log");
            return;
        }
        // Self-retire: also update local checkpoint
        if machine_id == config.machine_id {
            update_local_checkpoint("retired");
        }
        println!("vibestats: machine '{}' retired", machine_id);
    } else {
        // Purge path: prompt for confirmation
        print!(
            "This will permanently remove all historical data for {}. Continue? (y/N): ",
            hostname
        );
        // Flush stdout so prompt appears before blocking on stdin
        use std::io::Write;
        if let Err(e) = std::io::stdout().flush() {
            logger::error(&format!("machines: failed to flush stdout: {}", e));
        }

        let mut input = String::new();
        let confirmed = match std::io::stdin().read_line(&mut input) {
            Ok(_) => {
                let trimmed = input.trim();
                trimmed == "y" || trimmed == "Y"
            }
            Err(e) => {
                logger::error(&format!("machines: failed to read stdin: {}", e));
                false // treat as "N" — abort safely
            }
        };

        if !confirmed {
            println!("vibestats: purge cancelled");
            return;
        }

        // Set status = "purged"
        json["machines"][idx]["status"] = serde_json::Value::String("purged".to_string());
        let updated_json = match serde_json::to_string_pretty(&json) {
            Ok(s) => s,
            Err(e) => {
                logger::error(&format!("machines: failed to serialize registry: {}", e));
                println!("vibestats: failed to update registry — check vibestats.log");
                return;
            }
        };
        if let Err(e) = api.put_file("registry.json", &updated_json) {
            logger::error(&format!("machines: failed to update registry.json: {}", e));
            println!("vibestats: failed to update registry — check vibestats.log");
            return;
        }

        // Enumerate and delete Hive partition files
        let deleted_count = if machine_id == config.machine_id {
            // Self-purge: use local checkpoint date hashes (deterministic, low network cost)
            purge_self(&api, machine_id)
        } else {
            // Remote purge: list directory tree via GitHub Contents API
            purge_remote(&api, machine_id)
        };

        // Self-purge: update local checkpoint
        if machine_id == config.machine_id {
            update_local_checkpoint("purged");
        }

        println!(
            "vibestats: machine '{}' purged — {} file(s) deleted",
            machine_id, deleted_count
        );
    }
}

/// Update local `checkpoint.toml` machine_status to `status`.
/// Logs errors but does not surface them to stdout.
fn update_local_checkpoint(status: &str) {
    if let Some(ref path) = checkpoint_path() {
        let mut cp = Checkpoint::load(path);
        cp.set_machine_status(status);
        if let Err(e) = cp.save(path) {
            logger::error(&format!("machines: failed to save checkpoint: {}", e));
        }
    }
}

/// Purge Hive files for the current machine using checkpoint date hashes.
/// Returns the number of files successfully deleted.
fn purge_self(api: &GithubApi, machine_id: &str) -> usize {
    let mut deleted = 0usize;
    let cp_path = checkpoint_path();
    let checkpoint = cp_path
        .as_deref()
        .map(Checkpoint::load)
        .unwrap_or_default();

    for date in checkpoint.date_hashes.keys() {
        let parts: Vec<&str> = date.split('-').collect();
        if parts.len() == 3 {
            let hive_path = format!(
                "machines/year={}/month={}/day={}/harness=claude/machine_id={}/data.json",
                parts[0], parts[1], parts[2], machine_id
            );
            match api.delete_file(&hive_path) {
                Ok(()) => {
                    deleted += 1;
                }
                Err(e) => {
                    // Log errors but continue — best-effort cleanup
                    logger::error(&format!(
                        "machines: failed to delete {}: {}",
                        hive_path, e
                    ));
                }
            }
        }
    }
    deleted
}

/// Purge Hive files for a remote machine by listing the GitHub directory tree.
/// Walks the Hive path structure depth-first:
///   machines/ → year=YYYY/ → month=MM/ → day=DD/ → harness=claude/machine_id=<id>/ → data.json
/// Returns the number of files successfully deleted.
fn purge_remote(api: &GithubApi, machine_id: &str) -> usize {
    let mut deleted = 0usize;
    let base_path = "machines";

    // List year directories
    let (_, year_dirs) = match api.list_directory_all(base_path) {
        Ok(entries) => entries,
        Err(e) => {
            logger::error(&format!(
                "machines: failed to list directory {}: {}",
                base_path, e
            ));
            return deleted;
        }
    };

    for year_dir in &year_dirs {
        // List month directories within each year
        let (_, month_dirs) = match api.list_directory_all(year_dir) {
            Ok(entries) => entries,
            Err(e) => {
                logger::error(&format!(
                    "machines: failed to list directory {}: {}",
                    year_dir, e
                ));
                continue;
            }
        };

        for month_dir in &month_dirs {
            // List day directories within each month
            let (_, day_dirs) = match api.list_directory_all(month_dir) {
                Ok(entries) => entries,
                Err(e) => {
                    logger::error(&format!(
                        "machines: failed to list directory {}: {}",
                        month_dir, e
                    ));
                    continue;
                }
            };

            for day_dir in &day_dirs {
                // List harness directories
                let (_, harness_dirs) = match api.list_directory_all(day_dir) {
                    Ok(entries) => entries,
                    Err(e) => {
                        logger::error(&format!(
                            "machines: failed to list directory {}: {}",
                            day_dir, e
                        ));
                        continue;
                    }
                };

                for harness_dir in &harness_dirs {
                    // harness_dir is like "machines/year=2026/month=04/day=10/harness=claude"
                    // List machine_id directories within harness
                    let (_, machine_dirs) = match api.list_directory_all(harness_dir) {
                        Ok(entries) => entries,
                        Err(e) => {
                            logger::error(&format!(
                                "machines: failed to list directory {}: {}",
                                harness_dir, e
                            ));
                            continue;
                        }
                    };

                    for machine_dir in &machine_dirs {
                        // Filter to only this machine's partition
                        let target_segment = format!("machine_id={}", machine_id);
                        if !machine_dir.contains(&target_segment) {
                            continue;
                        }
                        // List data files within this machine's partition
                        let (data_files, _) = match api.list_directory_all(machine_dir) {
                            Ok(entries) => entries,
                            Err(e) => {
                                logger::error(&format!(
                                    "machines: failed to list directory {}: {}",
                                    machine_dir, e
                                ));
                                continue;
                            }
                        };
                        for file_path in &data_files {
                            match api.delete_file(file_path) {
                                Ok(()) => {
                                    deleted += 1;
                                }
                                Err(e) => {
                                    logger::error(&format!(
                                        "machines: failed to delete {}: {}",
                                        file_path, e
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    deleted
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    // ── Registry JSON parsing ──────────────────────────────────────────────────

    #[test]
    fn test_registry_parse_extracts_machine_fields() {
        let registry_json = r#"{
            "machines": [
                {
                    "machine_id": "stephens-mbp-a1b2c3",
                    "hostname": "Stephens-MacBook-Pro.local",
                    "status": "active",
                    "last_seen": "2026-04-10T14:23:00Z"
                }
            ]
        }"#;
        let json: serde_json::Value = serde_json::from_str(registry_json).unwrap();
        let machines = json["machines"].as_array().unwrap();
        assert_eq!(machines.len(), 1);
        let m = &machines[0];
        assert_eq!(m["machine_id"].as_str().unwrap(), "stephens-mbp-a1b2c3");
        assert_eq!(m["hostname"].as_str().unwrap(), "Stephens-MacBook-Pro.local");
        assert_eq!(m["status"].as_str().unwrap(), "active");
        assert_eq!(m["last_seen"].as_str().unwrap(), "2026-04-10T14:23:00Z");
    }

    #[test]
    fn test_registry_parse_multiple_machines() {
        let registry_json = r#"{
            "machines": [
                {
                    "machine_id": "mbp-a1b2c3",
                    "hostname": "MacBook-Pro.local",
                    "status": "active",
                    "last_seen": "2026-04-10T14:23:00Z"
                },
                {
                    "machine_id": "ubuntu-d4e5f6",
                    "hostname": "work-ubuntu",
                    "status": "retired",
                    "last_seen": "2026-03-15T09:10:00Z"
                }
            ]
        }"#;
        let json: serde_json::Value = serde_json::from_str(registry_json).unwrap();
        let machines = json["machines"].as_array().unwrap();
        assert_eq!(machines.len(), 2);
        assert_eq!(machines[1]["status"].as_str().unwrap(), "retired");
    }

    // ── Retire mutation ────────────────────────────────────────────────────────

    #[test]
    fn test_retire_mutation_sets_status_to_retired() {
        let registry_json = r#"{
            "machines": [
                {
                    "machine_id": "mbp-a1b2c3",
                    "hostname": "MacBook-Pro.local",
                    "status": "active",
                    "last_seen": "2026-04-10T14:23:00Z"
                }
            ]
        }"#;
        let mut json: serde_json::Value = serde_json::from_str(registry_json).unwrap();
        json["machines"][0]["status"] = serde_json::Value::String("retired".to_string());

        let updated: serde_json::Value =
            serde_json::from_str(&serde_json::to_string_pretty(&json).unwrap()).unwrap();
        assert_eq!(updated["machines"][0]["status"].as_str().unwrap(), "retired");
        // Other fields must be preserved
        assert_eq!(
            updated["machines"][0]["machine_id"].as_str().unwrap(),
            "mbp-a1b2c3"
        );
        assert_eq!(
            updated["machines"][0]["hostname"].as_str().unwrap(),
            "MacBook-Pro.local"
        );
    }

    // ── Machine-not-found path ─────────────────────────────────────────────────

    #[test]
    fn test_machine_not_found_returns_none() {
        let registry_json = r#"{
            "machines": [
                {
                    "machine_id": "mbp-a1b2c3",
                    "hostname": "MacBook-Pro.local",
                    "status": "active",
                    "last_seen": "2026-04-10T14:23:00Z"
                }
            ]
        }"#;
        let json: serde_json::Value = serde_json::from_str(registry_json).unwrap();
        let machines = json["machines"].as_array().unwrap();
        let found = machines
            .iter()
            .position(|m| m["machine_id"].as_str() == Some("nonexistent-machine-id"));
        assert!(found.is_none(), "machine_id not in registry must return None");
    }

    // ── stdin confirmation acceptance ──────────────────────────────────────────

    #[test]
    fn test_confirm_accepts_lowercase_y() {
        assert!(is_confirmed("y"), "'y' must be accepted as confirmation");
    }

    #[test]
    fn test_confirm_accepts_uppercase_y() {
        assert!(is_confirmed("Y"), "'Y' must be accepted as confirmation");
    }

    #[test]
    fn test_confirm_rejects_n() {
        assert!(!is_confirmed("n"), "'n' must not be accepted");
    }

    #[test]
    fn test_confirm_rejects_empty() {
        assert!(!is_confirmed(""), "empty input must not be accepted (default is N)");
    }

    #[test]
    fn test_confirm_rejects_yes() {
        assert!(!is_confirmed("yes"), "'yes' must not be accepted (only 'y' or 'Y')");
    }

    #[test]
    fn test_confirm_rejects_whitespace() {
        assert!(!is_confirmed(" "), "whitespace must not be accepted");
    }

    /// Helper that mirrors the confirmation logic in `remove()`.
    fn is_confirmed(input: &str) -> bool {
        let trimmed = input.trim();
        trimmed == "y" || trimmed == "Y"
    }

    // ── Hive path construction ─────────────────────────────────────────────────

    #[test]
    fn test_hive_path_zero_padded() {
        let date = "2026-04-09";
        let machine_id = "mbp-a1b2c3";
        let parts: Vec<&str> = date.split('-').collect();
        assert_eq!(parts.len(), 3);
        let hive_path = format!(
            "machines/year={}/month={}/day={}/harness=claude/machine_id={}/data.json",
            parts[0], parts[1], parts[2], machine_id
        );
        assert_eq!(
            hive_path,
            "machines/year=2026/month=04/day=09/harness=claude/machine_id=mbp-a1b2c3/data.json"
        );
    }

    #[test]
    fn test_hive_path_double_digit_day() {
        let date = "2026-12-31";
        let machine_id = "test-machine";
        let parts: Vec<&str> = date.split('-').collect();
        let hive_path = format!(
            "machines/year={}/month={}/day={}/harness=claude/machine_id={}/data.json",
            parts[0], parts[1], parts[2], machine_id
        );
        assert_eq!(
            hive_path,
            "machines/year=2026/month=12/day=31/harness=claude/machine_id=test-machine/data.json"
        );
    }
}
