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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn json_string_escapes_json_control_characters() {
        assert_eq!(
            json_string("quote\" slash\\ newline\n carriage\r tab\t bell\u{0007}"),
            "quote\\\" slash\\\\ newline\\n carriage\\r tab\\t bell\\u0007"
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
