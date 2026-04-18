use super::super::text::remove_diacritics_value;

pub(super) fn pluralize_english_word(word: &str) -> String {
    let lower = word.to_ascii_lowercase();
    if let Some(irregular) = english_irregular_plural(&lower) {
        return apply_plural_case(word, irregular);
    }

    let plural_lower = if lower.ends_with("ch")
        || lower.ends_with("sh")
        || lower.ends_with('s')
        || lower.ends_with('x')
    {
        format!("{lower}es")
    } else if lower.ends_with('z') {
        if lower == "quiz" {
            "quizzes".to_string()
        } else {
            format!("{lower}es")
        }
    } else if lower.ends_with('y') {
        let chars = lower.chars().collect::<Vec<_>>();
        if chars.len() >= 2 {
            let previous = chars[chars.len() - 2];
            if !matches!(previous, 'a' | 'e' | 'i' | 'o' | 'u') {
                format!("{}ies", &lower[..lower.len() - 1])
            } else {
                format!("{lower}s")
            }
        } else {
            format!("{lower}s")
        }
    } else if lower.ends_with("fe") {
        let stem = &lower[..lower.len() - 2];
        if matches!(stem, "kni" | "wi" | "li") {
            format!("{stem}ves")
        } else {
            format!("{lower}s")
        }
    } else if lower.ends_with('f') {
        let stem = &lower[..lower.len() - 1];
        if matches!(stem, "lea" | "wol" | "hal" | "cal" | "sel" | "loa") {
            format!("{stem}ves")
        } else {
            format!("{lower}s")
        }
    } else if lower.ends_with('o')
        && matches!(
            lower.as_str(),
            "hero" | "potato" | "tomato" | "echo" | "torpedo"
        )
    {
        format!("{lower}es")
    } else {
        format!("{lower}s")
    };

    apply_plural_case(word, &plural_lower)
}

fn english_irregular_plural(lower: &str) -> Option<&'static str> {
    match lower {
        "analysis" => Some("analyses"),
        "appendix" => Some("appendices"),
        "axis" => Some("axes"),
        "bacterium" => Some("bacteria"),
        "basis" => Some("bases"),
        "cactus" => Some("cacti"),
        "child" => Some("children"),
        "criterion" => Some("criteria"),
        "crisis" => Some("crises"),
        "datum" => Some("data"),
        "diagnosis" => Some("diagnoses"),
        "man" => Some("men"),
        "matrix" => Some("matrices"),
        "woman" => Some("women"),
        "person" => Some("people"),
        "mouse" => Some("mice"),
        "goose" => Some("geese"),
        "tooth" => Some("teeth"),
        "foot" => Some("feet"),
        "index" => Some("indices"),
        "phenomenon" => Some("phenomena"),
        "sheep" => Some("sheep"),
        "thesis" => Some("theses"),
        "vertex" => Some("vertices"),
        "deer" => Some("deer"),
        _ => None,
    }
}

fn apply_plural_case(original: &str, plural_lower: &str) -> String {
    if original
        .chars()
        .all(|ch| !ch.is_alphabetic() || ch.is_uppercase())
    {
        return plural_lower.to_ascii_uppercase();
    }

    let is_title_case = original.chars().next().is_some_and(char::is_uppercase)
        && original
            .chars()
            .skip(1)
            .all(|ch| !ch.is_alphabetic() || ch.is_lowercase());

    if is_title_case {
        let mut chars = plural_lower.chars();
        let mut rendered = String::new();
        if let Some(first) = chars.next() {
            rendered.extend(first.to_uppercase());
        }
        rendered.extend(chars);
        return rendered;
    }

    plural_lower.to_string()
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
