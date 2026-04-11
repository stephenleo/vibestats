//! GitHub Contents API module for vibestats.
//!
//! Provides all HTTP calls to the GitHub Contents API.
//! No other module may make inline HTTP calls to GitHub — all GitHub HTTP
//! goes through this module (architecture constraint).
//!
//! # Responsibilities
//! - GET file SHA (to detect create vs update)
//! - PUT file (create or update via GitHub Contents API)
//! - Exponential backoff retry on 429 / 5xx / transport errors
//! - Base64 encoding of file content (std-only, no external crate)
//! - Error logging via `logger::error` before propagating errors
//!
//! # Out of scope
//! - Calling `std::process::exit` — callers handle exit (NFR10)
//! - Writing to stdout or stderr (NFR11)
//! - Reading config — caller passes token + repo as constructor args
//! - Checkpoint state — caller (`sync.rs`) manages checkpoint

// The public API is not yet called by other modules (callers land in Story 3.1).
// Suppress dead-code lints so `cargo clippy --all-targets -- -D warnings` passes.
#![allow(dead_code)]
// `ureq::Error` is a third-party type sized at ~272 bytes. We cannot reduce its
// size, so suppress result_large_err for this module. The retry wrapper boxes
// errors after inspecting retriability, so callers never hold large errors.
#![allow(clippy::result_large_err)]

use crate::logger;

// ─── Public types ─────────────────────────────────────────────────────────────

/// Handles all GitHub Contents API interactions for vibestats.
pub struct GithubApi {
    token: String,
    repo: String,
}

/// Type alias for error propagation throughout this module.
pub type GithubApiError = Box<dyn std::error::Error>;

// ─── Retriability helpers ─────────────────────────────────────────────────────

/// Returns `true` if an HTTP status code should trigger a retry.
///
/// Retriable: 429 (rate limit), 5xx (server error)
/// Non-retriable: 401, 404, 422, other 4xx
fn is_status_retriable(code: u16) -> bool {
    code == 429 || code >= 500
}

/// Inspect a `ureq::Error` and return `(is_retriable, boxed_error)`.
///
/// Extracts the retriability flag *before* boxing so callers don't need to
/// hold an unboxed `ureq::Error` in a `Result` (which triggers the
/// `clippy::result_large_err` lint due to `ureq::Error` being 272 bytes).
fn classify(err: ureq::Error) -> (bool, GithubApiError) {
    let retriable = match &err {
        ureq::Error::Status(code, _) => is_status_retriable(*code),
        ureq::Error::Transport(_) => true,
    };
    (retriable, Box::new(err))
}

// ─── Retry wrapper ────────────────────────────────────────────────────────────

/// Execute `f` with exponential backoff retry on retriable errors.
///
/// The closure `f` must return `Result<T, ureq::Error>` so retriability can be
/// inspected before boxing. The outer return type is `Result<T, GithubApiError>`
/// (boxed) for ergonomic use throughout the module.
///
/// Retry policy:
/// - Max 3 attempts
/// - Delay before attempt 1: 1s; before attempt 2: 2s (no delay before attempt 0)
/// - Retriable: HTTP 429, HTTP 5xx, transport errors (timeout, DNS)
/// - Non-retriable: 401, 404, other 4xx — propagates immediately
///
/// On final failure: returns the last error (boxed).
#[allow(clippy::result_large_err)]
fn with_retry<F, T>(f: F) -> Result<T, GithubApiError>
where
    F: Fn() -> Result<T, ureq::Error>,
{
    let delays_secs = [1u64, 2]; // delays BEFORE attempts 1 and 2 (not before attempt 0)
    let max_attempts: usize = 3;
    // Seed with a synthetic "retry exhausted" error so we never rely on
    // `unwrap()` in the fallthrough path. This is overwritten on every
    // retriable failure; the seed only surfaces if `max_attempts == 0`.
    let mut last_err: GithubApiError = Box::<dyn std::error::Error>::from(
        "github_api: retry exhausted with no recorded error",
    );

    for attempt in 0..max_attempts {
        // Sleep before retry (not before the first attempt)
        if attempt > 0 {
            let delay = delays_secs[attempt - 1];
            std::thread::sleep(std::time::Duration::from_secs(delay));
        }

        match f() {
            Ok(val) => return Ok(val),
            Err(e) => {
                let (retriable, boxed) = classify(e);
                if retriable {
                    last_err = boxed;
                    // continue to next attempt
                } else {
                    // Non-retriable: 401, 404, other 4xx — fail immediately
                    return Err(boxed);
                }
            }
        }
    }

    // All attempts exhausted — return the last recorded error.
    Err(last_err)
}

