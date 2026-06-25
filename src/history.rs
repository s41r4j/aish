use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct HistoryEntry {
    pub request: String,
    pub command: String,
    pub risk: String,
    pub exit_code: Option<i32>,
}

pub struct HistoryStore {
    path: PathBuf,
}

impl HistoryStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn append(&self, entry: &HistoryEntry) -> Result<(), String> {
        ensure_parent(&self.path)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|err| err.to_string())?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);

        let exit_code = entry
            .exit_code
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string());

        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}",
            timestamp,
            escape(&entry.risk),
            escape(&exit_code),
            escape(&entry.request),
            escape(&entry.command)
        )
        .map_err(|err| err.to_string())
    }

    pub fn recent(&self, limit: usize) -> Vec<HistoryEntry> {
        let Ok(text) = fs::read_to_string(&self.path) else {
            return Vec::new();
        };

        let mut entries = Vec::new();
        for line in text.lines().rev().take(limit) {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 5 {
                continue;
            }

            entries.push(HistoryEntry {
                risk: unescape(parts[1]),
                exit_code: parts[2].parse().ok(),
                request: unescape(parts[3]),
                command: unescape(parts[4]),
            });
        }

        entries.reverse();
        entries
    }
}

fn ensure_parent(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    Ok(())
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
}

fn unescape(value: &str) -> String {
    let mut result = String::new();
    let mut chars = value.chars();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('t') => result.push('\t'),
                Some('n') => result.push('\n'),
                Some('\\') => result.push('\\'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(ch);
        }
    }

    result
}
