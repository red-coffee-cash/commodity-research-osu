//! Persistent progress: per-method accuracy and speed, and timed-mode bests.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::session::Record;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MethodStats {
    pub attempts: u64,
    pub correct: u64,
    /// Total time spent on correctly answered problems.
    pub correct_ms: u64,
}

impl MethodStats {
    pub fn accuracy(&self) -> Option<f64> {
        (self.attempts > 0).then(|| self.correct as f64 / self.attempts as f64)
    }

    pub fn avg_secs(&self) -> Option<f64> {
        (self.correct > 0).then(|| self.correct_ms as f64 / self.correct as f64 / 1000.0)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Stats {
    pub sessions: u64,
    /// Keyed by method name.
    pub methods: BTreeMap<String, MethodStats>,
    /// Best number correct, keyed by "method · level · duration".
    pub timed_best: BTreeMap<String, u32>,
}

impl Stats {
    /// Load stats. A missing file gives empty stats; an unreadable one is moved
    /// aside (so it isn't overwritten) and a warning is returned.
    pub fn load(path: &Path) -> (Stats, Option<String>) {
        let text = match fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return (Stats::default(), None),
            Err(e) => return (Stats::default(), Some(format!("Couldn't read stats: {e}"))),
        };
        match serde_json::from_str(&text) {
            Ok(s) => (s, None),
            Err(e) => {
                let backup = path.with_extension("json.corrupt");
                let _ = fs::rename(path, &backup);
                let msg = format!("Stats file was unreadable ({e}); moved to {}", backup.display());
                (Stats::default(), Some(msg))
            }
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
        }
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_string_pretty(self)?)
            .with_context(|| format!("writing {}", tmp.display()))?;
        fs::rename(&tmp, path).with_context(|| format!("replacing {}", path.display()))?;
        Ok(())
    }

    pub fn record(&mut self, records: &[Record]) {
        if records.is_empty() {
            return;
        }
        self.sessions += 1;
        for r in records {
            let m = self.methods.entry(r.method.name().to_string()).or_default();
            m.attempts += 1;
            if r.correct {
                m.correct += 1;
                m.correct_ms += r.time.as_millis() as u64;
            }
        }
    }

    /// Record a timed score; returns true if it's a new best.
    pub fn submit_timed(&mut self, key: String, score: u32) -> bool {
        let best = self.timed_best.entry(key).or_insert(0);
        if score > *best {
            *best = score;
            true
        } else {
            false
        }
    }
}

/// `$MENTALMATH_DATA_DIR/stats.json`, or the platform data dir.
pub fn default_path() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("MENTALMATH_DATA_DIR") {
        return Some(PathBuf::from(dir).join("stats.json"));
    }
    directories::ProjectDirs::from("", "", "mentalmath").map(|d| d.data_dir().join("stats.json"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::methods::Method;
    use std::time::Duration;

    fn tmpdir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("mentalmath-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        d
    }

    fn rec(correct: bool, ms: u64) -> Record {
        Record {
            method: Method::Cross,
            prompt: "12 × 34".into(),
            expected: "408".into(),
            given: Some("408".into()),
            correct,
            time: Duration::from_millis(ms),
        }
    }

    #[test]
    fn round_trip_and_aggregation() {
        let path = tmpdir("rt").join("stats.json");
        let mut s = Stats::default();
        s.record(&[rec(true, 1000), rec(false, 5000), rec(true, 3000)]);
        assert!(s.submit_timed("k".into(), 5));
        assert!(!s.submit_timed("k".into(), 4));
        s.save(&path).unwrap();
        let (loaded, warn) = Stats::load(&path);
        assert!(warn.is_none());
        assert_eq!(loaded, s);
        let m = loaded.methods["Cross multiplication"];
        assert_eq!((m.attempts, m.correct), (3, 2));
        assert_eq!(m.avg_secs(), Some(2.0));
        assert_eq!(loaded.timed_best["k"], 5);
    }

    #[test]
    fn missing_and_corrupt_files() {
        let dir = tmpdir("corrupt");
        let path = dir.join("stats.json");
        assert_eq!(Stats::load(&path), (Stats::default(), None));
        fs::create_dir_all(&dir).unwrap();
        fs::write(&path, "{not json").unwrap();
        let (s, warn) = Stats::load(&path);
        assert_eq!(s, Stats::default());
        assert!(warn.unwrap().contains("unreadable"));
        assert!(!path.exists());
        assert!(dir.join("stats.json.corrupt").exists());
    }
}
