use std::fs;
use std::path::Path;

pub const SUMMARY_SCHEMA_VERSION: usize = 1;

pub fn write_json_file(path: &Path, contents: String) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(path, contents)?;
    Ok(())
}

pub fn parse_max_issue_samples(value: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let max_issue_samples = value
        .parse::<usize>()
        .map_err(|_| format!("--max-issue-samples expects a positive integer, got `{value}`"))?;
    if max_issue_samples == 0 {
        return Err("--max-issue-samples expects a positive integer, got `0`".into());
    }
    Ok(max_issue_samples)
}

pub fn json_string(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => escaped.push(ch),
        }
    }
    escaped
}

pub fn json_string_option(value: Option<&str>) -> String {
    value
        .map(|value| format!("\"{}\"", json_string(value)))
        .unwrap_or_else(|| "null".to_string())
}

pub fn json_usize_option(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_string())
}

pub fn json_path_option(value: Option<&Path>) -> String {
    value
        .map(|value| format!("\"{}\"", json_string(&value.display().to_string())))
        .unwrap_or_else(|| "null".to_string())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn parse_max_issue_samples_accepts_positive_integers() {
        assert_eq!(parse_max_issue_samples("1").unwrap(), 1);
        assert_eq!(parse_max_issue_samples("25").unwrap(), 25);
    }

    #[test]
    fn parse_max_issue_samples_rejects_zero_and_non_numbers() {
        assert_eq!(
            parse_max_issue_samples("0").unwrap_err().to_string(),
            "--max-issue-samples expects a positive integer, got `0`"
        );
        assert_eq!(
            parse_max_issue_samples("many").unwrap_err().to_string(),
            "--max-issue-samples expects a positive integer, got `many`"
        );
    }

    #[test]
    fn json_string_escapes_json_control_characters() {
        assert_eq!(
            json_string("quote\" slash\\ newline\n carriage\r tab\t bell\u{0007}"),
            "quote\\\" slash\\\\ newline\\n carriage\\r tab\\t bell\\u0007"
        );
    }

    #[test]
    fn json_option_helpers_render_null_or_values() {
        assert_eq!(json_string_option(None), "null");
        assert_eq!(json_string_option(Some("a\"b")), "\"a\\\"b\"");
        assert_eq!(json_usize_option(None), "null");
        assert_eq!(json_usize_option(Some(42)), "42");
        assert_eq!(json_path_option(None), "null");
        assert_eq!(
            json_path_option(Some(Path::new("fixtures/a\"b.txt"))),
            "\"fixtures/a\\\"b.txt\""
        );
    }

    #[test]
    fn write_json_file_creates_parent_directories() {
        let root = unique_temp_root("rum-difftest-json-cli");
        let path = root.join("nested/summary.json");

        write_json_file(&path, "{\"clean\": true}\n".to_string()).unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "{\"clean\": true}\n");
        fs::remove_dir_all(root).unwrap();
    }

    fn unique_temp_root(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}"))
    }
}
