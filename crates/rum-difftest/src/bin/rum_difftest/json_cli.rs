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

pub fn parse_positive_usize_flag(
    flag: &str,
    value: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| format!("{flag} expects a positive integer, got `{value}`"))?;
    if parsed == 0 {
        return Err(format!("{flag} expects a positive integer, got `0`").into());
    }
    Ok(parsed)
}

pub fn parse_positive_u64_flag(flag: &str, value: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let parsed = value
        .parse::<u64>()
        .map_err(|_| format!("{flag} expects a positive integer, got `{value}`"))?;
    if parsed == 0 {
        return Err(format!("{flag} expects a positive integer, got `0`").into());
    }
    Ok(parsed)
}

pub fn parse_positive_u128_flag(
    flag: &str,
    value: &str,
) -> Result<u128, Box<dyn std::error::Error>> {
    let parsed = value
        .parse::<u128>()
        .map_err(|_| format!("{flag} expects a positive integer, got `{value}`"))?;
    if parsed == 0 {
        return Err(format!("{flag} expects a positive integer, got `0`").into());
    }
    Ok(parsed)
}

pub fn parse_nonnegative_u128_flag(
    flag: &str,
    value: &str,
) -> Result<u128, Box<dyn std::error::Error>> {
    value
        .parse::<u128>()
        .map_err(|_| format!("{flag} expects a non-negative integer, got `{value}`").into())
}

pub fn parse_max_issue_samples(value: &str) -> Result<usize, Box<dyn std::error::Error>> {
    parse_positive_usize_flag("--max-issue-samples", value)
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
    fn positive_integer_flag_helpers_report_flag_specific_errors() {
        assert_eq!(parse_positive_usize_flag("--limit", "3").unwrap(), 3);
        assert_eq!(
            parse_positive_usize_flag("--limit", "0")
                .unwrap_err()
                .to_string(),
            "--limit expects a positive integer, got `0`"
        );
        assert_eq!(
            parse_positive_usize_flag("--limit", "many")
                .unwrap_err()
                .to_string(),
            "--limit expects a positive integer, got `many`"
        );
        assert_eq!(parse_positive_u64_flag("--timeout", "9").unwrap(), 9);
        assert_eq!(
            parse_positive_u64_flag("--timeout", "0")
                .unwrap_err()
                .to_string(),
            "--timeout expects a positive integer, got `0`"
        );
        assert_eq!(
            parse_positive_u64_flag("--timeout", "many")
                .unwrap_err()
                .to_string(),
            "--timeout expects a positive integer, got `many`"
        );
        assert_eq!(parse_positive_u128_flag("--multiplier", "11").unwrap(), 11);
        assert_eq!(
            parse_positive_u128_flag("--multiplier", "0")
                .unwrap_err()
                .to_string(),
            "--multiplier expects a positive integer, got `0`"
        );
        assert_eq!(
            parse_positive_u128_flag("--multiplier", "many")
                .unwrap_err()
                .to_string(),
            "--multiplier expects a positive integer, got `many`"
        );
        assert_eq!(parse_nonnegative_u128_flag("--min", "0").unwrap(), 0);
        assert_eq!(
            parse_nonnegative_u128_flag("--min", "-1")
                .unwrap_err()
                .to_string(),
            "--min expects a non-negative integer, got `-1`"
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
