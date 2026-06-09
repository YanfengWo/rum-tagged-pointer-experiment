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

pub fn curated_wolfram_core_dir() -> PathBuf {
    crate_root().join("cases/wolfram/core")
}

pub fn stateful_dir() -> PathBuf {
    crate_root().join("cases/stateful")
}

pub fn lane_a_default_fixture_paths() -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = text_files_in(golden_dir())?;
    files.extend(text_files_in(curated_wolfram_core_dir())?);
    files.sort();
    Ok(files)
}

pub fn lane_b_default_fixture_paths() -> Result<Vec<PathBuf>, std::io::Error> {
    text_files_in(stateful_dir())
}

fn text_files_in(dir: PathBuf) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = fs::read_dir(dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .filter(|entry| {
            entry.as_ref().map_or(true, |path| {
                path.extension().and_then(|ext| ext.to_str()) == Some("txt")
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    files.sort();
    Ok(files)
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

        let files = text_files_in(golden_dir()).unwrap();

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
        let files = text_files_in(curated_wolfram_core_dir()).unwrap();

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
    fn lane_a_default_fixture_paths_include_seed_and_curated_cases() {
        let files = lane_a_default_fixture_paths().unwrap();

        assert_eq!(files.len(), 34);
        assert!(
            files
                .iter()
                .any(|path| path.ends_with("golden/arithmetic.txt"))
        );
        assert!(
            files
                .iter()
                .any(|path| path.ends_with("cases/wolfram/core/smoke_arithmetic.txt"))
        );
        assert!(
            files
                .iter()
                .any(|path| path.ends_with("cases/wolfram/core/options_surface.txt"))
        );
        assert!(
            files
                .iter()
                .any(|path| path.ends_with("cases/wolfram/core/context_namespace.txt"))
        );
        assert!(
            files
                .iter()
                .any(|path| path.ends_with("cases/wolfram/core/messages_attributes.txt"))
        );
        assert!(
            files
                .iter()
                .any(|path| path.ends_with("cases/wolfram/core/scoping_closures.txt"))
        );
        assert!(
            files
                .iter()
                .any(|path| path.ends_with("cases/wolfram/core/exact_arithmetic.txt"))
        );
        assert!(
            files
                .iter()
                .any(|path| path.ends_with("cases/wolfram/core/polynomial_rational.txt"))
        );
        assert!(
            files
                .iter()
                .any(|path| path.ends_with("cases/wolfram/core/predicate_surface.txt"))
        );
        assert!(
            files
                .iter()
                .any(|path| path.ends_with("cases/wolfram/core/structure_selectors.txt"))
        );
        assert!(
            files
                .iter()
                .any(|path| path.ends_with("cases/wolfram/core/threading.txt"))
        );
        assert!(
            files
                .iter()
                .any(|path| path.ends_with("cases/wolfram/core/truth_comparison.txt"))
        );
    }

    #[test]
    fn parses_curated_stateful_fixtures() {
        let files = lane_b_default_fixture_paths().unwrap();

        assert!(
            !files.is_empty(),
            "stateful fixture directory should contain at least one fixture"
        );
        assert!(
            files
                .iter()
                .any(|path| path.ends_with("cases/stateful/scoping_control.txt")),
            "Lane B default fixtures should include stateful scoping/control coverage"
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
