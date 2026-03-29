use super::text::remove_diacritics_value;
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NearestResultSpec {
    BestTies,
    Count(usize),
    All,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AlignmentBlock<T> {
    Common(Vec<T>),
    Different { left: Vec<T>, right: Vec<T> },
}

impl Session {
    pub(in crate::runtime) fn eval_nearest(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [data] => {
                return Ok(Expr::call(
                    system_expr("Function"),
                    vec![Expr::call(
                        system_expr("Nearest"),
                        vec![data.clone(), string_slot_expr()],
                    )],
                ));
            }
            [_, _] | [_, _, _] => {}
            _ => return Ok(self.invalid_call_with_arg_range("Nearest", args, 1, 3)),
        }

        let result_spec = match args.get(2) {
            Some(expr) => {
                let Some(spec) = parse_nearest_result_spec(expr) else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                };
                spec
            }
            None => NearestResultSpec::BestTies,
        };

        if let Some(target) = expr_as_string(&args[1]) {
            if let Some(values) = expr_as_string_list(&args[0]) {
                return Ok(string_list_expr(nearest_strings(
                    &values,
                    target,
                    result_spec,
                )));
            }

            if let Some((keys, values)) = parse_string_rule_dataset(&args[0]) {
                return Ok(list_expr(nearest_values_for_strings(
                    &keys,
                    &values,
                    target,
                    result_spec,
                )));
            }
        }

        if let Some(target) = expr_as_exact_rational(&args[1]) {
            if let Some((keys, values)) = parse_numeric_dataset(&args[0]) {
                return Ok(list_expr(nearest_values_for_numbers(
                    &keys,
                    &values,
                    &target,
                    result_spec,
                )));
            }
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(in crate::runtime) fn eval_pluralize(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [word_expr] => {
                let Some(word) = expr_as_string(word_expr) else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                };
                Ok(Expr::string(pluralize_english_word(word)))
            }
            [word_expr, count_expr] => {
                let Some(word) = expr_as_string(word_expr) else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                };
                let Some(count) = expr_as_exact_rational(count_expr) else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                };
                if count == Rational::one() {
                    Ok(Expr::string(word.to_string()))
                } else {
                    Ok(Expr::string(pluralize_english_word(word)))
                }
            }
            _ => Ok(self.invalid_call_with_arg_range("Pluralize", args, 1, 2)),
        }
    }

    pub(in crate::runtime) fn eval_alphabet(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let supported = match args {
            [] => true,
            [arg] => expr_as_string(arg).is_some_and(|name| name == "English"),
            _ => false,
        };

        if !supported {
            return Ok(if args.len() <= 1 {
                Expr::call(head.clone(), args.to_vec())
            } else {
                self.invalid_call_with_arg_range("Alphabet", args, 0, 1)
            });
        }

        Ok(string_list_expr(english_alphabet()))
    }

    pub(in crate::runtime) fn eval_alphabetic_sort(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("AlphabeticSort", args, 1, 2));
        }

        if let Some(alphabet_name) = args.get(1).and_then(expr_as_string) {
            if alphabet_name != "English" {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            }
        } else if args.len() == 2 {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }

        let Some(values) = expr_as_string_list(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let mut values = values;
        values.sort_by_key(|value| alphabetic_sort_key(value));
        Ok(string_list_expr(values))
    }

    pub(in crate::runtime) fn eval_edit_distance(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("EditDistance", args, 2));
        }

        if let (Some(lhs), Some(rhs)) = (expr_as_string(&args[0]), expr_as_string(&args[1])) {
            let lhs = lhs.chars().collect::<Vec<_>>();
            let rhs = rhs.chars().collect::<Vec<_>>();
            return Ok(Expr::integer(BigInt::from(
                levenshtein_distance(&lhs, &rhs) as u64,
            )));
        }

        if let (Some(lhs), Some(rhs)) = (
            call_items_with_head(&args[0], &system_expr("List")),
            call_items_with_head(&args[1], &system_expr("List")),
        ) {
            return Ok(Expr::integer(BigInt::from(
                levenshtein_distance(lhs, rhs) as u64
            )));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(in crate::runtime) fn eval_hamming_distance(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("HammingDistance", args, 2));
        }

        if let (Some(lhs), Some(rhs)) = (expr_as_string(&args[0]), expr_as_string(&args[1])) {
            let lhs = lhs.chars().collect::<Vec<_>>();
            let rhs = rhs.chars().collect::<Vec<_>>();
            let Some(distance) = hamming_distance(&lhs, &rhs) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            return Ok(Expr::integer(BigInt::from(distance as u64)));
        }

        if let (Some(lhs), Some(rhs)) = (
            call_items_with_head(&args[0], &system_expr("List")),
            call_items_with_head(&args[1], &system_expr("List")),
        ) {
            let Some(distance) = hamming_distance(lhs, rhs) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            return Ok(Expr::integer(BigInt::from(distance as u64)));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(in crate::runtime) fn eval_longest_common_subsequence(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("LongestCommonSubsequence", args, 2));
        }

        if let (Some(lhs), Some(rhs)) = (expr_as_string(&args[0]), expr_as_string(&args[1])) {
            let lhs = lhs.chars().collect::<Vec<_>>();
            let rhs = rhs.chars().collect::<Vec<_>>();
            let common = longest_common_contiguous_slice(&lhs, &rhs);
            return Ok(Expr::string(common.into_iter().collect::<String>()));
        }

        if let (Some(lhs), Some(rhs)) = (
            call_items_with_head(&args[0], &system_expr("List")),
            call_items_with_head(&args[1], &system_expr("List")),
        ) {
            return Ok(list_expr(longest_common_contiguous_slice(lhs, rhs)));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(in crate::runtime) fn eval_sequence_alignment(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("SequenceAlignment", args, 2));
        }

        if let (Some(lhs), Some(rhs)) = (expr_as_string(&args[0]), expr_as_string(&args[1])) {
            let lhs = lhs.chars().collect::<Vec<_>>();
            let rhs = rhs.chars().collect::<Vec<_>>();
            let alignment = global_alignment_blocks(&lhs, &rhs);
            return Ok(render_string_alignment_blocks(&alignment));
        }

        if let (Some(lhs), Some(rhs)) = (
            call_items_with_head(&args[0], &system_expr("List")),
            call_items_with_head(&args[1], &system_expr("List")),
        ) {
            let alignment = global_alignment_blocks(lhs, rhs);
            return Ok(render_expr_alignment_blocks(&alignment));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(in crate::runtime) fn eval_diff(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Diff", args, 2));
        }

        if let (Some(lhs), Some(rhs)) = (expr_as_string(&args[0]), expr_as_string(&args[1])) {
            let lhs = lhs.chars().collect::<Vec<_>>();
            let rhs = rhs.chars().collect::<Vec<_>>();
            let alignment = global_alignment_blocks(&lhs, &rhs);
            return Ok(render_diff_object_from_chars(&alignment));
        }

        if let (Some(lhs), Some(rhs)) = (
            call_items_with_head(&args[0], &system_expr("List")),
            call_items_with_head(&args[1], &system_expr("List")),
        ) {
            let alignment = global_alignment_blocks(lhs, rhs);
            return Ok(render_diff_object_from_exprs(&alignment));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }
}

