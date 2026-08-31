use super::*;

macro_rules! matcher_pattern_heads {
    ($($variant:ident => $name:literal),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub(in crate::runtime) enum MatcherPatternHead {
            $($variant),+
        }

        impl MatcherPatternHead {
            #[cfg(test)]
            pub(in crate::runtime) const ALL: &'static [Self] = &[$(Self::$variant),+];

            #[inline]
            pub(in crate::runtime) const fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => $name),+
                }
            }

            #[inline]
            pub(in crate::runtime) fn matches(self, expr: &Expr) -> bool {
                is_system_symbol_name(expr, self.name())
            }
        }

        #[inline]
        pub(in crate::runtime) fn matcher_pattern_head(expr: &Expr) -> Option<MatcherPatternHead> {
            let symbol = expr_as_symbol(expr)?;
            if symbol.context() != SYSTEM_CONTEXT {
                return None;
            }

            matcher_pattern_head_name(symbol.name())
        }

        #[inline]
        pub(in crate::runtime) fn matcher_pattern_head_name(
            name: &str,
        ) -> Option<MatcherPatternHead> {
            match name {
                $($name => Some(MatcherPatternHead::$variant)),+,
                _ => None,
            }
        }
    };
}

// This is the matcher pattern-language registry. Matcher parsing and lhs
// preparation consume the generated enum/predicates so adding a head cannot
// silently leave preparation with an unrelated string list.
matcher_pattern_heads! {
    Alternatives => "Alternatives",
    Blank => "Blank",
    BlankSequence => "BlankSequence",
    BlankNullSequence => "BlankNullSequence",
    Condition => "Condition",
    Except => "Except",
    HoldPattern => "HoldPattern",
    KeyValuePattern => "KeyValuePattern",
    Longest => "Longest",
    Optional => "Optional",
    OptionsPattern => "OptionsPattern",
    OrderlessPatternSequence => "OrderlessPatternSequence",
    Pattern => "Pattern",
    PatternSequence => "PatternSequence",
    PatternTest => "PatternTest",
    Repeated => "Repeated",
    RepeatedNull => "RepeatedNull",
    Shortest => "Shortest",
    Verbatim => "Verbatim",
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matcher_pattern_registry_round_trips_system_heads_only() {
        assert_eq!(MatcherPatternHead::ALL.len(), 19);

        let mut names = BTreeSet::new();
        for pattern_head in MatcherPatternHead::ALL {
            assert!(names.insert(pattern_head.name()));
            assert_eq!(
                matcher_pattern_head(&system_expr(pattern_head.name())),
                Some(*pattern_head)
            );
            assert_eq!(
                matcher_pattern_head(&Expr::symbol(pattern_head.name())),
                None
            );
        }
    }
}
