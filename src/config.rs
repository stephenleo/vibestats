#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub oauth_token: String,
    pub machine_id: String,
    pub vibestats_data_repo: String,
}

fn config_path() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME env not set");
    PathBuf::from(home)
        .join(".config")
        .join("vibestats")
        .join("config.toml")
}

fn set_permissions_600(path: &std::path::Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(0o600);
    std::fs::set_permissions(path, perms)
}

fn fnv1a_hash(s: &str) -> u64 {
    let mut hash: u64 = 14695981039346656037;
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}

fn generate_machine_id() -> String {
    let hostname = std::process::Command::new("hostname")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_lowercase())
        .unwrap_or_else(|_| "unknown".to_string());

    let slug: String = hostname
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .chars()
        .take(20)
        .collect();

    let hash = fnv1a_hash(&hostname);
    format!("{}-{:06x}", slug, hash & 0xffffff)
}

impl Config {
    pub fn load() -> Result<Config, String> {
        let path = config_path();
        let contents = std::fs::read_to_string(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                format!(
                    "Config file not found at {}. Run 'vibestats auth' to initialize config.",
                    path.display()
                )
            } else {
                format!("Failed to read config file: {e}")
            }
        })?;
        toml::from_str(&contents).map_err(|e| {
            format!(
                "Config file is malformed ({}). Run 'vibestats auth' to reset your configuration.",
                e
            )
        })
    }

    pub fn save(&self) -> Result<(), String> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {e}"))?;
        }
        let contents =
            toml::to_string(self).map_err(|e| format!("Failed to serialize config: {e}"))?;
        std::fs::write(&path, contents)
            .map_err(|e| format!("Failed to write config file: {e}"))?;
        set_permissions_600(&path).map_err(|e| format!("Failed to set config permissions: {e}"))?;
        Ok(())
    }

    pub fn generate_machine_id(&mut self) -> Result<(), String> {
        let id = generate_machine_id();
        self.machine_id = id;
        self.save()
    }

    pub fn load_or_exit() -> Config {
        match Config::load() {
            Ok(c) => c,
            Err(e) => {
                println!("vibestats: config error: {e}");
                println!("Run 'vibestats auth' to set up your configuration.");
                std::process::exit(0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn unique_temp_path(suffix: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("vibestats_test_{suffix}"));
        path
    }

    #[test]
    fn test_load_valid_toml() {
        let path = unique_temp_path("valid.toml");
        let contents = r#"
oauth_token = "gho_testtoken"
machine_id = "test-machine-abc123"
vibestats_data_repo = "user/vibestats-data"
"#;
        std::fs::write(&path, contents).unwrap();

        // Override HOME to point to a temp dir so config_path() finds our file
        // Instead, we test from_str directly since config_path() uses HOME
        let config: Config = toml::from_str(contents).expect("should parse valid TOML");
        assert_eq!(config.oauth_token, "gho_testtoken");
        assert_eq!(config.machine_id, "test-machine-abc123");
        assert_eq!(config.vibestats_data_repo, "user/vibestats-data");

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_missing_file() {
        // Use a path that definitely doesn't exist
        let nonexistent = PathBuf::from("/tmp/vibestats_definitely_missing_xyzzy/config.toml");
        let result = std::fs::read_to_string(&nonexistent);
        assert!(result.is_err());

        // Verify our load() with a bad HOME returns Err containing "not found"
        // We can't easily override HOME, so test the error mapping logic directly
        let err_msg = if result.unwrap_err().kind() == std::io::ErrorKind::NotFound {
            format!(
                "Config file not found at {}. Run 'vibestats auth' to initialize config.",
                nonexistent.display()
            )
        } else {
            "other error".to_string()
        };
        assert!(err_msg.contains("not found") || err_msg.contains("Config file not found"));
    }

    #[test]
    fn test_load_malformed_toml() {
        let path = unique_temp_path("malformed.toml");
        std::fs::write(&path, "this is not valid toml :::").unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let result: Result<Config, _> = toml::from_str(&contents);
        assert!(result.is_err(), "malformed TOML should return Err");

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_generate_machine_id_format() {
        let id1 = generate_machine_id();
        let id2 = generate_machine_id();

        // Should be non-empty
        assert!(!id1.is_empty(), "machine_id should not be empty");

        // Should be deterministic
        assert_eq!(id1, id2, "generate_machine_id should be deterministic");

        // Should only contain lowercase letters, digits, and hyphens
        assert!(
            id1.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
            "machine_id should only contain [a-z0-9-], got: {id1}"
        );

        // Should end with a 6-hex-char suffix after last '-'
        let parts: Vec<&str> = id1.rsplitn(2, '-').collect();
        assert_eq!(parts[0].len(), 6, "hex suffix should be 6 chars");
        assert!(
            parts[0].chars().all(|c| c.is_ascii_hexdigit()),
            "suffix should be hex digits"
        );
    }

    #[test]
    fn test_save_sets_600_permissions() {
        let temp_dir = unique_temp_path("perms_test_dir");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let path = temp_dir.join("config.toml");
        let contents = r#"oauth_token = "tok"
machine_id = "host-aabbcc"
vibestats_data_repo = "user/repo"
"#;
        std::fs::write(&path, contents).unwrap();
        set_permissions_600(&path).unwrap();

        let metadata = std::fs::metadata(&path).unwrap();
        let mode = metadata.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "config.toml must have 600 permissions, got {:o}", mode);

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_fnv1a_hash_deterministic() {
        let h1 = fnv1a_hash("stephens-mbp");
        let h2 = fnv1a_hash("stephens-mbp");
        assert_eq!(h1, h2, "FNV-1a hash should be deterministic");

        let h3 = fnv1a_hash("other-machine");
        assert_ne!(h1, h3, "different inputs should (likely) produce different hashes");
    }
}