// ─── Constructor ──────────────────────────────────────────────────────────────

impl GithubApi {
    /// Create a new `GithubApi` instance.
    ///
    /// * `token` — GitHub personal access token (oauth_token from config)
    /// * `repo`  — Full repository name, e.g. `"owner/repo-name"` (vibestats_data_repo from config)
    pub fn new(token: &str, repo: &str) -> Self {
        Self {
            token: token.to_string(),
            repo: repo.to_string(),
        }
    }

    // ─── Public API ───────────────────────────────────────────────────────────

    /// Create or update a file in the GitHub repository.
    ///
    /// * `path`    — Full path within the repo (e.g. `machines/year=2026/month=04/day=10/.../data.json`)
    /// * `content` — Raw string content to write (will be base64-encoded before upload)
    ///
    /// Behaviour:
    /// - Calls `get_file_sha` to detect whether the file already exists.
    /// - If not found (404): creates the file (PUT without `sha`).
    /// - If found: updates the file (PUT with current `sha`).
    /// - Retries on 429 / 5xx / transport errors (exponential backoff).
    /// - On 401: logs and returns `Err` without retrying.
    pub fn put_file(&self, path: &str, content: &str) -> Result<(), GithubApiError> {
        let encoded = base64_encode(content.as_bytes());

        // Step 1: Get SHA (with retry)
        let current_sha = match with_retry(|| get_file_sha_inner(&self.token, &self.repo, path)) {
            Ok(sha) => sha,
            Err(e) => {
                logger::error(&format!("github_api: get_file_sha failed for {}: {}", path, e));
                return Err(e);
            }
        };

        // Step 2: PUT file (with retry)
        match with_retry(|| put_file_inner(&self.token, &self.repo, path, &encoded, &current_sha))
        {
            Ok(()) => Ok(()),
            Err(e) => {
                logger::error(&format!("github_api: put_file failed for {}: {}", path, e));
                Err(e)
            }
        }
    }

    /// Retrieve the current SHA of a file in the repository.
    ///
    /// Returns:
    /// - `Ok(Some(sha))` — file exists, `sha` is the current blob SHA
    /// - `Ok(None)`      — file does not exist (404)
    /// - `Err(_)`        — network error or unexpected HTTP status
    pub fn get_file_sha(&self, path: &str) -> Result<Option<String>, GithubApiError> {
        with_retry(|| get_file_sha_inner(&self.token, &self.repo, path))
    }
}

// ─── Internal HTTP helpers ────────────────────────────────────────────────────

/// Inner GET helper — returns `Ok(Some(sha))`, `Ok(None)` for 404, or `Err`.
///
/// Returns `ureq::Error` directly (not boxed) so `with_retry` can classify
/// the error for retriability before boxing.
#[allow(clippy::result_large_err)]
fn get_file_sha_inner(
    token: &str,
    repo: &str,
    path: &str,
) -> Result<Option<String>, ureq::Error> {
    let url = format!("https://api.github.com/repos/{}/contents/{}", repo, path);

    let response = ureq::get(&url)
        .set("Authorization", &format!("Bearer {}", token))
        .set("User-Agent", "vibestats")
        .set("Accept", "application/vnd.github+json")
        .set("X-GitHub-Api-Version", "2022-11-28")
        .call();

    match response {
        Ok(r) => {
            // 200: file exists — parse sha from response body.
            //
            // Body read and JSON parse failures must NOT be collapsed into
            // `Ok(None)`: a subsequent PUT-without-sha against an existing
            // file would return 422 from GitHub and mask a real transport
            // or server-side problem. Surface these as ureq Transport
            // errors so the retry wrapper classifies them as retriable
            // and the caller logs them.
            let body = r.into_string().map_err(ureq::Error::from)?;

            let json: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
                // Malformed JSON is a server contract violation. Wrap in a
                // synthetic io::Error so From<io::Error> yields a
                // Transport variant that with_retry will classify as
                // retriable.
                ureq::Error::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("github_api: malformed JSON from Contents API: {}", e),
                ))
            })?;

            Ok(json["sha"].as_str().map(|s| s.to_string()))
        }
        Err(ureq::Error::Status(404, _)) => {
            // File does not exist — first-time create path
            Ok(None)
        }
        Err(e) => Err(e),
    }
}

