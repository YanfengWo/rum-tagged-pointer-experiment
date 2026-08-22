use rum_core::Expr;

use super::{YamlParseError, association_expr, list_expr, symbol_value};
use crate::adaptive_set::StringFingerprintIndex;

mod lines;
mod quoted;
mod scalars;

use lines::{Line, scan_lines};
use scalars::{
    Chomping, count_leading_spaces, find_mapping_colon, is_sequence_marker, parse_key,
    parse_literal_header, parse_plain_scalar, parse_quoted_scalar,
};

const MAPPING_KEY_INDEX_THRESHOLD: usize = 16;

enum MappingKeyIndex {
    Linear,
    Indexed(StringFingerprintIndex),
}

impl MappingKeyIndex {
    fn new() -> Self {
        Self::Linear
    }

    fn insert(&mut self, entries: &[(String, Expr)], key: &str) -> bool {
        if matches!(self, Self::Linear) {
            if entries.len() < MAPPING_KEY_INDEX_THRESHOLD {
                return !entries.iter().any(|(existing, _)| existing == key);
            }

            let mut index = StringFingerprintIndex::from_values(
                entries.len() + 1,
                entries.iter().map(|(existing, _)| existing.as_str()),
            );
            let unique = index.insert(entries.iter().map(|(existing, _)| existing.as_str()), key);
            *self = Self::Indexed(index);
            return unique;
        }

        let Self::Indexed(index) = self else {
            unreachable!();
        };
        index.insert(entries.iter().map(|(existing, _)| existing.as_str()), key)
    }
}

pub(super) fn from_yaml(source: &str) -> Result<Expr, YamlParseError> {
    let lines = scan_lines(source)?;
    Parser::new(lines).parse()
}

struct Parser<'a> {
    lines: Vec<Line<'a>>,
    index: usize,
}

impl<'a> Parser<'a> {
    fn new(lines: Vec<Line<'a>>) -> Self {
        Self { lines, index: 0 }
    }

    fn parse(&mut self) -> Result<Expr, YamlParseError> {
        let Some(first) = self.peek_content() else {
            return Ok(symbol_value("Null"));
        };
        if first.indent != 0 {
            return Err(YamlParseError::new(
                first.content_start,
                "top-level YAML content must start at indentation 0",
            ));
        }

        let value = self.parse_block(first.indent)?;
        if let Some(line) = self.peek_content() {
            return Err(YamlParseError::new(
                line.content_start,
                "trailing YAML content after scalar value",
            ));
        }
        Ok(value)
    }

    fn parse_block(&mut self, indent: usize) -> Result<Expr, YamlParseError> {
        let Some(line) = self.peek_content() else {
            return Ok(symbol_value("Null"));
        };
        if line.indent != indent {
            return Err(YamlParseError::new(
                line.content_start,
                "inconsistent indentation",
            ));
        }

        if is_sequence_marker(line.text) {
            return self.parse_sequence(indent);
        }
        if find_mapping_colon(line.text).is_some() {
            return self
                .parse_mapping_entries(indent, Vec::new())
                .map(association_expr);
        }

        self.index += 1;
        self.parse_value_text(line.text, line.content_start, indent)
    }

    fn parse_mapping_entries(
        &mut self,
        indent: usize,
        mut entries: Vec<(String, Expr)>,
    ) -> Result<Vec<(String, Expr)>, YamlParseError> {
        let mut seen_keys = MappingKeyIndex::new();

        loop {
            let Some(line) = self.peek_content() else {
                break;
            };
            if line.indent < indent {
                break;
            }
            if line.indent > indent {
                return Err(YamlParseError::new(
                    line.content_start,
                    "inconsistent indentation",
                ));
            }
            if is_sequence_marker(line.text) {
                return Err(YamlParseError::new(
                    line.content_start,
                    "sequence items and mapping keys cannot appear at the same indentation",
                ));
            }

            let Some(colon) = find_mapping_colon(line.text) else {
                return Err(YamlParseError::new(
                    line.content_start,
                    "expected YAML mapping key",
                ));
            };
            self.index += 1;
            let entry = self.parse_mapping_pair(line.text, line.content_start, colon, indent)?;
            if !seen_keys.insert(&entries, &entry.0) {
                return Err(YamlParseError::new(
                    line.content_start,
                    format!("duplicate mapping key {:?}", entry.0),
                ));
            }
            entries.push(entry);
        }
        Ok(entries)
    }

    fn parse_mapping_pair(
        &mut self,
        text: &str,
        offset: usize,
        colon: usize,
        parent_indent: usize,
    ) -> Result<(String, Expr), YamlParseError> {
        let key = parse_key(&text[..colon], offset)?;
        if key == "<<" {
            return Err(YamlParseError::new(
                offset,
                "YAML merge keys are unsupported",
            ));
        }

        let value_text = &text[colon + 1..];
        let value_offset = offset + colon + 1;
        let value = self.parse_mapping_value(value_text, value_offset, parent_indent)?;
        Ok((key, value))
    }