fn parse_string_rule_dataset(expr: &Expr) -> Option<(Vec<String>, Vec<Expr>)> {
    let (keys, values) = parse_nearest_rule_pairs(expr)?;
    Some((
        keys.iter()
            .map(expr_as_string)
            .collect::<Option<Vec<_>>>()?
            .into_iter()
            .map(str::to_string)
            .collect(),
        values,
    ))
}

fn parse_numeric_dataset(expr: &Expr) -> Option<(Vec<Rational>, Vec<Expr>)> {
    if let Some(items) = call_items_with_head(expr, &system_expr("List")) {
        let keys = items
            .iter()
            .map(expr_as_exact_rational)
            .collect::<Option<Vec<_>>>()?;
        return Some((keys, items.to_vec()));
    }

    let (keys, values) = parse_nearest_rule_pairs(expr)?;
    Some((
        keys.iter()
            .map(expr_as_exact_rational)
            .collect::<Option<Vec<_>>>()?,
        values,
    ))
}

fn parse_nearest_rule_pairs(expr: &Expr) -> Option<(Vec<Expr>, Vec<Expr>)> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Rule") && args.len() == 2 => {
            let keys = call_items_with_head(&args[0], &system_expr("List"))?.to_vec();
            let values = call_items_with_head(&args[1], &system_expr("List"))?.to_vec();
            (keys.len() == values.len()).then_some((keys, values))
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
            let mut keys = Vec::with_capacity(args.len());
            let mut values = Vec::with_capacity(args.len());
            for item in args {
                let Expr::Call {
                    head: rule_head,
                    args: rule_args,
                } = item
                else {
                    return None;
                };
                if !is_system_symbol_name(rule_head, "Rule") || rule_args.len() != 2 {
                    return None;
                }
                keys.push(rule_args[0].clone());
                values.push(rule_args[1].clone());
            }
            Some((keys, values))
        }
        _ => None,
    }
}

