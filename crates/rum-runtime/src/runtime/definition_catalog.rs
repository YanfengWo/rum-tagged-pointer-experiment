use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum CasesTraversalNormalizationError {
    InvalidThirdArgument,
    InvalidFourthArgument,
    InvalidFifthArgument,
    InvalidHeadsOption {
        positional_count: usize,
        option_expr: Expr,
        classification: HeadsOptionClassification,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum CasesTraversalResolution {
    Valid(CasesTraversalConfig),
    Invalid(CasesTraversalNormalizationError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum TraversalWithoutLimitNormalizationError {
    InvalidThirdArgument,
    InvalidFourthArgument,
    InvalidHeadsOption {
        positional_count: usize,
        option_expr: Expr,
        classification: HeadsOptionClassification,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum TraversalWithoutLimitResolution {
    Valid(CasesTraversalConfig),
    Invalid(TraversalWithoutLimitNormalizationError),
}

impl CasesTraversalNormalizationError {
    pub(super) fn message(&self) -> &'static str {
        match self {
            Self::InvalidThirdArgument => {
                "expected a standard levelspec or Heads -> True/False as the third argument"
            }
            Self::InvalidFourthArgument => {
                "expected a non-negative match limit or Heads -> True/False as the fourth argument"
            }
            Self::InvalidFifthArgument => {
                "expected a Heads -> True/False option as the fifth argument"
            }
            Self::InvalidHeadsOption {
                positional_count, ..
            } => match positional_count {
                2 => "expected a standard levelspec or Heads -> True/False as the third argument",
                3 => {
                    "expected a non-negative match limit or Heads -> True/False as the fourth argument"
                }
                4 => "expected a Heads -> True/False option as the fifth argument",
                _ => "expected a valid Heads option",
            },
        }
    }
}

impl TraversalWithoutLimitNormalizationError {
    pub(super) fn message(&self) -> &'static str {
        match self {
            Self::InvalidThirdArgument => {
                "expected a standard levelspec or Heads -> True/False as the third argument"
            }
            Self::InvalidFourthArgument => {
                "expected a Heads -> True/False option as the fourth argument"
            }
            Self::InvalidHeadsOption {
                positional_count, ..
            } => match positional_count {
                2 => "expected a standard levelspec or Heads -> True/False as the third argument",
                3 => "expected a Heads -> True/False option as the fourth argument",
                _ => "expected a valid Heads option",
            },
        }
    }
}

impl ContinuationValue for TraversalWithoutLimitResolution {
    fn into_continuation_expr(self) -> Expr {
        match self {
            Self::Valid(config) => Expr::call(
                system_expr("InternalContinuationTraversalWithoutLimitValid"),
                vec![config.into_continuation_expr()],
            ),
            Self::Invalid(error) => Expr::call(
                system_expr("InternalContinuationTraversalWithoutLimitInvalid"),
                vec![error.into_continuation_expr()],
            ),
        }
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, mut args }
                if is_system_symbol_name(
                    &head,
                    "InternalContinuationTraversalWithoutLimitValid",
                ) && args.len() == 1 =>
            {
                let config_expr = args
                    .pop()
                    .expect("valid traversal-without-limit stores config");
                Self::Valid(CasesTraversalConfig::from_continuation_expr(config_expr))
            }
            Expr::Call { head, mut args }
                if is_system_symbol_name(
                    &head,
                    "InternalContinuationTraversalWithoutLimitInvalid",
                ) && args.len() == 1 =>
            {
                let error_expr = args
                    .pop()
                    .expect("invalid traversal-without-limit stores error");
                Self::Invalid(
                    TraversalWithoutLimitNormalizationError::from_continuation_expr(error_expr),
                )
            }
            other => panic!(
                "unexpected continuation payload for TraversalWithoutLimitResolution: {other:?}"
            ),
        }
    }
}

impl ContinuationValue for TraversalWithoutLimitNormalizationError {
    fn into_continuation_expr(self) -> Expr {
        match self {
            Self::InvalidThirdArgument => Expr::integer(BigInt::from(3_i32)),
            Self::InvalidFourthArgument => Expr::integer(BigInt::from(4_i32)),
            Self::InvalidHeadsOption {
                positional_count,
                option_expr,
                classification,
            } => Expr::call(
                system_expr("InternalContinuationTraversalWithoutLimitHeadsOptionInvalid"),
                vec![
                    Expr::integer(BigInt::from(positional_count)),
                    option_expr,
                    classification.into_continuation_expr(),
                ],
            ),
        }
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        if let Some(code) = expr_as_non_negative_usize(&expr) {
            return match code {
                3 => Self::InvalidThirdArgument,
                4 => Self::InvalidFourthArgument,
                other => panic!("unexpected traversal-without-limit error code: {other}"),
            };
        }

        match expr {
            Expr::Call { head, args }
                if is_system_symbol_name(
                    &head,
                    "InternalContinuationTraversalWithoutLimitHeadsOptionInvalid",
                ) && args.len() == 3 =>
            {
                let mut args = args.into_iter();
                let positional_count = expr_as_non_negative_usize(
                    &args.next().expect("heads option error stores a position"),
                )
                .expect("heads option error position should be an integer");
                let option_expr = args.next().expect("heads option error stores option expr");
                let classification = HeadsOptionClassification::from_continuation_expr(
                    args.next()
                        .expect("heads option error stores classification"),
                );
                Self::InvalidHeadsOption {
                    positional_count,
                    option_expr,
                    classification,
                }
            }
            other => panic!("unexpected traversal-without-limit error payload: {other:?}"),
        }
    }
}

impl ContinuationValue for CasesTraversalNormalizationError {
    fn into_continuation_expr(self) -> Expr {
        match self {
            Self::InvalidThirdArgument => Expr::integer(BigInt::from(3_i32)),
            Self::InvalidFourthArgument => Expr::integer(BigInt::from(4_i32)),
            Self::InvalidFifthArgument => Expr::integer(BigInt::from(5_i32)),
            Self::InvalidHeadsOption {
                positional_count,
                option_expr,
                classification,
            } => Expr::call(
                system_expr("InternalContinuationCasesTraversalHeadsOptionInvalid"),
                vec![
                    Expr::integer(BigInt::from(positional_count)),
                    option_expr,
                    classification.into_continuation_expr(),
                ],
            ),
        }
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        if let Some(code) = expr_as_non_negative_usize(&expr) {
            return match code {
                3 => Self::InvalidThirdArgument,
                4 => Self::InvalidFourthArgument,
                5 => Self::InvalidFifthArgument,
                other => panic!("unexpected cases-traversal error code: {other}"),
            };
        }

        match expr {
            Expr::Call { head, args }
                if is_system_symbol_name(
                    &head,
                    "InternalContinuationCasesTraversalHeadsOptionInvalid",
                ) && args.len() == 3 =>
            {
                let mut args = args.into_iter();
                let positional_count = expr_as_non_negative_usize(
                    &args.next().expect("heads option error stores a position"),
                )
                .expect("heads option error position should be an integer");
                let option_expr = args.next().expect("heads option error stores option expr");
                let classification = HeadsOptionClassification::from_continuation_expr(
                    args.next()
                        .expect("heads option error stores classification"),
                );
                Self::InvalidHeadsOption {
                    positional_count,
                    option_expr,
                    classification,
                }
            }
            other => panic!("unexpected cases-traversal error payload: {other:?}"),
        }
    }
}

impl Session {
    fn compile_traversal_from_resolved_exprs(
        &self,
        resolved: &[Expr],
        defaults: CasesTraversalConfig,
    ) -> Result<CasesTraversalConfig, CasesTraversalNormalizationError> {
        match resolved {
            [] => Ok(defaults),
            [third] => {
                let classification = classify_heads_option_expr(third);
                if let HeadsOptionClassification::Value(Some(include_heads)) = classification {
                    return Ok(CasesTraversalConfig {
                        include_heads,
                        ..defaults
                    });
                }

                if let Some(levels) = parse_standard_level_spec(third) {
                    Ok(CasesTraversalConfig {
                        levels,
                        max_matches: defaults.max_matches,
                        include_heads: defaults.include_heads,
                    })
                } else if heads_option_classification_looks_like_option_syntax(&classification) {
                    Err(CasesTraversalNormalizationError::InvalidHeadsOption {
                        positional_count: 2,
                        option_expr: third.clone(),
                        classification,
                    })
                } else {
                    Err(CasesTraversalNormalizationError::InvalidThirdArgument)
                }
            }
            [third, fourth] => {
                let Some(levels) = parse_standard_level_spec(third) else {
                    return Err(CasesTraversalNormalizationError::InvalidThirdArgument);
                };

                let classification = classify_heads_option_expr(fourth);
                if let HeadsOptionClassification::Value(include_heads) = classification {
                    Ok(CasesTraversalConfig {
                        levels,
                        max_matches: defaults.max_matches,
                        include_heads: include_heads.unwrap_or(defaults.include_heads),
                    })
                } else if let Some(max_matches) = parse_non_negative_match_limit(fourth) {
                    Ok(CasesTraversalConfig {
                        levels,
                        max_matches: Some(max_matches),
                        include_heads: defaults.include_heads,
                    })
                } else if heads_option_classification_looks_like_option_syntax(&classification) {
                    Err(CasesTraversalNormalizationError::InvalidHeadsOption {
                        positional_count: 3,
                        option_expr: fourth.clone(),
                        classification,
                    })
                } else {
                    Err(CasesTraversalNormalizationError::InvalidFourthArgument)
                }
            }
            [third, fourth, fifth] => {
                let Some(levels) = parse_standard_level_spec(third) else {
                    return Err(CasesTraversalNormalizationError::InvalidThirdArgument);
                };
                let Some(max_matches) = parse_non_negative_match_limit(fourth) else {
                    return Err(CasesTraversalNormalizationError::InvalidFourthArgument);
                };
                let classification = classify_heads_option_expr(fifth);
                let HeadsOptionClassification::Value(include_heads) = classification else {
                    return Err(CasesTraversalNormalizationError::InvalidHeadsOption {
                        positional_count: 4,
                        option_expr: fifth.clone(),
                        classification,
                    });
                };

                Ok(CasesTraversalConfig {
                    levels,
                    max_matches: Some(max_matches),
                    include_heads: include_heads.unwrap_or(defaults.include_heads),
                })
            }
            _ => unreachable!("traversal arity is validated before traversal normalization"),
        }
    }

    fn compile_traversal_without_limit_from_resolved_exprs(
        &self,
        resolved: &[Expr],
        defaults: CasesTraversalConfig,
    ) -> Result<CasesTraversalConfig, TraversalWithoutLimitNormalizationError> {
        match resolved {
            [] => Ok(defaults),
            [third] => {
                let classification = classify_heads_option_expr(third);
                if let HeadsOptionClassification::Value(Some(include_heads)) = classification {
                    return Ok(CasesTraversalConfig {
                        include_heads,
                        ..defaults
                    });
                }

                if let Some(levels) = parse_standard_level_spec(third) {
                    Ok(CasesTraversalConfig { levels, ..defaults })
                } else if heads_option_classification_looks_like_option_syntax(&classification) {
                    Err(
                        TraversalWithoutLimitNormalizationError::InvalidHeadsOption {
                            positional_count: 2,
                            option_expr: third.clone(),
                            classification,
                        },
                    )
                } else {
                    Err(TraversalWithoutLimitNormalizationError::InvalidThirdArgument)
                }
            }
            [third, fourth] => {
                let Some(levels) = parse_standard_level_spec(third) else {
                    return Err(TraversalWithoutLimitNormalizationError::InvalidThirdArgument);
                };
                let classification = classify_heads_option_expr(fourth);
                let HeadsOptionClassification::Value(include_heads) = classification else {
                    return Err(
                        TraversalWithoutLimitNormalizationError::InvalidHeadsOption {
                            positional_count: 3,
                            option_expr: fourth.clone(),
                            classification,
                        },
                    );
                };

                Ok(CasesTraversalConfig {
                    levels,
                    include_heads: include_heads.unwrap_or(defaults.include_heads),
                    ..defaults
                })
            }
            _ => {
                unreachable!("count/member/free traversal arity is validated before normalization")
            }
        }
    }

    pub(super) fn invalid_cases_traversal_call(
        &mut self,
        symbol: &str,
        args: &[Expr],
        error: CasesTraversalNormalizationError,
    ) -> Expr {
        let fallback = error.message().to_string();
        match error {
            CasesTraversalNormalizationError::InvalidThirdArgument => {
                self.invalid_level_spec_call(symbol, args, &args[2])
            }
            CasesTraversalNormalizationError::InvalidFourthArgument => {
                self.invalid_non_negative_or_infinity_call(symbol, args, 4)
            }
            CasesTraversalNormalizationError::InvalidHeadsOption {
                positional_count,
                option_expr,
                classification,
            } => self.invalid_heads_option_call(
                symbol,
                args,
                &option_expr,
                positional_count,
                classification,
                fallback,
            ),
            CasesTraversalNormalizationError::InvalidFifthArgument => {
                self.invalid_call_with_arg_form(symbol, args, fallback)
            }
        }
    }

    pub(super) fn invalid_traversal_without_limit_call(
        &mut self,
        symbol: &str,
        args: &[Expr],
        error: TraversalWithoutLimitNormalizationError,
    ) -> Expr {
        let fallback = error.message().to_string();
        match error {
            TraversalWithoutLimitNormalizationError::InvalidThirdArgument => {
                self.invalid_level_spec_call(symbol, args, &args[2])
            }
            TraversalWithoutLimitNormalizationError::InvalidHeadsOption {
                positional_count,
                option_expr,
                classification,
            } => self.invalid_heads_option_call(
                symbol,
                args,
                &option_expr,
                positional_count,
                classification,
                fallback,
            ),
            TraversalWithoutLimitNormalizationError::InvalidFourthArgument => {
                self.invalid_call_with_arg_form(symbol, args, fallback)
            }
        }
    }

    pub(super) fn resolve_argument_exprs_owned(
        &mut self,
        exprs: Vec<Expr>,
    ) -> Result<Vec<Expr>, EvalError> {
        let Some((first, rest)) = exprs.split_first() else {
            return Ok(Vec::new());
        };

        let first_result = self.evaluate(first.clone());
        let rest = rest.to_vec();
        self.drive_evaluated_value(first_result, {
            let rest = rest.clone();
            move |session, first| session.resume_resolve_argument_exprs_after_first(first, rest)
        })
    }

    pub(super) fn resolve_cases_spec_expr_for_matching_owned(
        &mut self,
        expr: Expr,
    ) -> Result<Expr, EvalError> {
        let candidate = strip_hold_pattern(&expr).clone();

        if should_evaluate_single_rule_candidate(&candidate) {
            let resolved_result = self.evaluate(candidate);
            return self.drive_evaluated_value(resolved_result, move |session, resolved| {
                if session
                    .compile_rewrite_rule_for_matching(&resolved)
                    .is_some()
                {
                    Ok(resolved)
                } else {
                    Ok(session.normalize_match_expr(&resolved))
                }
            });
        }

        self.resolve_matcher_pattern_expr_owned(candidate)
    }

    pub(super) fn compile_cases_spec_from_resolved_expr(&self, resolved: Expr) -> CasesSpec {
        match self.compile_rewrite_rule_for_matching(&resolved) {
            Some(rule) => CasesSpec::Rule(rule),
            None => CasesSpec::Pattern(resolved),
        }
    }

    pub(super) fn compile_cases_traversal_from_resolved_exprs(
        &self,
        resolved: &[Expr],
        defaults: CasesTraversalConfig,
    ) -> Result<CasesTraversalConfig, CasesTraversalNormalizationError> {
        self.compile_traversal_from_resolved_exprs(resolved, defaults)
    }

    pub(super) fn compile_position_traversal_from_resolved_exprs(
        &self,
        resolved: &[Expr],
        defaults: CasesTraversalConfig,
    ) -> Result<CasesTraversalConfig, CasesTraversalNormalizationError> {
        self.compile_traversal_from_resolved_exprs(resolved, defaults)
    }

    pub(super) fn resolve_cases_traversal_owned(
        &mut self,
        exprs: Vec<Expr>,
        defaults: CasesTraversalConfig,
    ) -> Result<CasesTraversalResolution, EvalError> {
        let resolved_exprs_result = self.resolve_argument_exprs_owned(exprs);
        self.drive_internal_value(resolved_exprs_result, move |session, resolved_exprs| {
            Ok(
                match session.compile_cases_traversal_from_resolved_exprs(&resolved_exprs, defaults)
                {
                    Ok(config) => CasesTraversalResolution::Valid(config),
                    Err(error) => CasesTraversalResolution::Invalid(error),
                },
            )
        })
    }

    pub(super) fn resolve_position_traversal_owned(
        &mut self,
        exprs: Vec<Expr>,
        defaults: CasesTraversalConfig,
    ) -> Result<CasesTraversalResolution, EvalError> {
        let resolved_exprs_result = self.resolve_argument_exprs_owned(exprs);
        self.drive_internal_value(resolved_exprs_result, move |session, resolved_exprs| {
            Ok(
                match session
                    .compile_position_traversal_from_resolved_exprs(&resolved_exprs, defaults)
                {
                    Ok(config) => CasesTraversalResolution::Valid(config),
                    Err(error) => CasesTraversalResolution::Invalid(error),
                },
            )
        })
    }

    pub(super) fn resolve_traversal_without_limit_owned(
        &mut self,
        exprs: Vec<Expr>,
        defaults: CasesTraversalConfig,
    ) -> Result<TraversalWithoutLimitResolution, EvalError> {
        let resolved_exprs_result = self.resolve_argument_exprs_owned(exprs);
        self.drive_internal_value(resolved_exprs_result, move |session, resolved_exprs| {
            Ok(
                match session
                    .compile_traversal_without_limit_from_resolved_exprs(&resolved_exprs, defaults)
                {
                    Ok(config) => TraversalWithoutLimitResolution::Valid(config),
                    Err(error) => TraversalWithoutLimitResolution::Invalid(error),
                },
            )
        })
    }

    pub(super) fn resolve_rewrite_rule_exprs_owned(
        &mut self,
        expr: Expr,
    ) -> Result<Option<Vec<Expr>>, EvalError> {
        let resolved_result = self.evaluate(expr);
        self.drive_evaluated_value(resolved_result, |session, resolved| {
            Ok(session.normalize_rewrite_rule_exprs_from_resolved(&resolved))
        })
    }

    pub(super) fn drive_rewrite_rule_exprs_owned<U, F>(
        &mut self,
        expr: Expr,
        cont: F,
    ) -> Result<U, EvalError>
    where
        U: ContinuationValue + 'static,
        F: FnOnce(&mut Session, Option<Vec<Expr>>) -> Result<U, EvalError> + 'static,
    {
        let resolved_rule_exprs_result = self.resolve_rewrite_rule_exprs_owned(expr);
        self.drive_internal_value(resolved_rule_exprs_result, cont)
    }

    pub(super) fn normalize_rewrite_rule_exprs_from_resolved(
        &self,
        resolved: &Expr,
    ) -> Option<Vec<Expr>> {
        normalize_rewrite_rules(resolved).map(|rules| {
            rules
                .into_iter()
                .map(|rule| Self::rewrite_rule_expr(&rule))
                .collect()
        })
    }

    pub(super) fn compile_rewrite_rules_from_normalized_exprs(
        &self,
        rule_exprs: Vec<Expr>,
    ) -> Option<Vec<RewriteRule>> {
        self.compile_rewrite_rules_for_matching(&Expr::call(system_expr("List"), rule_exprs))
    }

    pub(super) fn build_definition_catalog_entries(
        &self,
        symbol: &Symbol,
        definition: &SymbolDefinition,
    ) -> Vec<DefinitionCatalogEntry> {
        let mut entries = Vec::new();

        if let Some(rhs) = &definition.own_value {
            entries.push(DefinitionCatalogEntry {
                kind: DefinitionKind::OwnValue,
                lhs: Expr::Atom(Atom::Symbol(symbol.clone())),
                rhs: rhs.clone(),
                delayed: false,
            });
        }

        if let Some(rhs) = &definition.delayed_own_value {
            entries.push(DefinitionCatalogEntry {
                kind: DefinitionKind::DelayedOwnValue,
                lhs: Expr::Atom(Atom::Symbol(symbol.clone())),
                rhs: rhs.clone(),
                delayed: true,
            });
        }

        entries.extend(
            definition
                .up_values
                .iter()
                .map(|up_value| DefinitionCatalogEntry {
                    kind: DefinitionKind::UpValue,
                    lhs: up_value.lhs.clone(),
                    rhs: up_value.rhs.clone(),
                    delayed: up_value.delayed,
                }),
        );

        entries.extend(
            definition
                .down_values
                .iter()
                .map(|down_value| DefinitionCatalogEntry {
                    kind: DefinitionKind::DownValue,
                    lhs: down_value.lhs.clone(),
                    rhs: down_value.rhs.clone(),
                    delayed: down_value.delayed,
                }),
        );

        entries.extend(
            definition
                .sub_values
                .iter()
                .map(|sub_value| DefinitionCatalogEntry {
                    kind: DefinitionKind::SubValue,
                    lhs: sub_value.lhs.clone(),
                    rhs: sub_value.rhs.clone(),
                    delayed: sub_value.delayed,
                }),
        );

        entries
    }

    pub(super) fn definition_catalog_entries_for_symbol(
        &self,
        symbol: &Symbol,
    ) -> Vec<DefinitionCatalogEntry> {
        self.definitions
            .get(symbol)
            .map(|definition| self.build_definition_catalog_entries(symbol, definition))
            .unwrap_or_default()
    }

    pub(super) fn definition_rule_expr(entry: &DefinitionCatalogEntry) -> Expr {
        Expr::call(
            system_expr("RuleDelayed"),
            vec![
                Expr::call(system_expr("HoldPattern"), vec![entry.lhs.clone()]),
                entry.rhs.clone(),
            ],
        )
    }

    pub(super) fn own_values_expr_for_symbol(&self, symbol: &Symbol) -> Expr {
        let items = self
            .definition_catalog_entries_for_symbol(symbol)
            .into_iter()
            .filter(|entry| {
                matches!(
                    entry.kind,
                    DefinitionKind::OwnValue | DefinitionKind::DelayedOwnValue
                )
            })
            .map(|entry| Self::definition_rule_expr(&entry))
            .collect::<Vec<_>>();

        Expr::call(system_expr("List"), items)
    }

    pub(super) fn down_values_expr_for_symbol(&self, symbol: &Symbol) -> Expr {
        let items = self
            .definition_catalog_entries_for_symbol(symbol)
            .into_iter()
            .filter(|entry| entry.kind == DefinitionKind::DownValue)
            .map(|entry| Self::definition_rule_expr(&entry))
            .collect::<Vec<_>>();

        Expr::call(system_expr("List"), items)
    }

    pub(super) fn up_values_expr_for_symbol(&self, symbol: &Symbol) -> Expr {
        let items = self
            .definition_catalog_entries_for_symbol(symbol)
            .into_iter()
            .filter(|entry| entry.kind == DefinitionKind::UpValue)
            .map(|entry| Self::definition_rule_expr(&entry))
            .collect::<Vec<_>>();

        Expr::call(system_expr("List"), items)
    }

    pub(super) fn sub_values_expr_for_symbol(&self, symbol: &Symbol) -> Expr {
        let items = self
            .definition_catalog_entries_for_symbol(symbol)
            .into_iter()
            .filter(|entry| entry.kind == DefinitionKind::SubValue)
            .map(|entry| Self::definition_rule_expr(&entry))
            .collect::<Vec<_>>();

        Expr::call(system_expr("List"), items)
    }

    pub(super) fn rewrite_rule_expr(rule: &RewriteRule) -> Expr {
        Expr::call(
            system_expr(if rule.delayed { "RuleDelayed" } else { "Rule" }),
            vec![rule.lhs.clone(), rule.rhs.clone()],
        )
    }

    pub(super) fn normalized_rules_expr(rules: &[RewriteRule]) -> Expr {
        Expr::call(
            system_expr("List"),
            rules.iter().map(Self::rewrite_rule_expr).collect(),
        )
    }

    pub(super) fn compile_rewrite_rules_for_matching(
        &self,
        expr: &Expr,
    ) -> Option<Vec<RewriteRule>> {
        match expr {
            Expr::Call { head, args } if is_system_symbol_name(head, "List") => args
                .iter()
                .map(|item| self.compile_rewrite_rule_for_matching(item))
                .collect(),
            _ => self
                .compile_rewrite_rule_for_matching(expr)
                .map(|rule| vec![rule]),
        }
    }

    pub(super) fn compile_rewrite_rule_for_matching(&self, expr: &Expr) -> Option<RewriteRule> {
        let mut rule = normalize_single_rewrite_rule(expr)?;
        rule.matcher = self.compile_matcher_for_expr(&rule.lhs);
        Some(rule)
    }

    pub(super) fn compile_matcher_for_expr(&self, expr: &Expr) -> MatcherNode {
        compile_matcher(&self.normalize_match_expr(expr))
    }

    fn resume_resolve_argument_exprs_after_first(
        &mut self,
        first: Expr,
        rest: Vec<Expr>,
    ) -> Result<Vec<Expr>, EvalError> {
        let rest_result = self.resolve_argument_exprs_owned(rest);
        self.drive_internal_value(rest_result, move |_, mut resolved_rest| {
            let mut resolved = Vec::with_capacity(resolved_rest.len() + 1);
            resolved.push(first);
            resolved.append(&mut resolved_rest);
            Ok(resolved)
        })
    }
}

fn should_evaluate_single_rule_candidate(expr: &Expr) -> bool {
    match expr {
        Expr::Atom(Atom::Symbol(_)) => true,
        Expr::Call { head, .. } => {
            is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed")
        }
        _ => false,
    }
}
