use super::YamlParseError;

#[derive(Clone, Copy)]
pub(super) struct Line<'a> {
    pub(super) content_start: usize,
    pub(super) indent: usize,
    pub(super) raw: &'a str,
    pub(super) text: &'a str,
}

impl Line<'_> {
    pub(super) fn is_blank(self) -> bool {
        self.text.is_empty()
    }

    pub(super) fn is_comment(self) -> bool {
        self.text.starts_with('#')
    }
}

pub(super) fn scan_lines(source: &str) -> Result<Vec<Line<'_>>, YamlParseError> {
    let mut lines = Vec::new();
    let mut offset = 0_usize;

    for piece in source.split_inclusive('\n') {
        let line_start = offset;
        offset += piece.len();

        let mut end = piece.len();
        if piece.ends_with('\n') {
            end -= 1;
            if end > 0 && piece.as_bytes()[end - 1] == b'\r' {
                end -= 1;
            }
        }

        let raw = &piece[..end];
        let mut indent = 0_usize;
        for (index, byte) in raw.bytes().enumerate() {
            match byte {
                b' ' => indent += 1,
                b'\t' => {
                    return Err(YamlParseError::new(
                        line_start + index,
                        "tab indentation is unsupported; use spaces only",
                    ));
                }
                _ => break,
            }
        }

        let text = &raw[indent..];
        if !text.is_empty() && !text.starts_with('#') {
            validate_control_line(text, line_start + indent)?;
        }

        lines.push(Line {
            content_start: line_start + indent,
            indent,
            raw,
            text,
        });
    }

    if source.is_empty() {
        return Ok(lines);
    }
    if !source.ends_with('\n') && offset < source.len() {
        unreachable!("split_inclusive should account for every source byte");
    }
    Ok(lines)
}

fn validate_control_line(text: &str, offset: usize) -> Result<(), YamlParseError> {
    if text.starts_with('%') {
        return Err(YamlParseError::new(
            offset,
            "YAML directives are unsupported",
        ));
    }
    if starts_document_marker(text, "---") || starts_document_marker(text, "...") {
        return Err(YamlParseError::new(
            offset,
            "YAML document markers are unsupported; FromYaml accepts a single document body",
        ));
    }
    Ok(())
}

fn starts_document_marker(text: &str, marker: &str) -> bool {
    let bytes = text.as_bytes();
    text.starts_with(marker)
        && (bytes.len() == marker.len() || bytes[marker.len()].is_ascii_whitespace())
}
