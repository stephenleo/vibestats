//! Platform-aware path resolution for vibestats data files.
//!
//! Single source of truth for the checkpoint location — command modules and
//! hooks must use these helpers instead of building paths locally.

use std::path::PathBuf;

/// Returns the path to the vibestats checkpoint file for the current platform.
///
/// Unix: `$HOME/.config/vibestats/checkpoint.toml`
/// Windows: `%LOCALAPPDATA%\\vibestats\\checkpoint.toml`
///
/// LOCALAPPDATA (not APPDATA) is deliberate: the checkpoint holds
/// machine-specific sync state, and roaming profiles sync APPDATA between
/// machines, which would corrupt per-machine sync tracking.
pub fn checkpoint_path() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var("LOCALAPPDATA")
            .ok()
            .map(|h| PathBuf::from(h).join("vibestats").join("checkpoint.toml"))
    }

    #[cfg(not(windows))]
    {
        std::env::var("HOME").ok().map(|h| {
            PathBuf::from(h)
                .join(".config")
                .join("vibestats")
                .join("checkpoint.toml")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkpoint_path_returns_some_when_home_is_set() {
        #[cfg(windows)]
        {
            let saved = std::env::var("LOCALAPPDATA").ok();
            unsafe {
                std::env::set_var("LOCALAPPDATA", r"C:\Users\Test\AppData\Local");
            }

            let result = checkpoint_path();
            assert!(
                result.is_some(),
                "checkpoint_path() should return Some when LOCALAPPDATA is set"
            );

            let path = result.unwrap();
            assert_eq!(
                path.file_name(),
                Some(std::ffi::OsStr::new("checkpoint.toml"))
            );
            assert_eq!(
                path.parent().and_then(|p| p.file_name()),
                Some(std::ffi::OsStr::new("vibestats"))
            );

            if let Some(value) = saved {
                unsafe {
                    std::env::set_var("LOCALAPPDATA", value);
                }
            } else {
                unsafe {
                    std::env::remove_var("LOCALAPPDATA");
                }
            }
        }

        #[cfg(not(windows))]
        {
            let saved = std::env::var("HOME").ok();
            unsafe {
                std::env::set_var("HOME", "/tmp/vibestats-test-home");
            }

            let result = checkpoint_path();
            assert!(
                result.is_some(),
                "checkpoint_path() should return Some when HOME is set"
            );

            let path = result.unwrap();
            assert_eq!(
                path.file_name(),
                Some(std::ffi::OsStr::new("checkpoint.toml"))
            );
            assert_eq!(
                path.parent().and_then(|p| p.file_name()),
                Some(std::ffi::OsStr::new("vibestats"))
            );
            assert_eq!(
                path.parent()
                    .and_then(|p| p.parent())
                    .and_then(|p| p.file_name()),
                Some(std::ffi::OsStr::new(".config"))
            );

            if let Some(value) = saved {
                unsafe {
                    std::env::set_var("HOME", value);
                }
            } else {
                unsafe {
                    std::env::remove_var("HOME");
                }
            }
        }
    }

    #[test]
    fn checkpoint_path_returns_none_when_home_unset() {
        #[cfg(windows)]
        const ENV_NAME: &str = "LOCALAPPDATA";

        #[cfg(not(windows))]
        const ENV_NAME: &str = "HOME";

        let saved = std::env::var(ENV_NAME).ok();

        unsafe {
            std::env::remove_var(ENV_NAME);
        }

        let result = checkpoint_path();

        if let Some(value) = saved {
            unsafe {
                std::env::set_var(ENV_NAME, value);
            }
        }

        assert!(
            result.is_none(),
            "checkpoint_path() should return None when {ENV_NAME} is unset"
        );
    }
}