    fn parse_mapping_value(
        &mut self,
        value_text: &str,
        value_offset: usize,
        parent_indent: usize,
    ) -> Result<Expr, YamlParseError> {
        let leading = count_leading_spaces(value_text);
        let text = &value_text[leading..];
        let offset = value_offset + leading;
        if text.is_empty() || text.starts_with('#') {
            return self.parse_nested_or_null(parent_indent);
        }
        self.parse_value_text(text, offset, parent_indent)
    }

    fn parse_nested_or_null(&mut self, parent_indent: usize) -> Result<Expr, YamlParseError> {
        let Some(line) = self.peek_content() else {
            return Ok(symbol_value("Null"));
        };
        if line.indent <= parent_indent {
            return Ok(symbol_value("Null"));
        }
        self.parse_block(line.indent)
    }

    fn parse_sequence(&mut self, indent: usize) -> Result<Expr, YamlParseError> {
        let mut items = Vec::new();

        loop {
            let Some(line) = self.peek_content() else {
                break;
            };
            if line.indent < indent {
                break;
            }
            if line.indent > indent {
                return Err(YamlParseError::new(
                    line.content_start,
                    "inconsistent indentation",
                ));
            }
            if !is_sequence_marker(line.text) {
                let message = if find_mapping_colon(line.text).is_some() {
                    "sequence items and mapping keys cannot appear at the same indentation"
                } else {
                    "expected YAML sequence item"
                };
                return Err(YamlParseError::new(line.content_start, message));
            }

            self.index += 1;
            items.push(self.parse_sequence_item(line)?);
        }

        Ok(list_expr(items))
    }

    fn parse_sequence_item(&mut self, line: Line<'a>) -> Result<Expr, YamlParseError> {
        let item_start = if line.text == "-" { 1 } else { 2 };
        let item_text = &line.text[item_start..];
        let item_offset = line.content_start + item_start;
        if item_text.is_empty() || item_text.starts_with('#') {
            return self.parse_nested_or_null(line.indent);
        }

        if let Some(colon) = find_mapping_colon(item_text) {
            let logical_indent = line.indent + 2;
            let mut entries =
                vec![self.parse_mapping_pair(item_text, item_offset, colon, logical_indent)?];

            if let Some(next) = self.peek_content() {
                if next.indent > line.indent {
                    if next.indent != logical_indent {
                        return Err(YamlParseError::new(
                            next.content_start,
                            "inconsistent indentation",
                        ));
                    }
                    entries = self.parse_mapping_entries(logical_indent, entries)?;
                }
            }

            return Ok(association_expr(entries));
        }

        self.parse_value_text(item_text, item_offset, line.indent)
    }

    fn parse_value_text(
        &mut self,
        text: &str,
        offset: usize,
        parent_indent: usize,
    ) -> Result<Expr, YamlParseError> {
        if text.is_empty() || text.starts_with('#') {
            return Ok(symbol_value("Null"));
        }

        match text.as_bytes()[0] {
            b'\'' | b'"' => parse_quoted_scalar(text, offset),
            b'|' => self.parse_literal_block(text, offset, parent_indent),
            b'>' => Err(YamlParseError::new(
                offset,
                "folded block scalars are unsupported; use literal | or |-",
            )),
            _ => parse_plain_scalar(text, offset),
        }
    }

    fn parse_literal_block(
        &mut self,
        text: &str,
        offset: usize,
        parent_indent: usize,
    ) -> Result<Expr, YamlParseError> {
        let chomping = parse_literal_header(text, offset)?;
        let mut block_lines = Vec::new();

        while let Some(line) = self.lines.get(self.index).copied() {
            if !line.is_blank() && line.indent <= parent_indent {
                break;
            }
            self.index += 1;
            block_lines.push(line);
        }

        let content_indent = block_lines
            .iter()
            .filter(|line| !line.is_blank())
            .map(|line| line.indent)
            .min()
            .unwrap_or(parent_indent + 1);

        let mut rendered = block_lines
            .into_iter()
            .map(|line| {
                if line.is_blank() {
                    String::new()
                } else {
                    line.raw[content_indent..].to_string()
                }
            })
            .collect::<Vec<_>>();

        while rendered.last().is_some_and(|line| line.is_empty()) {
            rendered.pop();
        }

        if rendered.is_empty() {
            return Ok(Expr::string(""));
        }

        let mut value = rendered.join("\n");
        if chomping == Chomping::Clip {
            value.push('\n');
        }
        Ok(Expr::string(value))
    }

    fn peek_content(&mut self) -> Option<Line<'a>> {
        while let Some(line) = self.lines.get(self.index).copied() {
            if line.is_blank() || line.is_comment() {
                self.index += 1;
                continue;
            }
            return Some(line);
        }
        None
    }
}
