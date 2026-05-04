//! Harness registry and trait.
//!
//! Each harness (Claude Code, Codex, etc.) lives in its own submodule and
//! implements [`Harness`]. The registry below enumerates every harness vibestats
//! supports — adding a new harness is two lines in this file plus a new
//! `src/harnesses/<name>.rs`. See `CONTRIBUTING.md` for the recipe.

// The trait, registry, and lookup helpers are unused until Tasks 2-3 add the
// Claude and Codex entries and migrate dispatch sites to call them.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// Per-day aggregated session activity. This is the harness-agnostic output
/// every parser produces. Field order, names, and types are part of the
/// stable serialized contract — the `sha256_payload_known_vector` test in
/// `src/sync.rs` pins the SHA256 of a known instance to catch drift.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DailyActivity {
    pub sessions: u32,
    pub active_minutes: u32,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
    /// Maps model name → total output tokens attributed to that model on this day.
    /// `BTreeMap` ensures deterministic serialization order (alphabetical keys).
    pub models: BTreeMap<String, u64>,
    pub longest_session_minutes: u32,
    pub message_count: u32,
    pub tool_uses: u32,
}

/// One AI coding harness. Implementations live in their own submodule.
pub trait Harness: Sync {
    /// Stable id used in CLI args, checkpoint keys, and hive paths.
    /// MUST be lowercase ASCII with no spaces. Changing this is a breaking
    /// change to the on-disk and on-remote data format.
    fn id(&self) -> &'static str;

    /// Human-readable name for log/error messages.
    fn display_name(&self) -> &'static str;

    /// Returns true if this harness is installed on the local machine
    /// (typically: its session directory exists under `$HOME`).
    fn is_installed(&self) -> bool;

    /// Walk local session files and aggregate per-day activity for dates in
    /// `[start, end]` inclusive (YYYY-MM-DD strings). Returns an empty map
    /// when the harness is not installed or has no data.
    fn parse_date_range(&self, start: &str, end: &str) -> HashMap<String, DailyActivity>;
}

// Registry starts empty; Tasks 2 and 3 add the Claude and Codex entries.
static REGISTRY: &[&dyn Harness] = &[];

/// All registered harnesses, in registry order.
pub fn all() -> &'static [&'static dyn Harness] {
    REGISTRY
}

/// Look up a harness by its stable id.
pub fn by_id(id: &str) -> Option<&'static dyn Harness> {
    REGISTRY.iter().copied().find(|h| h.id() == id)
}

/// All registered harness ids.
pub fn ids() -> Vec<&'static str> {
    REGISTRY.iter().map(|h| h.id()).collect()
}
