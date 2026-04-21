use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;

const SENSITIVE_PATTERNS: &[&str] = &[
    "SECRET", "TOKEN", "PASSWORD", "PASSWD", "KEY", "PRIVATE",
    "CREDENTIAL", "AUTH", "API_KEY", "APIKEY", "ACCESS",
];

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EnvEntry {
    pub key: String,
    pub value: String,
    pub line_number: usize,
    pub is_comment: bool,
    pub is_empty: bool,
    pub is_duplicate: bool,
    pub has_format_error: bool,
    pub is_sensitive: bool,
    pub expands_variables: bool,
}

impl EnvEntry {
    fn blank(line: usize) -> Self {
        Self {
            key: String::new(), value: String::new(), line_number: line,
            is_comment: false, is_empty: false, is_duplicate: false,
            has_format_error: false, is_sensitive: false, expands_variables: false,
        }
    }
}

pub fn parse(filepath: &str) -> Result<Vec<EnvEntry>, String> {
    let content = fs::read_to_string(filepath).map_err(|e| e.to_string())?;
    parse_content(&content)
}

pub fn parse_content(content: &str) -> Result<Vec<EnvEntry>, String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut seen: HashSet<String> = HashSet::new();
    let mut entries: Vec<EnvEntry> = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if line.is_empty() {
            entries.push(EnvEntry { is_empty: true, ..EnvEntry::blank(i + 1) });
            i += 1;
            continue;
        }

        if line.starts_with('#') {
            entries.push(EnvEntry { key: line.into(), is_comment: true, ..EnvEntry::blank(i + 1) });
            i += 1;
            continue;
        }

        match line.split_once('=') {
            Some((raw_key, raw_val)) => {
                let key = raw_key.trim().to_string();
                let trimmed = raw_val.trim();

                let (value, consumed) = if has_unclosed_quote(trimmed) {
                    collect_multiline(&lines, i, trimmed)
                } else {
                    (strip_quotes(trimmed), 1)
                };

                entries.push(EnvEntry {
                    is_sensitive: is_sensitive(&key),
                    expands_variables: value.contains("${"),
                    is_duplicate: !seen.insert(key.clone()),
                    has_format_error: key.is_empty() || key.contains(' '),
                    key, value, line_number: i + 1,
                    ..EnvEntry::blank(0)
                });

                i += consumed;
            }
            None => {
                entries.push(EnvEntry {
                    key: line.into(), has_format_error: true, ..EnvEntry::blank(i + 1)
                });
                i += 1;
            }
        }
    }

    Ok(entries)
}

fn has_unclosed_quote(val: &str) -> bool {
    if val.is_empty() { return false; }
    let q = val.as_bytes()[0];
    (q == b'"' || q == b'\'') && !val[1..].ends_with(q as char)
}

fn collect_multiline(lines: &[&str], start: usize, fragment: &str) -> (String, usize) {
    let q = fragment.as_bytes()[0] as char;
    let mut buf = String::from(&fragment[1..]);
    let mut consumed = 1;

    for line in &lines[start + 1..] {
        consumed += 1;
        buf.push('\n');
        if line.trim_end().ends_with(q) {
            let t = line.trim_end();
            buf.push_str(&t[..t.len() - 1]);
            break;
        }
        buf.push_str(line);
    }

    (buf, consumed)
}

fn strip_quotes(val: &str) -> String {
    if val.len() >= 2 {
        let b = val.as_bytes();
        if (b[0] == b'"' && b[b.len() - 1] == b'"') || (b[0] == b'\'' && b[b.len() - 1] == b'\'') {
            return val[1..val.len() - 1].to_string();
        }
    }
    val.to_string()
}

fn is_sensitive(key: &str) -> bool {
    let upper = key.to_uppercase();
    SENSITIVE_PATTERNS.iter().any(|p| upper.contains(p))
}
