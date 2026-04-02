use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum AlignmentBlock<T> {
    Common(Vec<T>),
    Different { left: Vec<T>, right: Vec<T> },
}
pub(super) fn levenshtein_distance<T: Eq>(lhs: &[T], rhs: &[T]) -> usize {
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

pub(super) fn hamming_distance<T: Eq>(lhs: &[T], rhs: &[T]) -> Option<usize> {
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

pub(super) fn longest_common_contiguous_slice<T: Clone + Eq>(lhs: &[T], rhs: &[T]) -> Vec<T> {
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

pub(super) fn global_alignment_blocks<T: Clone + Eq>(lhs: &[T], rhs: &[T]) -> Vec<AlignmentBlock<T>> {
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

pub(super) fn render_string_alignment_blocks(blocks: &[AlignmentBlock<char>]) -> Expr {
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

pub(super) fn render_expr_alignment_blocks(blocks: &[AlignmentBlock<Expr>]) -> Expr {
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

pub(super) fn render_diff_object_from_chars(blocks: &[AlignmentBlock<char>]) -> Expr {
    render_diff_object("Characters", blocks, |items| {
        Expr::string(items.iter().collect::<String>())
    })
}

pub(super) fn render_diff_object_from_exprs(blocks: &[AlignmentBlock<Expr>]) -> Expr {
    render_diff_object("Elements", blocks, |items| list_expr(items.to_vec()))
}

pub(super) fn render_diff_object<T: Clone>(
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
pub(super) enum AlignmentStep {
    Done,
    Diag,
    Up,
    Left,
}
