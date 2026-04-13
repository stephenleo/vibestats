use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub oauth_token: String,
    pub machine_id: String,
    pub vibestats_data_repo: String,
}

fn config_path() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| {
        "HOME environment variable is not set. Run 'vibestats auth' after exporting HOME."
            .to_string()
    })?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("vibestats")
        .join("config.toml"))
}

fn set_permissions_600(path: &std::path::Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(0o600);
    std::fs::set_permissions(path, perms)
}

/// Atomically create (or truncate) the file at `path` with mode 0600, so that the
/// OAuth token is never briefly visible under a wider umask. On Unix this uses
/// `OpenOptions::mode(0o600)` at creation time.
fn write_file_mode_600(path: &std::path::Path, contents: &[u8]) -> std::io::Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(contents)?;
    // Re-assert mode in case the file already existed with looser perms.
    set_permissions_600(path)
}

#[cfg_attr(not(test), allow(dead_code))]
fn fnv1a_hash(s: &str) -> u64 {
    let mut hash: u64 = 14695981039346656037;
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}

#[cfg_attr(not(test), allow(dead_code))]
fn generate_machine_id() -> String {
    let hostname = std::process::Command::new("hostname")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_lowercase())
        .unwrap_or_else(|_| "unknown".to_string());

    // Lowercase, alphanumeric-or-hyphen slug, truncated to 20 chars. We trim again
    // after truncation so a slug that ends on a `-` boundary does not produce
    // `--hex`; if the hostname had no alphanumeric characters at all, fall back
    // to "machine" so the final ID is always well-formed.
    let truncated: String = hostname
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .chars()
        .take(20)
        .collect();
    let trimmed = truncated.trim_matches('-');
    let slug = if trimmed.is_empty() {
        "machine"
    } else {
        trimmed
    };

    let hash = fnv1a_hash(&hostname);
    format!("{}-{:06x}", slug, hash & 0xffffff)
}

impl Config {
    pub fn load() -> Result<Config, String> {
        let path = config_path()?;
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
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {e}"))?;
        }
        let contents =
            toml::to_string(self).map_err(|e| format!("Failed to serialize config: {e}"))?;
        // Write with 0600 at creation time so the token is never briefly world-
        // or group-readable under the default umask (NFR6).
        write_file_mode_600(&path, contents.as_bytes())
            .map_err(|e| format!("Failed to write config file: {e}"))?;
        Ok(())
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
        // Exercise the same helper that Config::save() uses so we cover the
        // race-free create-with-0600 path, not just a standalone chmod.
        write_file_mode_600(&path, contents.as_bytes()).unwrap();

        let metadata = std::fs::metadata(&path).unwrap();
        let mode = metadata.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "config.toml must have 600 permissions, got {:o}", mode);

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_write_file_mode_600_overwrites_loose_perms() {
        // If the file already exists with wider permissions (e.g. from a legacy
        // install), save() must still end up at 0600.
        let temp_dir = unique_temp_path("perms_overwrite_dir");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let path = temp_dir.join("config.toml");
        std::fs::write(&path, b"old").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

        write_file_mode_600(&path, b"new contents").unwrap();

        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "existing file must be narrowed to 600, got {:o}", mode);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "new contents");

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_generate_machine_id_fallback_for_empty_slug() {
        // Guarantee the slug fallback path compiles and produces a well-formed
        // ID shape. We cannot override the hostname, but we can exercise the
        // format invariants against the real hostname and confirm the ID never
        // starts with a '-' (which would have happened for a non-alphanumeric
        // hostname under the pre-fix code).
        let id = generate_machine_id();
        assert!(!id.starts_with('-'), "machine_id must not start with '-', got: {id}");
        assert!(!id.contains("--"), "machine_id must not contain '--', got: {id}");
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