/// Inner PUT helper. Returns `Ok(())` on 200/201, `Err` otherwise.
///
/// Returns `ureq::Error` directly (not boxed) so `with_retry` can classify
/// the error for retriability before boxing.
#[allow(clippy::result_large_err)]
fn put_file_inner(
    token: &str,
    repo: &str,
    path: &str,
    encoded_content: &str,
    current_sha: &Option<String>,
) -> Result<(), ureq::Error> {
    let url = format!("https://api.github.com/repos/{}/contents/{}", repo, path);

    let body = if let Some(sha) = current_sha {
        serde_json::json!({
            "message": "vibestats sync",
            "content": encoded_content,
            "sha": sha
        })
        .to_string()
    } else {
        serde_json::json!({
            "message": "vibestats sync",
            "content": encoded_content
        })
        .to_string()
    };

    let response = ureq::put(&url)
        .set("Authorization", &format!("Bearer {}", token))
        .set("User-Agent", "vibestats")
        .set("Accept", "application/vnd.github+json")
        .set("X-GitHub-Api-Version", "2022-11-28")
        .set("Content-Type", "application/json")
        .send_string(&body);

    match response {
        Ok(_) => Ok(()), // 200 (update) or 201 (create) = success
        Err(e) => Err(e),
    }
}

// ─── Base64 encoding (std-only, RFC 4648 standard alphabet) ───────────────────

