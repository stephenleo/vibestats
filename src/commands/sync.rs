use crate::codex_parser;
use crate::jsonl_parser;
use crate::sync::Harness;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HarnessSelection {
    All,
    Claude,
    Codex,
}

impl HarnessSelection {
    fn harnesses(self) -> &'static [Harness] {
        match self {
            Self::All => &[Harness::Claude, Harness::Codex],
            Self::Claude => &[Harness::Claude],
            Self::Codex => &[Harness::Codex],
        }
    }
}

/// Computes today's date in UTC as "YYYY-MM-DD" using only std.
/// Uses the civil-from-days algorithm (Howard Hinnant):
/// https://howardhinnant.github.io/date_algorithms.html
///
/// This is identical logic to `checkpoint.rs::format_iso8601_utc` and
/// `logger.rs::epoch_to_datetime` minus the time component.
fn today_utc() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Days since Unix epoch
    let z = secs / 86400;

    // Civil-from-days: https://howardhinnant.github.io/date_algorithms.html
    let z = z + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };

    format!("{:04}-{:02}-{:02}", y, mo, d)
}

/// Entry point called from `main.rs` for the `vibestats sync` command.
///
/// - `backfill = false`: syncs today only (unthrottled).
/// - `backfill = true`: discovers all dates in selected harness history and syncs the
///   full range from the earliest date to today.
///
/// NEVER calls `std::process::exit` — `main.rs` handles exit.
pub fn run(backfill: bool, selection: HarnessSelection, quiet: bool) {
    let today = today_utc();
    let harnesses = selection.harnesses();

    if !backfill {
        crate::sync::run_harnesses(&today, &today, harnesses);
        if !quiet {
            println!("vibestats: sync complete");
        }
    } else {
        // Discover all historical dates from selected harnesses.
        // "0000-00-00" is lexicographically less than any real ISO date, so
        // parse_date_range returns every date present in local history.
        let mut activities = std::collections::HashMap::new();
        for harness in harnesses {
            let harness_activities = match harness {
                Harness::Claude => jsonl_parser::parse_date_range("0000-00-00", &today),
                Harness::Codex => codex_parser::parse_date_range("0000-00-00", &today),
            };
            for date in harness_activities.keys() {
                activities.insert(date.clone(), ());
            }
        }

        if activities.is_empty() {
            if !quiet {
                println!("vibestats: backfill complete — no local data found");
            }
            return;
        }

        let mut dates: Vec<&String> = activities.keys().collect();
        dates.sort();
        let earliest = dates[0].clone();
        let count = dates.len();

        crate::sync::run_harnesses(&earliest, &today, harnesses);
        if !quiet {
            println!("vibestats: backfill complete — processed {} date(s)", count);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn today_utc_has_correct_format() {
        let date = today_utc();
        // Must be exactly 10 characters: "YYYY-MM-DD"
        assert_eq!(date.len(), 10, "date must be 10 chars: got '{}'", date);
        // Correct separators at positions 4 and 7
        assert_eq!(
            &date[4..5],
            "-",
            "expected '-' at position 4, got '{}'",
            date
        );
        assert_eq!(
            &date[7..8],
            "-",
            "expected '-' at position 7, got '{}'",
            date
        );
        // All other characters must be ASCII digits
        let digits_only: String = date.chars().filter(|c| *c != '-').collect();
        assert!(
            digits_only.chars().all(|c| c.is_ascii_digit()),
            "non-digit characters in date: '{}'",
            date
        );
    }

    #[test]
    fn today_utc_year_is_at_least_2026() {
        let date = today_utc();
        let year: u32 = date[0..4].parse().expect("year portion must be a number");
        assert!(
            year >= 2026,
            "year {} is before 2026 — likely an epoch bug",
            year
        );
    }
}
