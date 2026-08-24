use super::*;
use rum_syntax::{ScriptFormsError, parse_script_forms};

pub(super) fn normalize_context_name(context: impl Into<String>) -> String {
    let mut context = context.into();

    if !context.ends_with('`') {
        context.push('`');
    }

    context
}

pub(super) fn expand_context_spec(spec: &str, current_context: &str) -> String {
    if let Some(suffix) = spec.strip_prefix('`') {
        normalize_context_name(format!("{current_context}{suffix}"))
    } else {
        normalize_context_name(spec)
    }
}

pub(super) fn prepend_context(context: &str, path: &[String]) -> Vec<String> {
    let mut result = Vec::with_capacity(path.len() + 1);
    result.push(normalize_context_name(context));

    for existing in path {
        if existing != context {
            result.push(existing.clone());
        }
    }

    result
}

pub(super) fn dedup_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut deduped = Vec::new();

    for path in paths {
        if !deduped.iter().any(|existing| existing == &path) {
            deduped.push(path);
        }
    }

    deduped
}

pub(super) fn normalize_found_path(path: &Path) -> PathBuf {
    strip_windows_verbatim_prefix(if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    })
}

pub(super) fn strip_windows_verbatim_prefix(path: PathBuf) -> PathBuf {
    let rendered = path.display().to_string();

    rendered
        .strip_prefix("\\\\?\\")
        .map(PathBuf::from)
        .unwrap_or(path)
}

pub(super) fn package_candidates_for_context(context: &str) -> Vec<PathBuf> {
    let trimmed = context.trim_end_matches('`');
    let segments = trimmed
        .split('`')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    let Some((leaf, parents)) = segments.split_last() else {
        return Vec::new();
    };

    package_candidates_for_segments(parents, leaf)
}

pub(super) fn package_candidates_for_segments(parents: &[&str], leaf: &str) -> Vec<PathBuf> {
    let base_dir = parents.iter().fold(PathBuf::new(), |mut path, segment| {
        path.push(segment);
        path
    });
    let mut candidates = Vec::new();

    for extension in ["rum", "wl", "m"] {
        candidates.push(base_dir.join(format!("{leaf}.{extension}")));
        candidates.push(
            base_dir
                .join(leaf)
                .join("Kernel")
                .join(format!("init.{extension}")),
        );
        candidates.push(base_dir.join(leaf).join(format!("init.{extension}")));
    }

    candidates
}

pub(super) fn split_script_into_forms(source: &str) -> Result<Vec<String>, EvalError> {
    // Dual path, deliberately (not an oversight): a source whose whole-source
    // lex succeeds takes the linear single-lex/single-parse splitter in
    // `rum-syntax`; a source whose lex fails (unterminated string or comment,
    // stray character) falls back to the legacy line-driven loop below,
    // because that loop evaluates the earlier forms before surfacing the
    // tail error — an ordering pinned by the loader rollback tests. Clean
    // sources, including every wide multi-line data literal that motivates
    // the linear path, never reach the fallback.
    match parse_script_forms(source) {
        Ok(forms) => Ok(forms),
        Err(ScriptFormsError::IncompleteTail(err)) => Err(err.into()),
        Err(ScriptFormsError::LexFailed(_)) => split_script_into_forms_line_driven(source),
    }
}

/// Legacy line-driven splitter: accumulate lines, flush at the first line
/// whose accumulated parse succeeds or fails hard, and report an incomplete
/// tail. Retained as the lex-failure fallback above and as the
/// differential-test oracle for `parse_script_forms`.
pub(super) fn split_script_into_forms_line_driven(source: &str) -> Result<Vec<String>, EvalError> {
    let mut forms = Vec::new();
    let mut current = String::new();

    for raw_line in source.lines() {
        if current.is_empty() && raw_line.trim().is_empty() {
            continue;
        }

        if !current.is_empty() {
            current.push('\n');
        }

        current.push_str(raw_line);

        if current.trim().is_empty() {
            current.clear();
            continue;
        }

        match parse_expr(&current) {
            Ok(_) => {
                forms.push(current.trim().to_string());
                current.clear();
            }
            Err(SyntaxError::EmptyInput) => {
                current.clear();
            }
            Err(err) if err.is_incomplete() => {}
            Err(_) => {
                forms.push(current.trim().to_string());
                current.clear();
            }
        }
    }

    if !current.trim().is_empty() {
        match parse_expr(&current) {
            Ok(_) => {
                forms.push(current.trim().to_string());
            }
            Err(SyntaxError::EmptyInput) => {}
            Err(err) if err.is_incomplete() => return Err(err.into()),
            Err(_) => {
                forms.push(current.trim().to_string());
            }
        }
    }

    Ok(forms)
}
