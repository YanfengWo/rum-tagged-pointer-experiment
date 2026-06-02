use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GoldenEntry {
    Normal {
        input: String,
        expected: String,
    },
    KnownDifference {
        input: String,
        metadata: BTreeMap<String, String>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpressionSuiteEntry {
    pub category: Option<String>,
    pub threshold: Option<String>,
    pub expr: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FixtureParseError {
    InvalidGoldenLine(String),
    InvalidKnownDifferenceLine(String),
    MissingKnownDifferenceField { line: String, field: &'static str },
}

impl fmt::Display for FixtureParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidGoldenLine(line) => write!(f, "invalid golden fixture line: {line}"),
            Self::InvalidKnownDifferenceLine(line) => {
                write!(f, "invalid known-difference fixture line: {line}")
            }
            Self::MissingKnownDifferenceField { line, field } => {
                write!(f, "known-difference line is missing `{field}`: {line}")
            }
        }
    }
}

impl std::error::Error for FixtureParseError {}

pub fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn seed_root() -> PathBuf {
    crate_root().join("seed/ham-import")
}

pub fn golden_dir() -> PathBuf {
    seed_root().join("golden")
}

pub fn stateful_dir() -> PathBuf {
    crate_root().join("cases/stateful")
}

pub fn parse_golden_file(text: &str) -> Result<Vec<GoldenEntry>, FixtureParseError> {
    let mut entries = Vec::new();

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((input, expected)) = line.split_once(" ||| ") {
            entries.push(GoldenEntry::Normal {
                input: input.trim().to_owned(),
                expected: expected.trim().to_owned(),
            });
            continue;
        }

        if let Some((input, rest)) = line.split_once(" !!! ") {
            let metadata = parse_metadata_fields(rest, line)?;
            for required in ["id", "type", "reason"] {
                if !metadata.contains_key(required) {
                    return Err(FixtureParseError::MissingKnownDifferenceField {
                        line: line.to_owned(),
                        field: required,
                    });
                }
            }

            entries.push(GoldenEntry::KnownDifference {
                input: input.trim().to_owned(),
                metadata,
            });
            continue;
        }

        return Err(FixtureParseError::InvalidGoldenLine(line.to_owned()));
    }

    Ok(entries)
}

pub fn parse_metadata_fields(
    text: &str,
    full_line: &str,
) -> Result<BTreeMap<String, String>, FixtureParseError> {
    let mut metadata = BTreeMap::new();

    for field in text.split(" | ") {
        let Some((key, value)) = field.split_once('=') else {
            return Err(FixtureParseError::InvalidKnownDifferenceLine(
                full_line.to_owned(),
            ));
        };
        metadata.insert(key.trim().to_owned(), value.trim().to_owned());
    }

    Ok(metadata)
}

pub fn parse_expression_suite(text: &str) -> Vec<ExpressionSuiteEntry> {
    let mut entries = Vec::new();
    let mut current_category = None;
    let mut current_threshold = None;

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') && !line.starts_with("# @") {
            continue;
        }

        if let Some(value) = line.strip_prefix("# @category:") {
            current_category = Some(value.trim().to_owned());
            continue;
        }

        if let Some(value) = line.strip_prefix("# @threshold:") {
            current_threshold = Some(value.trim().to_owned());
            continue;
        }

        entries.push(ExpressionSuiteEntry {
            category: current_category.clone(),
            threshold: current_threshold.clone(),
            expr: line.to_owned(),
        });
    }

    entries
}

pub fn load_text(path: impl AsRef<Path>) -> std::io::Result<String> {
    fs::read_to_string(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_imported_golden_seed_pack() {
        let mut total = 0usize;
        let mut normal = 0usize;
        let mut known_diffs = 0usize;

        let mut files = fs::read_dir(golden_dir())
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();
        files.sort();

        assert_eq!(files.len(), 9);

        for file in files {
            let text = load_text(&file).unwrap();
            let entries = parse_golden_file(&text).unwrap();
            total += entries.len();
            for entry in entries {
                match entry {
                    GoldenEntry::Normal { .. } => normal += 1,
                    GoldenEntry::KnownDifference { .. } => known_diffs += 1,
                }
            }
        }

        assert_eq!(normal, 166);
        assert_eq!(known_diffs, 3);
        assert_eq!(total, 169);
    }

    #[test]
    fn parses_imported_timing_and_memory_seed_suites() {
        let timing =
            parse_expression_suite(&load_text(seed_root().join("bench/timing_suite.txt")).unwrap());
        let memory =
            parse_expression_suite(&load_text(seed_root().join("mem/memory_suite.txt")).unwrap());

        assert_eq!(timing.len(), 20);
        assert_eq!(memory.len(), 13);
        assert_eq!(timing[0].category.as_deref(), Some("trivial"));
        assert_eq!(memory[0].category.as_deref(), Some("minimal"));
    }

    #[test]
    fn parses_curated_lane_a_smoke_fixtures() {
        let core_dir = crate_root().join("cases/wolfram/core");
        let mut files = fs::read_dir(core_dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("txt"))
            .collect::<Vec<_>>();
        files.sort();

        assert!(files.len() >= 4);

        for file in files {
            let text = load_text(&file).unwrap();
            let entries = parse_golden_file(&text).unwrap();
            assert!(
                !entries.is_empty(),
                "curated fixture should not be empty: {}",
                file.display()
            );
        }
    }

    #[test]
    fn parses_curated_stateful_fixtures() {
        let mut files = fs::read_dir(stateful_dir())
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("txt"))
            .collect::<Vec<_>>();
        files.sort();

        assert!(
            !files.is_empty(),
            "stateful fixture directory should contain at least one fixture"
        );

        for file in files {
            let text = load_text(&file).unwrap();
            let entries = parse_golden_file(&text).unwrap();
            assert!(
                !entries.is_empty(),
                "stateful fixture should not be empty: {}",
                file.display()
            );
        }
    }
}
