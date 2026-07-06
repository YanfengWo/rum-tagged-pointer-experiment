use super::*;

pub(super) fn expr_as_nested_string_list(expr: &Expr) -> Option<Vec<Vec<String>>> {
    let items = call_items_with_head(expr, &system_expr("List"))?;
    items.iter().map(expr_as_string_list).collect()
}

pub(super) fn single_code_point_string(expr: &Expr) -> Option<char> {
    let value = expr_as_string(expr)?;
    let mut chars = value.chars();
    let ch = chars.next()?;
    chars.next().is_none().then_some(ch)
}

pub(super) fn string_pad_left_value(value: &str, width: usize, padding: &str) -> String {
    let value_len = value.chars().count();
    if value_len >= width {
        return value.to_string();
    }

    let needed = width - value_len;
    let pad_chars = padding.chars().collect::<Vec<_>>();
    let mut prefix = String::new();
    for index in 0..needed {
        prefix.push(pad_chars[index % pad_chars.len()]);
    }
    prefix.push_str(value);
    prefix
}

pub(super) fn string_pad_left_pieces_value(value: &str, width: usize, pieces: &[String]) -> String {
    let value_len = value.chars().count();
    if value_len >= width {
        return value.to_string();
    }

    let mut prefix = padding_from_pieces(width - value_len, pieces);
    prefix.push_str(value);
    prefix
}

pub(super) fn string_pad_right_value(value: &str, width: usize, padding: &str) -> String {
    let value_len = value.chars().count();
    if value_len >= width {
        return value.to_string();
    }

    let needed = width - value_len;
    let pad_chars = padding.chars().collect::<Vec<_>>();
    let mut padded = value.to_string();
    for index in 0..needed {
        padded.push(pad_chars[index % pad_chars.len()]);
    }
    padded
}

pub(super) fn string_pad_right_pieces_value(
    value: &str,
    width: usize,
    pieces: &[String],
) -> String {
    let value_len = value.chars().count();
    if value_len >= width {
        return value.to_string();
    }

    let mut padded = value.to_string();
    padded.push_str(&padding_from_pieces(width - value_len, pieces));
    padded
}

fn padding_from_pieces(width: usize, pieces: &[String]) -> String {
    let mut output = String::new();
    let mut remaining = width;
    let mut index = 0usize;

    while remaining > 0 {
        for ch in pieces[index % pieces.len()].chars() {
            if remaining == 0 {
                break;
            }
            output.push(ch);
            remaining -= 1;
        }
        index += 1;
    }

    output
}

pub(super) fn string_lines_value(value: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut chars = value.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\n' {
            lines.push(std::mem::take(&mut current));
        } else if ch == '\r' && chars.peek() == Some(&'\n') {
            chars.next();
            lines.push(std::mem::take(&mut current));
        } else {
            current.push(ch);
        }
    }

    lines.push(current);
    if value.ends_with('\n') {
        lines.pop();
    }
    lines
}

pub(super) fn text_words_value(value: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();

    for ch in value.chars() {
        if ch.is_alphanumeric() {
            current.push(ch);
        } else if !current.is_empty() {
            words.push(std::mem::take(&mut current));
        }
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}

pub(super) fn character_code_expr(value: &str) -> Expr {
    list_expr(
        value
            .chars()
            .map(|ch| Expr::integer(BigInt::from(ch as u32)))
            .collect(),
    )
}

pub(super) fn character_counts_expr(value: &str) -> Expr {
    let mut counts = BTreeMap::<String, usize>::new();
    for ch in value.chars() {
        *counts.entry(ch.to_string()).or_insert(0) += 1;
    }

    let mut entries = counts.into_iter().collect::<Vec<_>>();
    entries.sort_by(|(lhs_key, lhs_count), (rhs_key, rhs_count)| {
        rhs_count.cmp(lhs_count).then_with(|| rhs_key.cmp(lhs_key))
    });

    render_association(
        entries
            .into_iter()
            .map(|(key, count)| (Expr::string(key), Expr::integer(BigInt::from(count as u64))))
            .collect(),
    )
}
