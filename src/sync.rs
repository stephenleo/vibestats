use crate::checkpoint::Checkpoint;
use crate::config::Config;
use crate::github_api::GithubApi;
use crate::logger;
use std::path::PathBuf;

/// Returns the path to the checkpoint file, or None if HOME is not set.
fn checkpoint_path() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(|h| {
        PathBuf::from(h)
            .join(".config")
            .join("vibestats")
            .join("checkpoint.toml")
    })
}

/// Constructs the Hive path for a given date, harness id, and machine_id.
/// Input: date as "YYYY-MM-DD", harness_id from the Harness trait, machine_id from config.
/// Output: "machines/year=YYYY/month=MM/day=DD/harness=<harness_id>/machine_id=<id>/data.json"
fn hive_path(date: &str, harness_id: &str, machine_id: &str) -> String {
    // date is "YYYY-MM-DD" — indexing is safe because parse_date_range only returns
    // dates extracted from JSONL timestamps in that exact format.
    let year = &date[0..4];
    let month = &date[5..7];
    let day = &date[8..10];
    format!(
        "machines/year={}/month={}/day={}/harness={}/machine_id={}/data.json",
        year, month, day, harness_id, machine_id
    )
}

/// Computes SHA256 of data and returns lowercase hex string (64 chars).
/// Implemented std-only — no external crate.
fn sha256_hex(data: &[u8]) -> String {
    // SHA256 initial hash values (first 32 bits of fractional parts of square roots of primes 2..19)
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    // SHA256 round constants (first 32 bits of fractional parts of cube roots of primes 2..311)
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    // Pre-processing: padding
    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0x00);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    // Process each 512-bit chunk
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for (i, b) in chunk.chunks(4).enumerate().take(16) {
            w[i] = u32::from_be_bytes([b[0], b[1], b[2], b[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] =
            [h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]];
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    format!(
        "{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}",
        h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]
    )
}

/// Orchestrates the sync operation for a given date range across all registered harnesses.
///
/// Loads config and checkpoint internally. Calls each harness's `parse_date_range` via
/// the trait, computes per-date payload hashes, skips unchanged dates, pushes changed
/// dates via `github_api`, and updates the checkpoint. Always saves checkpoint and
/// returns `()` — never calls `std::process::exit`.
pub fn run(start_date: &str, end_date: &str) {
    run_harnesses(start_date, end_date, crate::harnesses::all());
}