/// Encode `input` bytes as standard Base64 (RFC 4648).
///
/// Uses the standard alphabet (`A–Z`, `a–z`, `0–9`, `+`, `/`) with `=` padding.
/// No external crates — stdlib only.
fn base64_encode(input: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    let mut i = 0;

    // Process full 3-byte groups
    while i + 2 < input.len() {
        let b0 = input[i] as usize;
        let b1 = input[i + 1] as usize;
        let b2 = input[i + 2] as usize;
        out.push(ALPHABET[b0 >> 2] as char);
        out.push(ALPHABET[((b0 & 0x3) << 4) | (b1 >> 4)] as char);
        out.push(ALPHABET[((b1 & 0xf) << 2) | (b2 >> 6)] as char);
        out.push(ALPHABET[b2 & 0x3f] as char);
        i += 3;
    }

    // Handle remaining bytes with padding
    match input.len() - i {
        1 => {
            let b0 = input[i] as usize;
            out.push(ALPHABET[b0 >> 2] as char);
            out.push(ALPHABET[(b0 & 0x3) << 4] as char);
            out.push('=');
            out.push('=');
        }
        2 => {
            let b0 = input[i] as usize;
            let b1 = input[i + 1] as usize;
            out.push(ALPHABET[b0 >> 2] as char);
            out.push(ALPHABET[((b0 & 0x3) << 4) | (b1 >> 4)] as char);
            out.push(ALPHABET[(b1 & 0xf) << 2] as char);
            out.push('=');
        }
        _ => {} // 0 remaining bytes — no padding needed
    }

    out
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── base64_encode test vectors (RFC 4648) ─────────────────────────────────

    #[test]
    fn test_base64_empty() {
        assert_eq!(base64_encode(b""), "");
    }

    #[test]
    fn test_base64_one_byte() {
        // "M" → "TQ=="
        assert_eq!(base64_encode(b"M"), "TQ==");
    }

    #[test]
    fn test_base64_two_bytes() {
        // "Ma" → "TWE="
        assert_eq!(base64_encode(b"Ma"), "TWE=");
    }

    #[test]
    fn test_base64_three_bytes() {
        // "Man" → "TWFu"
        assert_eq!(base64_encode(b"Man"), "TWFu");
    }

    #[test]
    fn test_base64_four_bytes() {
        // "Many" → "TWFueQ=="
        assert_eq!(base64_encode(b"Many"), "TWFueQ==");
    }

    #[test]
    fn test_base64_hello() {
        // "hello" → "aGVsbG8="
        assert_eq!(base64_encode(b"hello"), "aGVsbG8=");
    }

    #[test]
    fn test_base64_all_zeros() {
        // 3 zero bytes → "AAAA"
        assert_eq!(base64_encode(&[0u8, 0, 0]), "AAAA");
    }

    #[test]
    fn test_base64_all_ones() {
        // 3 bytes of 0xFF → "////"
        assert_eq!(base64_encode(&[0xFFu8, 0xFF, 0xFF]), "////");
    }

    #[test]
    fn test_base64_longer_string() {
        // "Many hands make light work." → known Base64 output
        assert_eq!(
            base64_encode(b"Many hands make light work."),
            "TWFueSBoYW5kcyBtYWtlIGxpZ2h0IHdvcmsu"
        );
    }

    // ── is_status_retriable: HTTP status classification ───────────────────────

    #[test]
    fn test_status_retriable_429() {
        assert!(is_status_retriable(429), "429 (rate limit) must be retriable");
    }

    #[test]
    fn test_status_retriable_500() {
        assert!(is_status_retriable(500), "500 (server error) must be retriable");
    }

    #[test]
    fn test_status_retriable_503() {
        assert!(is_status_retriable(503), "503 (service unavailable) must be retriable");
    }

    #[test]
    fn test_status_retriable_599() {
        assert!(is_status_retriable(599), "all 5xx must be retriable");
    }

    #[test]
    fn test_status_not_retriable_401() {
        assert!(!is_status_retriable(401), "401 (unauthorized) must NOT be retriable");
    }

    #[test]
    fn test_status_not_retriable_404() {
        assert!(!is_status_retriable(404), "404 (not found) must NOT be retriable");
    }

    #[test]
    fn test_status_not_retriable_422() {
        assert!(
            !is_status_retriable(422),
            "422 (unprocessable entity) must NOT be retriable"
        );
    }

    #[test]
    fn test_status_not_retriable_200() {
        assert!(!is_status_retriable(200), "200 (OK) must NOT be retriable");
    }

    #[test]
    fn test_status_not_retriable_400() {
        assert!(!is_status_retriable(400), "400 (bad request) must NOT be retriable");
    }

    // ── with_retry: success on first attempt ──────────────────────────────────

    #[test]
    fn test_retry_succeeds_on_first_attempt() {
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let cc = call_count.clone();

        let result: Result<i32, GithubApiError> = with_retry(|| {
            cc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(42)
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn test_retry_invokes_f_exactly_once_on_success() {
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let cc = call_count.clone();

        let _ = with_retry::<_, ()>(|| {
            cc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        });

        assert_eq!(
            call_count.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "f should be invoked exactly once when it succeeds immediately"
        );
    }

    // ── with_retry: transport errors are retriable ────────────────────────────
    //
    // `std::io::Error` implements `From<io::Error> for ureq::Error` via the
    // public `ureq::Error::from` conversion, giving us a valid Transport variant
    // without requiring network access or test-mode server infrastructure.

    fn make_transport_error() -> ureq::Error {
        ureq::Error::from(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "simulated network error for test",
        ))
    }

    #[test]
    fn test_classify_transport_error_is_retriable() {
        let err = make_transport_error();
        let (retriable, _) = classify(err);
        assert!(retriable, "transport errors must be classified as retriable");
    }

    #[test]
    fn test_retry_transport_error_exhausts_3_attempts() {
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let cc = call_count.clone();

        // Always return a transport error — should exhaust all 3 attempts
        let result: Result<(), GithubApiError> = with_retry(|| {
            cc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Err(make_transport_error())
        });

        assert!(result.is_err());
        assert_eq!(
            call_count.load(std::sync::atomic::Ordering::SeqCst),
            3,
            "transport error should trigger 3 total attempts (no early exit)"
        );
    }

    #[test]
    fn test_retry_succeeds_after_two_transport_errors() {
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let cc = call_count.clone();

        // Fail with transport error twice, succeed on third attempt
        let result: Result<i32, GithubApiError> = with_retry(|| {
            let n = cc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if n < 2 {
                Err(make_transport_error())
            } else {
                Ok(77)
            }
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 77);
        assert_eq!(
            call_count.load(std::sync::atomic::Ordering::SeqCst),
            3,
            "should have been called exactly 3 times (2 failures + 1 success)"
        );
    }

    // ── get_file_sha: SHA parsing from JSON body ──────────────────────────────

    #[test]
    fn test_parse_sha_present_in_json_body() {
        let json_body = r#"{"sha": "abc123def456", "content": "aGVsbG8=", "encoding": "base64"}"#;
        let json: serde_json::Value = serde_json::from_str(json_body).unwrap();
        let sha = json["sha"].as_str().map(|s| s.to_string());
        assert_eq!(sha, Some("abc123def456".to_string()));
    }

    #[test]
    fn test_parse_sha_missing_field_returns_none() {
        let json_body = r#"{"content": "aGVsbG8=", "encoding": "base64"}"#;
        let json: serde_json::Value = serde_json::from_str(json_body).unwrap();
        let sha = json["sha"].as_str().map(|s| s.to_string());
        assert_eq!(sha, None);
    }

    // ── put_file body construction ────────────────────────────────────────────

    #[test]
    fn test_put_body_without_sha_excludes_sha_field() {
        // When SHA is None, the JSON body must NOT include a "sha" field (first-time create)
        let sha: Option<String> = None;
        let body = build_put_body("aGVsbG8=", &sha);
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(
            parsed.get("sha").is_none(),
            "body must not include sha field when creating a new file"
        );
        assert_eq!(parsed["message"], "vibestats sync");
        assert_eq!(parsed["content"], "aGVsbG8=");
    }

    #[test]
    fn test_put_body_with_sha_includes_sha_field() {
        // When SHA is Some, the JSON body must include the "sha" field (update)
        let sha: Option<String> = Some("abc123".to_string());
        let body = build_put_body("aGVsbG8=", &sha);
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            parsed["sha"], "abc123",
            "body must include sha field when updating existing file"
        );
        assert_eq!(parsed["message"], "vibestats sync");
        assert_eq!(parsed["content"], "aGVsbG8=");
    }

    /// Helper that mirrors the body-building logic in `put_file_inner`.
    fn build_put_body(encoded_content: &str, current_sha: &Option<String>) -> String {
        if let Some(sha) = current_sha {
            serde_json::json!({
                "message": "vibestats sync",
                "content": encoded_content,
                "sha": sha
            })
            .to_string()
        } else {
            serde_json::json!({
                "message": "vibestats sync",
                "content": encoded_content
            })
            .to_string()
        }
    }

    // ── GithubApi::new ────────────────────────────────────────────────────────

    #[test]
    fn test_github_api_new_stores_token_and_repo() {
        let api = GithubApi::new("my-token", "owner/repo");
        assert_eq!(api.token, "my-token");
        assert_eq!(api.repo, "owner/repo");
    }

    // ── base64_encode produces valid GitHub Contents API encoding ─────────────

    #[test]
    fn test_base64_output_uses_standard_alphabet() {
        // Verify that JSON content encodes to base64 with only valid standard alphabet chars
        let content = r#"{"key": "value", "num": 42}"#;
        let encoded = base64_encode(content.as_bytes());
        assert!(!encoded.is_empty());
        for c in encoded.chars() {
            assert!(
                c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=',
                "invalid base64 character in output: {c:?}"
            );
        }
    }
}