fn parse_nearest_result_spec(expr: &Expr) -> Option<NearestResultSpec> {
    if is_system_symbol_name(expr, heads::structure::ALL) {
        return Some(NearestResultSpec::All);
    }

    expr_as_non_negative_usize(expr).map(NearestResultSpec::Count)
}

fn nearest_strings(values: &[String], target: &str, spec: NearestResultSpec) -> Vec<String> {
    let target_chars = target.chars().collect::<Vec<_>>();
    let distances = values
        .iter()
        .map(|value| {
            let value_chars = value.chars().collect::<Vec<_>>();
            levenshtein_distance(&value_chars, &target_chars)
        })
        .collect::<Vec<_>>();
    select_nearest_indices_by_usize(&distances, spec)
        .into_iter()
        .map(|index| values[index].clone())
        .collect()
}

fn nearest_values_for_strings(
    keys: &[String],
    values: &[Expr],
    target: &str,
    spec: NearestResultSpec,
) -> Vec<Expr> {
    let target_chars = target.chars().collect::<Vec<_>>();
    let distances = keys
        .iter()
        .map(|value| {
            let value_chars = value.chars().collect::<Vec<_>>();
            levenshtein_distance(&value_chars, &target_chars)
        })
        .collect::<Vec<_>>();
    select_nearest_indices_by_usize(&distances, spec)
        .into_iter()
        .map(|index| values[index].clone())
        .collect()
}

fn nearest_values_for_numbers(
    keys: &[Rational],
    values: &[Expr],
    target: &Rational,
    spec: NearestResultSpec,
) -> Vec<Expr> {
    let distances = keys
        .iter()
        .map(|value| rational_abs_difference(value, target))
        .collect::<Vec<_>>();
    select_nearest_indices_by_rational(&distances, spec)
        .into_iter()
        .map(|index| values[index].clone())
        .collect()
}

fn select_nearest_indices_by_usize(distances: &[usize], spec: NearestResultSpec) -> Vec<usize> {
    match spec {
        NearestResultSpec::Count(0) => Vec::new(),
        NearestResultSpec::Count(limit) => {
            let mut indices = (0..distances.len()).collect::<Vec<_>>();
            indices.sort_by_key(|index| (distances[*index], *index));
            indices.truncate(limit.min(indices.len()));
            indices
        }
        NearestResultSpec::All => {
            let mut indices = (0..distances.len()).collect::<Vec<_>>();
            indices.sort_by_key(|index| (distances[*index], *index));
            indices
        }
        NearestResultSpec::BestTies => {
            let Some(best) = distances.iter().copied().min() else {
                return Vec::new();
            };
            distances
                .iter()
                .enumerate()
                .filter_map(|(index, distance)| (*distance == best).then_some(index))
                .collect()
        }
    }
}

fn select_nearest_indices_by_rational(
    distances: &[Rational],
    spec: NearestResultSpec,
) -> Vec<usize> {
    match spec {
        NearestResultSpec::Count(0) => Vec::new(),
        NearestResultSpec::Count(limit) => {
            let mut indices = (0..distances.len()).collect::<Vec<_>>();
            indices.sort_by(|lhs, rhs| {
                compare_rationals_local(&distances[*lhs], &distances[*rhs]).then(lhs.cmp(rhs))
            });
            indices.truncate(limit.min(indices.len()));
            indices
        }
        NearestResultSpec::All => {
            let mut indices = (0..distances.len()).collect::<Vec<_>>();
            indices.sort_by(|lhs, rhs| {
                compare_rationals_local(&distances[*lhs], &distances[*rhs]).then(lhs.cmp(rhs))
            });
            indices
        }
        NearestResultSpec::BestTies => {
            let Some((best_index, _)) = distances
                .iter()
                .enumerate()
                .min_by(|(_, lhs), (_, rhs)| compare_rationals_local(lhs, rhs))
            else {
                return Vec::new();
            };
            distances
                .iter()
                .enumerate()
                .filter_map(|(index, distance)| {
                    (compare_rationals_local(distance, &distances[best_index])
                        == std::cmp::Ordering::Equal)
                        .then_some(index)
                })
                .collect()
        }
    }
}

