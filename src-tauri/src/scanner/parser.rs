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

pub fn parse(filepath: &str) -> Result<Vec<EnvEntry>, String> {
    let content = fs::read_to_string(filepath).map_err(|e| e.to_string())?;
    parse_content(&content)
}

pub fn parse_content(content: &str) -> Result<Vec<EnvEntry>, String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut seen_keys: HashSet<String> = HashSet::new();
    let mut entries: Vec<EnvEntry> = Vec::new();
    let mut idx = 0;

    while idx < lines.len() {
        let line = lines[idx].trim();

        if line.is_empty() {
            entries.push(empty_entry(idx + 1));
            idx += 1;
            continue;
        }

        if line.starts_with('#') {
            entries.push(comment_entry(line, idx + 1));
            idx += 1;
            continue;
        }

        let start_line = idx + 1;

        match line.split_once('=') {
            Some((raw_key, raw_value)) => {
                let key = raw_key.trim().to_string();
                let trimmed = raw_value.trim();

                let (value, consumed) = if starts_with_unclosed_quote(trimmed) {
                    collect_multiline(&lines, idx, trimmed)
                } else {
                    (strip_quotes(trimmed), 1)
                };

                let is_duplicate = !seen_keys.insert(key.clone());
                let has_format_error = key.is_empty() || key.contains(' ');

                entries.push(EnvEntry {
                    is_sensitive: detect_sensitive(&key),
                    expands_variables: detect_variable_expansion(&value),
                    key,
                    value,
                    line_number: start_line,
                    is_comment: false,
                    is_empty: false,
                    is_duplicate,
                    has_format_error,
                });

                idx += consumed;
            }
            None => {
                entries.push(EnvEntry {
                    key: line.to_string(),
                    value: String::new(),
                    line_number: start_line,
                    is_comment: false,
                    is_empty: false,
                    is_duplicate: false,
                    has_format_error: true,
                    is_sensitive: false,
                    expands_variables: false,
                });
                idx += 1;
            }
        }
    }

    Ok(entries)
}

fn starts_with_unclosed_quote(val: &str) -> bool {
    if val.is_empty() {
        return false;
    }
    let quote = val.as_bytes()[0];
    if quote != b'"' && quote != b'\'' {
        return false;
    }
    !val[1..].ends_with(quote as char)
}

fn collect_multiline(lines: &[&str], start: usize, first_fragment: &str) -> (String, usize) {
    let quote = first_fragment.as_bytes()[0] as char;
    let mut buf = String::from(&first_fragment[1..]);
    let mut consumed = 1;

    for line in &lines[start + 1..] {
        consumed += 1;
        if line.trim_end().ends_with(quote) {
            let trimmed = line.trim_end();
            buf.push('\n');
            buf.push_str(&trimmed[..trimmed.len() - 1]);
            break;
        }
        buf.push('\n');
        buf.push_str(line);
    }

    (buf, consumed)
}

fn strip_quotes(val: &str) -> String {
    if val.len() >= 2 {
        let bytes = val.as_bytes();
        if (bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\'')
        {
            return val[1..val.len() - 1].to_string();
        }
    }
    val.to_string()
}

fn detect_sensitive(key: &str) -> bool {
    let upper = key.to_uppercase();
    SENSITIVE_PATTERNS.iter().any(|p| upper.contains(p))
}

fn detect_variable_expansion(value: &str) -> bool {
    value.contains("${")
}

fn empty_entry(line: usize) -> EnvEntry {
    EnvEntry {
        key: String::new(),
        value: String::new(),
        line_number: line,
        is_comment: false,
        is_empty: true,
        is_duplicate: false,
        has_format_error: false,
        is_sensitive: false,
        expands_variables: false,
    }
}

fn comment_entry(text: &str, line: usize) -> EnvEntry {
    EnvEntry {
        key: text.to_string(),
        value: String::new(),
        line_number: line,
        is_comment: true,
        is_empty: false,
        is_duplicate: false,
        has_format_error: false,
        is_sensitive: false,
        expands_variables: false,
    }
}
