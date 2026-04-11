//! `vibestats status` command — shows registered machines and auth token validity.
//!
//! # Responsibilities
//! - Load config via `Config::load_or_exit()`
//! - Fetch `registry.json` from the vibestats-data repo via `GithubApi::get_file_content`
//! - Print each registered machine (FR32)
//! - Check auth token validity via `GithubApi::get_user` (FR33)
//!
//! # Constraints
//! - NEVER calls `std::process::exit` — `main.rs` handles exit
//! - NEVER logs to `vibestats.log` for user-facing output — stdout only
//! - Auth check always runs regardless of registry fetch outcome

use crate::config::Config;
use crate::github_api::GithubApi;

/// Entry point called from `main.rs` for the `vibestats status` command.
///
/// Prints machine registry and auth token status to stdout.
/// Never calls `std::process::exit`.
pub fn run() {
    let config = Config::load_or_exit();
    let api = GithubApi::new(&config.oauth_token, &config.vibestats_data_repo);

    // ── Registry section (FR32) ──────────────────────────────────────────────
    match api.get_file_content("registry.json") {
        Ok(Some(content)) => {
            let json: serde_json::Value =
                serde_json::from_str(&content).unwrap_or(serde_json::Value::Null);
            let machines = json["machines"].as_array();
            if let Some(machines) = machines {
                for m in machines {
                    let machine_id = m["machine_id"].as_str().unwrap_or("unknown");
                    let hostname = m["hostname"].as_str().unwrap_or("unknown");
                    let status = m["status"].as_str().unwrap_or("unknown");
                    let last_seen = m["last_seen"].as_str().unwrap_or("never");
                    println!(
                        "machine: {}  hostname: {}  status: {}  last_seen: {}",
                        machine_id, hostname, status, last_seen
                    );
                }
            }
            // If machines array is absent or empty, print nothing for the registry section
        }
        Ok(None) => {
            // 404 — registry.json does not exist yet
            println!("No machines registered yet.");
        }
        Err(_) => {
            // Network or server error
            println!("vibestats: failed to fetch registry — check your connection.");
        }
    }

    // ── Auth section (FR33) — always runs regardless of registry outcome ─────
    match api.get_user() {
        Ok(login) => {
            println!("Auth: OK (github.com/{})", login);
        }
        Err(_) => {
            println!("Auth: ERROR — run `vibestats auth` to re-authenticate");
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    // ── Registry JSON parsing logic ───────────────────────────────────────────

    /// Parse a valid registry JSON with 2 machines and verify both entries are found.
    #[test]
    fn test_registry_parse_two_machines() {
        let json_str = r#"{
            "machines": [
                {
                    "machine_id": "mbp-abc123",
                    "hostname": "my-macbook.local",
                    "status": "active",
                    "last_seen": "2026-04-10T14:23:00Z"
                },
                {
                    "machine_id": "desktop-xyz789",
                    "hostname": "desktop.local",
                    "status": "inactive",
                    "last_seen": "2026-03-01T08:00:00Z"
                }
            ]
        }"#;

        let json: serde_json::Value =
            serde_json::from_str(json_str).unwrap_or(serde_json::Value::Null);
        let machines = json["machines"].as_array();
        assert!(machines.is_some(), "machines array should be present");
        let machines = machines.unwrap();
        assert_eq!(machines.len(), 2, "should have 2 machines");

        assert_eq!(machines[0]["machine_id"].as_str().unwrap_or(""), "mbp-abc123");
        assert_eq!(machines[0]["hostname"].as_str().unwrap_or(""), "my-macbook.local");
        assert_eq!(machines[0]["status"].as_str().unwrap_or(""), "active");
        assert_eq!(
            machines[0]["last_seen"].as_str().unwrap_or(""),
            "2026-04-10T14:23:00Z"
        );

        assert_eq!(
            machines[1]["machine_id"].as_str().unwrap_or(""),
            "desktop-xyz789"
        );
        assert_eq!(machines[1]["status"].as_str().unwrap_or(""), "inactive");
    }

    /// Parse registry JSON with an empty machines array — should produce no machine output lines.
    #[test]
    fn test_registry_parse_empty_machines_array() {
        let json_str = r#"{"machines": []}"#;

        let json: serde_json::Value =
            serde_json::from_str(json_str).unwrap_or(serde_json::Value::Null);
        let machines = json["machines"].as_array();
        assert!(machines.is_some(), "machines array should be present");
        assert_eq!(
            machines.unwrap().len(),
            0,
            "empty machines array should yield zero entries"
        );
    }

    /// Parse malformed registry JSON — should not panic; machines array returns None.
    #[test]
    fn test_registry_parse_malformed_json_does_not_panic() {
        let json_str = r#"{ this is not valid json }"#;

        let json: serde_json::Value =
            serde_json::from_str(json_str).unwrap_or(serde_json::Value::Null);
        // unwrap_or(Null) means json is Value::Null — machines array is None
        let machines = json["machines"].as_array();
        assert!(
            machines.is_none(),
            "malformed JSON should yield None for machines array, not panic"
        );
    }

    /// Verify that missing optional fields fall back to default strings (not panic).
    #[test]
    fn test_registry_parse_machine_with_missing_fields() {
        let json_str = r#"{"machines": [{"machine_id": "only-id"}]}"#;

        let json: serde_json::Value =
            serde_json::from_str(json_str).unwrap_or(serde_json::Value::Null);
        let machines = json["machines"].as_array().unwrap();
        let m = &machines[0];

        assert_eq!(m["machine_id"].as_str().unwrap_or("unknown"), "only-id");
        assert_eq!(m["hostname"].as_str().unwrap_or("unknown"), "unknown");
        assert_eq!(m["status"].as_str().unwrap_or("unknown"), "unknown");
        assert_eq!(m["last_seen"].as_str().unwrap_or("never"), "never");
    }
}
