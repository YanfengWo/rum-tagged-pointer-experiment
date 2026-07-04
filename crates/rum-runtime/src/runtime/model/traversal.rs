use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum CasesSpec {
    Pattern(Expr),
    Rule(RewriteRule),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CasesLevelSpec {
    pub(in crate::runtime) min: isize,
    pub(in crate::runtime) max: Option<isize>,
}

impl CasesLevelSpec {
    pub(in crate::runtime) fn default_cases() -> Self {
        Self::exact(1)
    }

    /// Builds a level specification matching exactly one level.
    pub fn exact(level: isize) -> Self {
        Self {
            min: level,
            max: Some(level),
        }
    }

    /// Builds a level specification matching a signed inclusive range.
    pub fn range(min: isize, max: Option<isize>) -> Self {
        Self { min, max }
    }

    /// Tests whether a traversal level and depth satisfy this specification.
    pub fn matches(self, level: usize, depth: usize) -> bool {
        let level = level_as_signed(level);
        let depth = level_as_signed(depth);

        if self.spans_negative_and_nonnegative_levels() {
            let positive_level_matches = self.max.is_none_or(|max| level <= max);
            let negative_depth_matches = self
                .min
                .checked_abs()
                .is_none_or(|max_depth| depth <= max_depth);
            return positive_level_matches && negative_depth_matches;
        }

        self.contains_signed(level) || self.contains_signed(-depth)
    }

    fn contains_signed(self, level: isize) -> bool {
        level >= self.min && self.max.is_none_or(|max| level <= max)
    }

    fn spans_negative_and_nonnegative_levels(self) -> bool {
        self.min < 0 && self.max.is_none_or(|max| max >= 0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CasesTraversalConfig {
    pub(in crate::runtime) levels: CasesLevelSpec,
    pub(in crate::runtime) max_matches: Option<usize>,
    pub(in crate::runtime) include_heads: bool,
}

impl CasesTraversalConfig {
    /// Builds the default Cases-style traversal configuration.
    pub fn default_cases() -> Self {
        Self {
            levels: CasesLevelSpec::default_cases(),
            max_matches: None,
            include_heads: false,
        }
    }

    /// Returns this traversal configuration with a different level spec.
    pub fn with_levels(mut self, levels: CasesLevelSpec) -> Self {
        self.levels = levels;
        self
    }

    pub(in crate::runtime) fn default_member_q() -> Self {
        Self {
            max_matches: Some(1),
            ..Self::default_cases()
        }
    }

    pub(in crate::runtime) fn default_free_q() -> Self {
        Self {
            levels: CasesLevelSpec::range(0, None),
            max_matches: Some(1),
            include_heads: true,
        }
    }

    pub(in crate::runtime) fn default_position() -> Self {
        Self {
            levels: CasesLevelSpec::range(0, None),
            max_matches: None,
            include_heads: true,
        }
    }

    pub(in crate::runtime) fn limit_reached(self, current_count: usize) -> bool {
        self.max_matches.is_some_and(|limit| current_count >= limit)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct CasesCollectionOutcome {
    pub(in crate::runtime) matches: Vec<Expr>,
    pub(in crate::runtime) depth: usize,
    pub(in crate::runtime) exhausted: bool,
}

impl CasesCollectionOutcome {
    pub(in crate::runtime) fn new(matches: Vec<Expr>, depth: usize, exhausted: bool) -> Self {
        Self {
            matches,
            depth,
            exhausted,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct PositionCollectionOutcome {
    pub(in crate::runtime) positions: Vec<Expr>,
    pub(in crate::runtime) depth: usize,
    pub(in crate::runtime) exhausted: bool,
}

impl PositionCollectionOutcome {
    pub(in crate::runtime) fn new(positions: Vec<Expr>, depth: usize, exhausted: bool) -> Self {
        Self {
            positions,
            depth,
            exhausted,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct TraversalCountOutcome {
    pub(in crate::runtime) count: usize,
    pub(in crate::runtime) depth: usize,
    pub(in crate::runtime) exhausted: bool,
}

impl TraversalCountOutcome {
    pub(in crate::runtime) fn new(count: usize, depth: usize, exhausted: bool) -> Self {
        Self {
            count,
            depth,
            exhausted,
        }
    }
}

fn level_as_signed(level: usize) -> isize {
    isize::try_from(level).unwrap_or(isize::MAX)
}
