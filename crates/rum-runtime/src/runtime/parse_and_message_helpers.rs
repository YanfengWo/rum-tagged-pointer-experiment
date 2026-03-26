use super::*;

pub(super) fn parse_thread_arg_spec(expr: &Expr) -> Option<ThreadArgSpec> {
    if is_system_symbol_name(expr, "All") {
        return Some(ThreadArgSpec::All);
    }

    if is_system_symbol_name(expr, "None") {
        return Some(ThreadArgSpec::None);
    }

    if let Some(count) = expr_as_signed_isize(expr) {
        return if count >= 0 {
            usize::try_from(count).ok().map(ThreadArgSpec::Prefix)
        } else {
            count
                .checked_abs()
                .and_then(|value| usize::try_from(value).ok())
                .map(ThreadArgSpec::Suffix)
        };
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "List") {
        return None;
    }

    match args.as_slice() {
        [index] => Some(ThreadArgSpec::Single(expr_as_signed_isize(index)?)),
        [start, end] => Some(ThreadArgSpec::Range {
            start: expr_as_signed_isize(start)?,
            end: expr_as_signed_isize(end)?,
            step: 1,
        }),
        [start, end, step] => Some(ThreadArgSpec::Range {
            start: expr_as_signed_isize(start)?,
            end: expr_as_signed_isize(end)?,
            step: expr_as_signed_isize(step)?,
        }),
        _ => None,
    }
}

pub(super) fn resolve_thread_arg_indices(spec: &ThreadArgSpec, arg_count: usize) -> Vec<usize> {
    match spec {
        ThreadArgSpec::All => (0..arg_count).collect(),
        ThreadArgSpec::None => Vec::new(),
        ThreadArgSpec::Prefix(count) => (0..(*count).min(arg_count)).collect(),
        ThreadArgSpec::Suffix(count) => {
            let start = arg_count.saturating_sub(*count);
            (start..arg_count).collect()
        }
        ThreadArgSpec::Single(index) => normalize_thread_position(*index, arg_count)
            .map(|position| vec![position])
            .unwrap_or_default(),
        ThreadArgSpec::Range { start, end, step } => {
            resolve_thread_range_indices(*start, *end, *step, arg_count)
        }
    }
}

pub(super) fn resolve_thread_range_indices(
    start: isize,
    end: isize,
    step: isize,
    arg_count: usize,
) -> Vec<usize> {
    if step == 0 || arg_count == 0 {
        return Vec::new();
    }

    let Some(start_index) = clamp_thread_position(start, arg_count) else {
        return Vec::new();
    };
    let Some(end_index) = clamp_thread_position(end, arg_count) else {
        return Vec::new();
    };

    let ascending = step > 0;
    let step = step.unsigned_abs();
    let mut indices = Vec::new();

    if ascending {
        if start_index > end_index {
            return indices;
        }

        let mut current = start_index;
        while current <= end_index {
            indices.push(current);
            current = current.saturating_add(step);
            if current == usize::MAX {
                break;
            }
        }
    } else {
        if start_index < end_index {
            return indices;
        }

        let mut current = start_index;
        while current >= end_index {
            indices.push(current);
            let next = current.saturating_sub(step);
            if next == current {
                break;
            }
            current = next;
        }
    }

    indices
}

pub(super) fn normalize_thread_position(position: isize, arg_count: usize) -> Option<usize> {
    if arg_count == 0 || position == 0 {
        return None;
    }

    if position > 0 {
        let zero_based = usize::try_from(position).ok()?.checked_sub(1)?;
        (zero_based < arg_count).then_some(zero_based)
    } else {
        let from_end = usize::try_from(position.checked_abs()?).ok()?;
        (from_end <= arg_count).then_some(arg_count - from_end)
    }
}

pub(super) fn clamp_thread_position(position: isize, arg_count: usize) -> Option<usize> {
    if arg_count == 0 || position == 0 {
        return None;
    }

    if position > 0 {
        let one_based = usize::try_from(position).ok()?;
        Some(one_based.clamp(1, arg_count) - 1)
    } else {
        let from_end = usize::try_from(position.checked_abs()?).ok()?;
        Some(arg_count.saturating_sub(from_end.min(arg_count)))
    }
}

pub(super) fn parse_standard_level_spec(expr: &Expr) -> Option<CasesLevelSpec> {
    if let Some(level) = expr_as_signed_isize(expr) {
        return Some(match level.cmp(&0) {
            std::cmp::Ordering::Greater => CasesLevelSpec::range(1, Some(level)),
            std::cmp::Ordering::Equal => CasesLevelSpec::exact(0),
            std::cmp::Ordering::Less => CasesLevelSpec::range(level, Some(-1)),
        });
    }

    if is_system_symbol_name(expr, "Infinity") {
        return Some(CasesLevelSpec::range(1, None));
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "List") {
        return None;
    }

    match args.as_slice() {
        [level] => Some(CasesLevelSpec::exact(expr_as_signed_isize(level)?)),
        [min, max] => parse_standard_level_range(min, max),
        _ => None,
    }
}

fn parse_standard_level_range(min: &Expr, max: &Expr) -> Option<CasesLevelSpec> {
    let min = expr_as_signed_isize(min)?;

    if let Some(max) = expr_as_signed_isize(max) {
        (min <= max).then_some(CasesLevelSpec::range(min, Some(max)))
    } else if is_system_symbol_name(max, "Infinity") {
        Some(CasesLevelSpec::range(min, None))
    } else {
        None
    }
}

pub(super) fn parse_non_negative_match_limit(expr: &Expr) -> Option<usize> {
    expr_as_non_negative_usize(expr)
}

pub(super) fn parse_heads_option(expr: &Expr) -> Option<bool> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return None;
    }

    let symbol = expr_as_symbol(&args[0])?;

    if symbol.context() != SYSTEM_CONTEXT || symbol.name() != "Heads" {
        return None;
    }

    truthiness_symbol(&args[1])
}
