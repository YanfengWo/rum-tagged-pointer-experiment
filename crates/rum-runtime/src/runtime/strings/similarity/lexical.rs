use super::super::text::remove_diacritics_value;

pub(super) fn pluralize_english_word(word: &str) -> String {
    let lower = word.to_ascii_lowercase();
    if lower.ends_with("ch")
        || lower.ends_with("sh")
        || lower.ends_with('s')
        || lower.ends_with('x')
        || lower.ends_with('z')
    {
        return format!("{word}es");
    }

    if lower.ends_with('y') {
        let mut chars = word.chars().collect::<Vec<_>>();
        if chars.len() >= 2 {
            let previous = chars[chars.len() - 2].to_ascii_lowercase();
            if !matches!(previous, 'a' | 'e' | 'i' | 'o' | 'u') {
                chars.pop();
                return format!("{}ies", chars.into_iter().collect::<String>());
            }
        }
    }

    format!("{word}s")
}

pub(super) fn english_alphabet() -> Vec<String> {
    ('a'..='z').map(|ch| ch.to_string()).collect()
}

pub(super) fn alphabetic_sort_key(value: &str) -> Vec<(usize, String)> {
    value
        .chars()
        .map(|ch| {
            let folded = remove_diacritics_value(&ch.to_string()).to_lowercase();
            let primary = folded
                .chars()
                .next()
                .and_then(english_letter_index)
                .unwrap_or(usize::MAX);
            (primary, ch.to_lowercase().to_string())
        })
        .collect()
}

pub(super) fn english_letter_index(ch: char) -> Option<usize> {
    ch.is_ascii_lowercase().then(|| (ch as u8 - b'a') as usize)
}
