pub(crate) fn completion_prefix(source: &str, cursor_offset: usize) -> (usize, usize, String) {
    let cursor_offset = clamp_char_boundary(source, cursor_offset);
    let replacement_start = scan_symbol_left(source, cursor_offset);
    let replacement_end = scan_symbol_right(source, cursor_offset);
    let prefix = source[replacement_start..cursor_offset].to_string();

    (replacement_start, replacement_end, prefix)
}

pub(crate) fn symbol_at_cursor(source: &str, cursor_offset: usize) -> Option<String> {
    let cursor_offset = clamp_char_boundary(source, cursor_offset);
    let start = scan_symbol_left(source, cursor_offset);
    let end = scan_symbol_right(source, cursor_offset);

    (start < end).then(|| source[start..end].to_string())
}

fn scan_symbol_left(source: &str, cursor_offset: usize) -> usize {
    let mut start = cursor_offset;

    while let Some((previous_start, ch)) = previous_char(source, start) {
        if is_symbol_char(ch) {
            start = previous_start;
        } else {
            break;
        }
    }

    start
}

fn scan_symbol_right(source: &str, cursor_offset: usize) -> usize {
    let mut end = cursor_offset;

    while end < source.len() {
        let ch = source[end..]
            .chars()
            .next()
            .expect("end is always on a char boundary");

        if is_symbol_char(ch) {
            end += ch.len_utf8();
        } else {
            break;
        }
    }

    end
}

fn previous_char(source: &str, end: usize) -> Option<(usize, char)> {
    if end == 0 {
        return None;
    }

    let mut start = end - 1;
    while !source.is_char_boundary(start) {
        start -= 1;
    }

    source[start..end].chars().next().map(|ch| (start, ch))
}

fn clamp_char_boundary(source: &str, offset: usize) -> usize {
    let mut offset = offset.min(source.len());

    while offset > 0 && !source.is_char_boundary(offset) {
        offset -= 1;
    }

    offset
}

fn is_symbol_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '$' | '`')
}