fn rational_abs_difference(lhs: &Rational, rhs: &Rational) -> Rational {
    let difference = lhs.sub(rhs);
    if compare_rationals_local(&difference, &Rational::zero()) == std::cmp::Ordering::Less {
        difference.negated()
    } else {
        difference
    }
}

fn compare_rationals_local(lhs: &Rational, rhs: &Rational) -> std::cmp::Ordering {
    let lhs_cross = lhs.numerator() * rhs.denominator();
    let rhs_cross = rhs.numerator() * lhs.denominator();
    lhs_cross.cmp(&rhs_cross)
}

fn pluralize_english_word(word: &str) -> String {
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

fn english_alphabet() -> Vec<String> {
    ('a'..='z').map(|ch| ch.to_string()).collect()
}

fn alphabetic_sort_key(value: &str) -> Vec<(usize, String)> {
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

fn english_letter_index(ch: char) -> Option<usize> {
    ch.is_ascii_lowercase().then(|| (ch as u8 - b'a') as usize)
}

fn levenshtein_distance<T: Eq>(lhs: &[T], rhs: &[T]) -> usize {
    let mut previous = (0..=rhs.len()).collect::<Vec<_>>();
    let mut current = vec![0usize; rhs.len() + 1];

    for (i, left) in lhs.iter().enumerate() {
        current[0] = i + 1;
        for (j, right) in rhs.iter().enumerate() {
            let substitution = previous[j] + usize::from(left != right);
            let insertion = current[j] + 1;
            let deletion = previous[j + 1] + 1;
            current[j + 1] = substitution.min(insertion).min(deletion);
        }
        std::mem::swap(&mut previous, &mut current);
    }

    previous[rhs.len()]
}

fn hamming_distance<T: Eq>(lhs: &[T], rhs: &[T]) -> Option<usize> {
    if lhs.len() != rhs.len() {
        return None;
    }

    Some(
        lhs.iter()
            .zip(rhs)
            .filter(|(left, right)| left != right)
            .count(),
    )
}

fn longest_common_contiguous_slice<T: Clone + Eq>(lhs: &[T], rhs: &[T]) -> Vec<T> {
    if lhs.is_empty() || rhs.is_empty() {
        return Vec::new();
    }

    let mut dp = vec![vec![0usize; rhs.len() + 1]; lhs.len() + 1];
    let mut best_len = 0usize;
    let mut best_end = 0usize;

    for i in 1..=lhs.len() {
        for j in 1..=rhs.len() {
            if lhs[i - 1] == rhs[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
                if dp[i][j] > best_len {
                    best_len = dp[i][j];
                    best_end = i;
                }
            }
        }
    }

    lhs[best_end.saturating_sub(best_len)..best_end].to_vec()
}

fn global_alignment_blocks<T: Clone + Eq>(lhs: &[T], rhs: &[T]) -> Vec<AlignmentBlock<T>> {
    let m = lhs.len();
    let n = rhs.len();
    let mut scores = vec![vec![0isize; n + 1]; m + 1];
    let mut steps = vec![vec![AlignmentStep::Done; n + 1]; m + 1];

    for i in 1..=m {
        scores[i][0] = -(i as isize);
        steps[i][0] = AlignmentStep::Up;
    }
    for j in 1..=n {
        scores[0][j] = -(j as isize);
        steps[0][j] = AlignmentStep::Left;
    }

    for i in 1..=m {
        for j in 1..=n {
            let diag_score = scores[i - 1][j - 1] + if lhs[i - 1] == rhs[j - 1] { 1 } else { -1 };
            let up_score = scores[i - 1][j] - 1;
            let left_score = scores[i][j - 1] - 1;
            let (score, step) = if diag_score >= up_score && diag_score >= left_score {
                (diag_score, AlignmentStep::Diag)
            } else if up_score >= left_score {
                (up_score, AlignmentStep::Up)
            } else {
                (left_score, AlignmentStep::Left)
            };
            scores[i][j] = score;
            steps[i][j] = step;
        }
    }

    let mut aligned = Vec::new();
    let (mut i, mut j) = (m, n);
    while i > 0 || j > 0 {
        match steps[i][j] {
            AlignmentStep::Diag => {
                aligned.push((Some(lhs[i - 1].clone()), Some(rhs[j - 1].clone())));
                i -= 1;
                j -= 1;
            }
            AlignmentStep::Up => {
                aligned.push((Some(lhs[i - 1].clone()), None));
                i -= 1;
            }
            AlignmentStep::Left => {
                aligned.push((None, Some(rhs[j - 1].clone())));
                j -= 1;
            }
            AlignmentStep::Done => break,
        }
    }
    aligned.reverse();

    let mut blocks = Vec::new();
    let mut common = Vec::new();
    let mut left_diff = Vec::new();
    let mut right_diff = Vec::new();

    for (left, right) in aligned {
        match (left, right) {
            (Some(left), Some(right)) if left == right => {
                if !left_diff.is_empty() || !right_diff.is_empty() {
                    blocks.push(AlignmentBlock::Different {
                        left: std::mem::take(&mut left_diff),
                        right: std::mem::take(&mut right_diff),
                    });
                }
                common.push(left);
            }
            (left, right) => {
                if !common.is_empty() {
                    blocks.push(AlignmentBlock::Common(std::mem::take(&mut common)));
                }
                if let Some(left) = left {
                    left_diff.push(left);
                }
                if let Some(right) = right {
                    right_diff.push(right);
                }
            }
        }
    }

    if !common.is_empty() {
        blocks.push(AlignmentBlock::Common(common));
    }
    if !left_diff.is_empty() || !right_diff.is_empty() {
        blocks.push(AlignmentBlock::Different {
            left: left_diff,
            right: right_diff,
        });
    }

    blocks
}

fn render_string_alignment_blocks(blocks: &[AlignmentBlock<char>]) -> Expr {
    list_expr(
        blocks
            .iter()
            .map(|block| match block {
                AlignmentBlock::Common(common) => Expr::string(common.iter().collect::<String>()),
                AlignmentBlock::Different { left, right } => list_expr(vec![
                    Expr::string(left.iter().collect::<String>()),
                    Expr::string(right.iter().collect::<String>()),
                ]),
            })
            .collect(),
    )
}

fn render_expr_alignment_blocks(blocks: &[AlignmentBlock<Expr>]) -> Expr {
    list_expr(
        blocks
            .iter()
            .map(|block| match block {
                AlignmentBlock::Common(common) => list_expr(common.clone()),
                AlignmentBlock::Different { left, right } => {
                    list_expr(vec![list_expr(left.clone()), list_expr(right.clone())])
                }
            })
            .collect(),
    )
}

fn render_diff_object_from_chars(blocks: &[AlignmentBlock<char>]) -> Expr {
    render_diff_object("Characters", blocks, |items| {
        Expr::string(items.iter().collect::<String>())
    })
}

fn render_diff_object_from_exprs(blocks: &[AlignmentBlock<Expr>]) -> Expr {
    render_diff_object("Elements", blocks, |items| list_expr(items.to_vec()))
}

fn render_diff_object<T: Clone>(
    mode: &str,
    blocks: &[AlignmentBlock<T>],
    render_items: impl Fn(&[T]) -> Expr,
) -> Expr {
    let mut left_index = 0usize;
    let mut right_index = 0usize;
    let mut changes = Vec::new();

    for block in blocks {
        match block {
            AlignmentBlock::Common(common) => {
                let rule = Expr::call(
                    system_expr("Rule"),
                    vec![
                        list_expr(vec![
                            Expr::integer(BigInt::from(left_index + 1)),
                            Expr::integer(BigInt::from(right_index + 1)),
                        ]),
                        render_items(common),
                    ],
                );
                changes.push(list_expr(vec![rule]));
                left_index += common.len();
                right_index += common.len();
            }
            AlignmentBlock::Different { left, right } => {
                let left_key = if left.is_empty() {
                    left_index
                } else {
                    left_index + 1
                };
                let right_key = if right.is_empty() {
                    right_index
                } else {
                    right_index + 1
                };
                changes.push(list_expr(vec![
                    Expr::call(
                        system_expr("Rule"),
                        vec![Expr::integer(BigInt::from(left_key)), render_items(left)],
                    ),
                    Expr::call(
                        system_expr("Rule"),
                        vec![Expr::integer(BigInt::from(right_key)), render_items(right)],
                    ),
                ]));
                left_index += left.len();
                right_index += right.len();
            }
        }
    }

    Expr::call(
        system_expr("DiffObject"),
        vec![
            Expr::integer(BigInt::from(1_u8)),
            Expr::string(mode.to_string()),
            render_association(vec![
                (Expr::string("Changes"), list_expr(changes)),
                (
                    Expr::string("View"),
                    list_expr(vec![system_expr("Automatic")]),
                ),
            ]),
        ],
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AlignmentStep {
    Done,
    Diag,
    Up,
    Left,
}