pub fn run_harnesses(
    start_date: &str,
    end_date: &str,
    harnesses: &[&'static dyn crate::harnesses::Harness],
) {
    let config = Config::load_or_exit();
    let cp_path = checkpoint_path();
    let mut checkpoint = cp_path.as_deref().map(Checkpoint::load).unwrap_or_default();

    let api = GithubApi::new(&config.oauth_token, &config.vibestats_data_repo);
    for harness in harnesses {
        let activities = harness.parse_date_range(start_date, end_date);

        // Iterate dates in sorted order so log output and HTTP call ordering are
        // deterministic across runs (HashMap iteration order is randomized per run).
        // This has no correctness impact — idempotency is guaranteed by the hash
        // check per date — but it keeps the log stream reproducible for debugging.
        let mut dates: Vec<&String> = activities.keys().collect();
        dates.sort();

        // INVARIANT: this loop is the only path that mutates remote state, and it
        // only ever calls put_file — never delete. Combined with iterating solely
        // over dates the parser actually returned, this makes sync non-destructive:
        // a date that is fully pruned locally produces no entry in `activities`,
        // so the remote `data.json` for that date is left untouched. Future edits
        // must preserve both halves — adding a "reconcile/trim" path that deletes
        // remote dates absent from local would silently destroy archived history.
        for date in dates {
            let activity = &activities[date];
            // serde_json serializes struct fields in definition order; BTreeMap keys sort
            // alphabetically — both guarantee deterministic bytes for the SHA256 hash (NFR12).
            let payload =
                serde_json::to_string(activity).expect("DailyActivity serialization is infallible");
            let hash = sha256_hex(payload.as_bytes());

            // Skip if hash matches checkpoint (idempotency — NFR12)
            if checkpoint.hash_matches_for_harness(harness.id(), date, &hash) {
                continue;
            }

            let path = hive_path(date, harness.id(), &config.machine_id);
            match api.put_file(&path, &payload) {
                Ok(()) => {
                    checkpoint.update_hash_for_harness(harness.id(), date, &hash);
                    checkpoint.clear_auth_error();
                }
                Err(e) => {
                    // Treat all API errors as potential auth errors (err on the side of
                    // caution; user can clear via `vibestats auth`). Log and continue.
                    checkpoint.set_auth_error();
                    logger::error(&format!(
                        "sync: put_file failed for {} {date}: {e}",
                        harness.id()
                    ));
                }
            }
        }
    }

    // Always save checkpoint after processing all dates (AC #6)
    if let Some(ref path) = cp_path {
        if let Err(e) = checkpoint.save(path) {
            logger::error(&format!("sync: failed to save checkpoint: {e}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── sha256_hex test vectors ──────────────────────────────────────────────

    #[test]
    fn sha256_empty_string() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn sha256_hello() {
        assert_eq!(
            sha256_hex(b"hello"),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn sha256_is_deterministic() {
        let input = b"{\"sessions\":4,\"active_minutes\":87}";
        let h1 = sha256_hex(input);
        let h2 = sha256_hex(input);
        assert_eq!(h1, h2, "sha256_hex must be deterministic");
        // Result must be lowercase 64-char hex
        assert_eq!(h1.len(), 64);
        assert!(h1
            .chars()
            .all(|c| c.is_ascii_digit() || c.is_ascii_lowercase()));
    }

    #[test]
    fn sha256_payload_known_vector() {
        // Pin the serialized payload hash so any accidental change to the byte
        // format (field order, spacing, new fields) breaks loudly rather than
        // silently violating idempotency (NFR12) against already-synced data.
        // Fields serialized in DailyActivity definition order; BTreeMap models: {}.
        use crate::harnesses::DailyActivity;
        let activity = DailyActivity {
            sessions: 4,
            active_minutes: 87,
            ..Default::default()
        };
        let payload = serde_json::to_string(&activity).unwrap();
        assert_eq!(
            sha256_hex(payload.as_bytes()),
            "2bbeb81ced9d736690d79e2ff461ada5c42db9f23f22ff578fe8d90fe473b67e"
        );
    }

    #[test]
    fn payload_format_matches_hashed_bytes() {
        // Verify the exact JSON bytes produced by serde_json for a known DailyActivity.
        // This pins the field order and ensures the payload is byte-for-byte deterministic.
        use crate::harnesses::DailyActivity;
        let activity = DailyActivity {
            sessions: 4,
            active_minutes: 87,
            ..Default::default()
        };
        let payload = serde_json::to_string(&activity).unwrap();
        // Confirm exact byte sequence — field order matches DailyActivity struct definition.
        assert!(
            payload.starts_with("{\"sessions\":4,\"active_minutes\":87,"),
            "payload: {payload}"
        );
        assert!(
            payload.contains("\"models\":{}"),
            "models must be empty: {payload}"
        );
        // Hash must be stable across runs (idempotency — NFR12).
        let h1 = sha256_hex(payload.as_bytes());
        let h2 = sha256_hex(payload.as_bytes());
        assert_eq!(h1, h2);
    }

    // ── hive_path tests ──────────────────────────────────────────────────────

    #[test]
    fn hive_path_formats_correctly() {
        let result = hive_path("2026-04-10", "claude", "stephens-mbp-a1b2c3");
        assert_eq!(
            result,
            "machines/year=2026/month=04/day=10/harness=claude/machine_id=stephens-mbp-a1b2c3/data.json"
        );
    }

    #[test]
    fn hive_path_preserves_zero_padding() {
        let result = hive_path("2026-01-05", "claude", "my-machine-000001");
        assert_eq!(
            result,
            "machines/year=2026/month=01/day=05/harness=claude/machine_id=my-machine-000001/data.json"
        );
    }

    #[test]
    fn hive_path_uses_selected_harness() {
        let result = hive_path("2026-05-03", "codex", "my-machine-000001");
        assert_eq!(
            result,
            "machines/year=2026/month=05/day=03/harness=codex/machine_id=my-machine-000001/data.json"
        );
    }

    // ── idempotency / hash_matches tests ────────────────────────────────────

    #[test]
    fn same_payload_produces_same_hash_idempotency() {
        let payload = r#"{"sessions":4,"active_minutes":87}"#;
        let hash = sha256_hex(payload.as_bytes());

        let mut checkpoint = Checkpoint::default();
        // Before updating: hash does NOT match
        assert!(!checkpoint.hash_matches("2026-04-10", &hash));

        // Update checkpoint with hash
        checkpoint.update_hash("2026-04-10", &hash);

        // After update: hash_matches returns true → put_file should not be called
        assert!(checkpoint.hash_matches("2026-04-10", &hash));
    }

    #[test]
    fn different_payloads_produce_different_hashes() {
        let h1 = sha256_hex(b"{\"sessions\":1,\"active_minutes\":10}");
        let h2 = sha256_hex(b"{\"sessions\":2,\"active_minutes\":20}");
        assert_ne!(h1, h2);
    }
}
