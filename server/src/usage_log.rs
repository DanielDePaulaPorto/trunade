//! Local, anonymous record of how a lesson was used, for classroom research:
//! one JSON Lines file per `trunade` run in `~/.local/share/trunade/logs/`.
//! It holds what happened in the page (runs, outcomes, verdicts, quiz answers,
//! sections shown), never who was there. Writing errors never stop the lesson.

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value, json};

/// Events the page may record; anything else it sends is dropped.
pub const PAGE_EVENTS: &[&str] = &["section", "quiz", "step", "zoom", "print", "runAll", "visibility"];

pub struct UsageLog {
    file: Option<Mutex<File>>,
}

impl UsageLog {
    /// Starts a new file named after the start time and the lesson.
    pub fn create(dir: &Path, document: &Path) -> std::io::Result<(Self, PathBuf)> {
        std::fs::create_dir_all(dir)?;
        let stem = document.file_stem().unwrap_or_default().to_string_lossy();
        let path = dir.join(format!("{}-{stem}.jsonl", file_timestamp(SystemTime::now())));
        let file = File::options().create_new(true).append(true).open(&path)?;
        Ok((Self { file: Some(Mutex::new(file)) }, path))
    }

    pub fn disabled() -> Self {
        Self { file: None }
    }

    /// Appends `{"time": …, "event": …, …fields}` as one line.
    pub fn record(&self, event: &str, fields: Value) {
        let Some(file) = &self.file else { return };
        let mut line = Map::new();
        line.insert("time".into(), json!(iso_timestamp(SystemTime::now())));
        line.insert("event".into(), json!(event));
        if let Value::Object(fields) = fields {
            line.extend(fields.into_iter().filter(|(key, _)| key != "time" && key != "event"));
        }
        let mut text = Value::Object(line).to_string();
        text.push('\n');
        if let Ok(mut file) = file.lock() {
            let _ = file.write_all(text.as_bytes());
        }
    }
}

/// `$XDG_DATA_HOME/trunade/logs`, or `~/.local/share/trunade/logs`.
pub fn default_dir() -> Option<PathBuf> {
    let data_home = std::env::var_os("XDG_DATA_HOME")
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))?;
    Some(data_home.join("trunade/logs"))
}

/// UTC, millisecond precision: `2026-10-16T14:02:33.120Z`.
fn iso_timestamp(time: SystemTime) -> String {
    let (date, seconds_of_day, millis) = split_time(time);
    let (h, m, s) = clock(seconds_of_day);
    format!("{date}T{h:02}:{m:02}:{s:02}.{millis:03}Z")
}

/// Sorts by name and has no `:` (not allowed in file names everywhere): `2026-10-16T14-02-33Z`.
fn file_timestamp(time: SystemTime) -> String {
    let (date, seconds_of_day, _) = split_time(time);
    let (h, m, s) = clock(seconds_of_day);
    format!("{date}T{h:02}-{m:02}-{s:02}Z")
}

fn split_time(time: SystemTime) -> (String, u64, u32) {
    let since_epoch = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let seconds = since_epoch.as_secs();
    let (year, month, day) = civil_from_days((seconds / 86_400) as i64);
    (format!("{year:04}-{month:02}-{day:02}"), seconds % 86_400, since_epoch.subsec_millis())
}

fn clock(seconds_of_day: u64) -> (u64, u64, u64) {
    (seconds_of_day / 3600, seconds_of_day / 60 % 60, seconds_of_day % 60)
}

/// Days since 1970-01-01 → (year, month, day). Howard Hinnant's algorithm.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * month_index + 2) / 5 + 1) as u32;
    let month = if month_index < 10 { month_index + 3 } else { month_index - 9 } as u32;
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn timestamps_are_utc() {
        let time = UNIX_EPOCH + Duration::from_millis(1_792_160_553_120);
        assert_eq!(iso_timestamp(time), "2026-10-16T14:22:33.120Z");
        assert_eq!(file_timestamp(time), "2026-10-16T14-22-33Z");
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(11_016), (2000, 2, 29));
    }

    #[test]
    fn records_one_json_object_per_line() {
        let dir = tempfile::tempdir().unwrap();
        let (log, path) = UsageLog::create(dir.path(), Path::new("/aulas/aula14.md")).unwrap();
        assert!(path.file_name().unwrap().to_string_lossy().ends_with("-aula14.jsonl"));
        log.record("run", json!({ "line": 12, "event": "ignored" }));
        log.record("section", json!({ "title": "GCD" }));
        let text = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<Value> = text.lines().map(|line| serde_json::from_str(line).unwrap()).collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0]["event"], "run");
        assert_eq!(lines[0]["line"], 12);
        assert_eq!(lines[1]["title"], "GCD");
        assert!(lines[1]["time"].as_str().unwrap().ends_with('Z'));
    }

    #[test]
    fn disabled_log_writes_nothing() {
        UsageLog::disabled().record("run", json!({}));
    }
}
