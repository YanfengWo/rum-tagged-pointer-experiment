use std::collections::BTreeMap;
use std::fmt;

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
