use super::*;
use std::sync::Arc;

const REWRITE_RULE_EXACT_INDEX_THRESHOLD: usize = 32;

#[derive(Clone, Debug)]
pub(super) struct RewriteRulePlan(Arc<RewriteRulePlanData>);

#[derive(Debug)]
struct RewriteRulePlanData {
    rules: Vec<RewriteRule>,
    exact_index: Option<RewriteRuleExactIndex>,
}

#[derive(Debug)]
struct RewriteRuleExactIndex {
    first_rule_by_pattern: HashMap<Expr, usize>,
    non_exact_rule_indices: Vec<usize>,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum RewriteRuleScanCursor {
    Linear {
        next_rule_index: usize,
    },
    Indexed {
        next_non_exact_position: usize,
        exact_rule_index: Option<usize>,
    },
}

impl RewriteRulePlan {
    pub(super) fn from_slice(rules: &[RewriteRule]) -> Self {
        Self::indexed(rules.to_vec())
    }

    pub(super) fn linear(rules: Vec<RewriteRule>) -> Self {
        Self(Arc::new(RewriteRulePlanData {
            rules,
            exact_index: None,
        }))
    }

    fn indexed(rules: Vec<RewriteRule>) -> Self {
        let exact_rule_count = rules
            .iter()
            .filter(|rule| matches!(rule.matcher, MatcherNode::Exact(_)))
            .count();
        let exact_index = (exact_rule_count >= REWRITE_RULE_EXACT_INDEX_THRESHOLD).then(|| {
            let mut first_rule_by_pattern = HashMap::with_capacity(exact_rule_count);
            let mut non_exact_rule_indices =
                Vec::with_capacity(rules.len().saturating_sub(exact_rule_count));

            for (index, rule) in rules.iter().enumerate() {
                if let MatcherNode::Exact(pattern) = &rule.matcher {
                    first_rule_by_pattern
                        .entry(pattern.clone())
                        .or_insert(index);
                } else {
                    non_exact_rule_indices.push(index);
                }
            }

            RewriteRuleExactIndex {
                first_rule_by_pattern,
                non_exact_rule_indices,
            }
        });

        Self(Arc::new(RewriteRulePlanData { rules, exact_index }))
    }

    pub(super) fn start_scan(&self, candidate: &Expr) -> RewriteRuleScanCursor {
        match &self.0.exact_index {
            Some(index) => RewriteRuleScanCursor::Indexed {
                next_non_exact_position: 0,
                exact_rule_index: index.first_rule_by_pattern.get(candidate).copied(),
            },
            None => RewriteRuleScanCursor::Linear { next_rule_index: 0 },
        }
    }

    pub(super) fn next_rule_index(&self, cursor: &mut RewriteRuleScanCursor) -> Option<usize> {
        match cursor {
            RewriteRuleScanCursor::Linear { next_rule_index } => {
                let index = *next_rule_index;
                if index >= self.0.rules.len() {
                    return None;
                }
                *next_rule_index += 1;
                Some(index)
            }
            RewriteRuleScanCursor::Indexed {
                next_non_exact_position,
                exact_rule_index,
            } => {
                let index = self
                    .0
                    .exact_index
                    .as_ref()
                    .expect("indexed rewrite-rule cursor requires an index");
                let next_non_exact = index
                    .non_exact_rule_indices
                    .get(*next_non_exact_position)
                    .copied();

                match (next_non_exact, *exact_rule_index) {
                    (Some(non_exact), Some(exact)) if exact < non_exact => {
                        *exact_rule_index = None;
                        Some(exact)
                    }
                    (Some(non_exact), _) => {
                        *next_non_exact_position += 1;
                        Some(non_exact)
                    }
                    (None, Some(exact)) => {
                        *exact_rule_index = None;
                        Some(exact)
                    }
                    (None, None) => None,
                }
            }
        }
    }

    pub(super) fn get(&self, index: usize) -> Option<&RewriteRule> {
        self.0.rules.get(index)
    }
}
