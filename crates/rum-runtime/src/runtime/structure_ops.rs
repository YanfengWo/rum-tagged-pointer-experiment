use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AtomicPartSpec {
    Index(isize),
    All,
    Span(PartSpanSpec),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PartSpanSpec {
    start: SpanEndpoint,
    end: SpanEndpoint,
    step: isize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SpanEndpoint {
    Index(isize),
    All,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StructuralSequenceSpec {
    All,
    None,
    Count(isize),
    Single(isize),
    Range {
        start: isize,
        end: isize,
        step: isize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PartSelector {
    Single(AtomicPartSpec),
    Key(Expr),
    Multi(Vec<PartSelector>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReplacePartRule {
    position_specs: Vec<ReplacePartPositionSpec>,
    rhs: Expr,
    delayed: bool,
    heads: ReplacePartHeadsMode,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ReplacePartPositionSpec {
    Exact(Vec<PartSelector>),
    Pattern(Expr),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ReplacePlanNode {
    action: Option<ReplacePlanAction>,
    children: BTreeMap<isize, ReplacePlanNode>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ReplacePlanAction {
    Immediate(Expr),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReplacePartHeadsMode {
    Automatic,
    Include,
    Exclude,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SelectedPartShape {
    Leaf(Expr),
    Branch(Vec<SelectedPartShape>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReplacePartTarget {
    pub(super) path: Vec<isize>,
    pub(super) rhs: Expr,
    pub(super) delayed: bool,
}

impl ContinuationValue for Vec<ReplacePartTarget> {
    fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationReplacePartTargets"),
            self.into_iter()
                .map(|target| {
                    Expr::call(
                        system_expr("List"),
                        vec![
                            replace_part_position_expr(&target.path),
                            target.rhs,
                            system_expr(if target.delayed { "True" } else { "False" }),
                        ],
                    )
                })
                .collect(),
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, args }
                if is_system_symbol_name(&head, "InternalContinuationReplacePartTargets") =>
            {
                args.into_iter()
                    .map(|item| match item {
                        Expr::Call { head, args }
                            if is_system_symbol_name(&head, "List") && args.len() == 3 =>
                        {
                            let mut parts = args.into_iter();
                            let path_expr = parts
                                .next()
                                .expect("replace-part target payload stores a path");
                            let rhs = parts
                                .next()
                                .expect("replace-part target payload stores a rhs");
                            let delayed_expr = parts
                                .next()
                                .expect("replace-part target payload stores a delayed flag");

                            ReplacePartTarget {
                                path: replace_part_path_from_expr(path_expr),
                                rhs,
                                delayed: is_system_symbol_name(&delayed_expr, "True"),
                            }
                        }
                        other => {
                            panic!("unexpected continuation item for ReplacePartTarget: {other:?}")
                        }
                    })
                    .collect()
            }
            other => panic!("unexpected continuation payload for ReplacePartTarget: {other:?}"),
        }
    }
}

impl Session {
    pub(super) fn eval_head(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [expr] => Ok(structural_head_expr(expr)),
            [expr, wrapper] => Ok(Expr::call(
                wrapper.clone(),
                vec![structural_head_expr(expr)],
            )),
            _ => Ok(self.invalid_call_with_arg_range("Head", args, 1, 2)),
        }
    }

    pub(super) fn eval_length(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Length", args, 1));
        };

        Ok(Expr::integer(BigInt::from(structural_length(expr))))
    }

    pub(super) fn eval_depth(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let (expr, include_heads) = match args {
            [expr] => (expr, false),
            [expr, option] => {
                let Some(include_heads) = parse_heads_option(option) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "Depth",
                        args,
                        "expected a Heads -> True/False option as the second argument",
                    ));
                };
                (expr, include_heads)
            }
            _ => return Ok(self.invalid_call_with_arg_range("Depth", args, 1, 2)),
        };

        Ok(Expr::integer(BigInt::from(structural_depth(
            expr,
            include_heads,
        ))))
    }

    pub(super) fn eval_full_form(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("FullForm", args, 1));
        };

        Ok(Expr::string(expr.to_string()))
    }

    pub(super) fn eval_first(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [expr] => Ok(self.resolve_first_like(expr, None, "First", "first", true)),
            [expr, default] => {
                Ok(self.resolve_first_like(expr, Some(default), "First", "first", true))
            }
            _ => Ok(self.invalid_call_with_arg_range("First", args, 1, 2)),
        }
    }

    pub(super) fn eval_last(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [expr] => Ok(self.resolve_first_like(expr, None, "Last", "nolast", false)),
            [expr, default] => {
                Ok(self.resolve_first_like(expr, Some(default), "Last", "nolast", false))
            }
            _ => Ok(self.invalid_call_with_arg_range("Last", args, 1, 2)),
        }
    }

    pub(super) fn eval_rest(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Rest", args, 1));
        };

        Ok(match expr {
            Expr::Atom(_) => self.invalid_normal_expression_call("Rest", args),
            Expr::Call { head, args } => Expr::call(
                head.as_ref().clone(),
                args.iter().skip(1).cloned().collect::<Vec<_>>(),
            ),
        })
    }

    pub(super) fn eval_most(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Most", args, 1));
        };

        Ok(match expr {
            Expr::Atom(_) => self.invalid_normal_expression_call("Most", args),
            Expr::Call { head, args } => Expr::call(
                head.as_ref().clone(),
                args.iter()
                    .take(args.len().saturating_sub(1))
                    .cloned()
                    .collect::<Vec<_>>(),
            ),
        })
    }

    pub(super) fn eval_take(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() < 2 {
            return Ok(self.invalid_call_with_message(
                "Take",
                args,
                "argx",
                "expected at least 2 arguments",
            ));
        }

        let Some(specs) = parse_structural_sequence_specs(&args[1..]) else {
            return Ok(Expr::call(system_expr("Take"), args.to_vec()));
        };

        match apply_take_specs(&args[0], &specs) {
            Some(result) => Ok(result),
            None if matches!(args[0], Expr::Atom(_)) => {
                Ok(self.invalid_normal_expression_call("Take", args))
            }
            None => Ok(self.invalid_take_drop_sequence_call("Take", "take", args)),
        }
    }

    pub(super) fn eval_drop(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() < 2 {
            return Ok(self.invalid_call_with_message(
                "Drop",
                args,
                "argx",
                "expected at least 2 arguments",
            ));
        }

        let Some(specs) = parse_structural_sequence_specs(&args[1..]) else {
            return Ok(Expr::call(system_expr("Drop"), args.to_vec()));
        };

        match apply_drop_specs(&args[0], &specs) {
            Some(result) => Ok(result),
            None if matches!(args[0], Expr::Atom(_)) => {
                Ok(self.invalid_normal_expression_call("Drop", args))
            }
            None => Ok(self.invalid_take_drop_sequence_call("Drop", "drop", args)),
        }
    }

    pub(super) fn eval_take_drop(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() < 2 {
            return Ok(self.invalid_call_with_message(
                "TakeDrop",
                args,
                "argx",
                "expected at least 2 arguments",
            ));
        }

        let Some(specs) = parse_structural_sequence_specs(&args[1..]) else {
            return Ok(Expr::call(system_expr("TakeDrop"), args.to_vec()));
        };

        match (
            apply_take_specs(&args[0], &specs),
            apply_drop_specs(&args[0], &specs),
        ) {
            (Some(taken), Some(dropped)) => {
                Ok(Expr::call(system_expr("List"), vec![taken, dropped]))
            }
            _ if matches!(args[0], Expr::Atom(_)) => {
                Ok(self.invalid_normal_expression_call("TakeDrop", args))
            }
            _ => Ok(self.invalid_call_with_message(
                "TakeDrop",
                args,
                "take",
                format!(
                    "Cannot apply the requested sequence specification in {}.",
                    Expr::call(system_expr("TakeDrop"), args.to_vec())
                ),
            )),
        }
    }

    pub(super) fn eval_insert(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [expr, item, position] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Insert", args, 3));
        };

        let Some(position) = expr_as_signed_isize(position) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Expr::Call {
            head: expr_head,
            args: expr_args,
        } = expr
        else {
            return Ok(self.invalid_normal_expression_call("Insert", args));
        };

        let Some(index) = normalize_insert_arg_position(position, expr_args.len()) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let mut inserted = expr_args.clone();
        inserted.insert(index, item.clone());
        Ok(Expr::call(expr_head.as_ref().clone(), inserted))
    }

    pub(super) fn eval_delete(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [expr, positions_expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Delete", args, 2));
        };

        let Expr::Call {
            head: expr_head,
            args: expr_args,
        } = expr
        else {
            return Ok(self.invalid_normal_expression_call("Delete", args));
        };

        let Some(positions) = parse_delete_positions(positions_expr, expr_args.len()) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let dropped = positions.into_iter().collect::<BTreeSet<_>>();
        let kept = expr_args
            .iter()
            .enumerate()
            .filter(|(index, _)| !dropped.contains(index))
            .map(|(_, arg)| arg.clone())
            .collect::<Vec<_>>();
        Ok(Expr::call(expr_head.as_ref().clone(), kept))
    }

    pub(super) fn eval_take_list(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [expr, counts] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("TakeList", args, 2));
        };

        let Expr::Call { .. } = expr else {
            return Ok(self.invalid_normal_expression_call("TakeList", args));
        };

        let Some(counts) = parse_take_list_counts(counts) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Some(chunks) = apply_take_list_counts(expr, &counts) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::call(system_expr("List"), chunks))
    }

    pub(super) fn eval_append(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [expr, item] => Ok(match expr {
                Expr::Atom(_) => self.invalid_normal_expression_call("Append", args),
                Expr::Call { head, args } => {
                    let mut new_args = args.clone();
                    new_args.push(item.clone());
                    Expr::call(head.as_ref().clone(), new_args)
                }
            }),
            [item] => Ok(unary_operator_form("Append", item.clone())),
            _ => Ok(self.invalid_call_with_arg_range("Append", args, 1, 2)),
        }
    }

    pub(super) fn eval_prepend(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [expr, item] => Ok(match expr {
                Expr::Atom(_) => self.invalid_normal_expression_call("Prepend", args),
                Expr::Call { head, args } => {
                    let mut new_args = Vec::with_capacity(args.len() + 1);
                    new_args.push(item.clone());
                    new_args.extend(args.clone());
                    Expr::call(head.as_ref().clone(), new_args)
                }
            }),
            [item] => Ok(unary_operator_form("Prepend", item.clone())),
            _ => Ok(self.invalid_call_with_arg_range("Prepend", args, 1, 2)),
        }
    }

    pub(super) fn eval_symbol(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [name_expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Symbol", args, 1));
        };

        let Some(name) = expr_as_string(name_expr) else {
            return Ok(self.invalid_call_with_arg_form(
                "Symbol",
                args,
                "expected a string symbol name",
            ));
        };

        Ok(self
            .resolve_symbol_name(name, true)
            .map(|symbol| Expr::Atom(Atom::Symbol(symbol)))
            .unwrap_or_else(|| Expr::call(head.clone(), args.to_vec())))
    }

    pub(super) fn eval_construct(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.is_empty() {
            return Ok(self.invalid_call_with_message(
                "Construct",
                args,
                "argx",
                "expected at least 1 argument",
            ));
        }

        Ok(Expr::call(args[0].clone(), args[1..].to_vec()))
    }

    fn resolve_first_like(
        &mut self,
        expr: &Expr,
        default: Option<&Expr>,
        symbol: &str,
        empty_tag: &str,
        take_first: bool,
    ) -> Expr {
        match expr {
            Expr::Call { args, .. } if !args.is_empty() => {
                if take_first {
                    args[0].clone()
                } else {
                    args[args.len() - 1].clone()
                }
            }
            Expr::Call { .. } | Expr::Atom(_) if default.is_some() => {
                default.expect("checked above").clone()
            }
            Expr::Call { .. } => {
                self.invalid_empty_structural_selection_call(symbol, empty_tag, expr)
            }
            Expr::Atom(_) => self.invalid_normal_expression_call(symbol, &[expr.clone()]),
        }
    }

    fn invalid_empty_structural_selection_call(
        &mut self,
        symbol: &str,
        tag: &str,
        expr: &Expr,
    ) -> Expr {
        self.push_message(
            symbol,
            tag,
            format!(
                "{} has zero length and no {} element.",
                expr,
                selection_name(symbol)
            ),
        );
        Expr::call(system_expr(symbol), vec![expr.clone()])
    }

    fn invalid_normal_expression_call(&mut self, symbol: &str, args: &[Expr]) -> Expr {
        self.push_message(
            heads::messages::GENERAL,
            "normal",
            format!(
                "Nonatomic expression expected at position 1 in {}.",
                Expr::call(system_expr(symbol), args.to_vec())
            ),
        );
        Expr::call(system_expr(symbol), args.to_vec())
    }

    fn invalid_take_drop_sequence_call(&mut self, symbol: &str, tag: &str, args: &[Expr]) -> Expr {
        self.push_message(
            symbol,
            tag,
            format!(
                "Cannot apply the requested sequence specification in {}.",
                Expr::call(system_expr(symbol), args.to_vec())
            ),
        );
        Expr::call(system_expr(symbol), args.to_vec())
    }

    pub(super) fn eval_part(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.is_empty() {
            return Ok(self.invalid_call_with_message(
                "Part",
                args,
                "argx",
                "expected at least 1 argument",
            ));
        }

        let Some(selectors) = parse_part_selectors(&args[1..]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(apply_part_selectors(&args[0], &selectors)
            .unwrap_or_else(|| Expr::call(head.clone(), args.to_vec())))
    }

    pub(super) fn eval_extract(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(2..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Extract", args, 2, 3));
        }

        let Some(position_spec) = parse_extract_position_spec(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let extracted = if position_spec.multiple {
            let mut items = Vec::new();
            for selectors in &position_spec.position_specs {
                let Some(value) = apply_part_selectors(&args[0], selectors) else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                };
                items.push(wrap_extracted_value(value, args.get(2)));
            }
            Expr::call(system_expr("List"), items)
        } else {
            let Some(value) = apply_part_selectors(&args[0], &position_spec.position_specs[0])
            else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            wrap_extracted_value(value, args.get(2))
        };

        Ok(extracted)
    }

    pub(super) fn eval_replace_part(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(2..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("ReplacePart", args, 2, 3));
        }

        let heads = if let Some(option_expr) = args.get(2) {
            let Some(heads) = parse_replace_part_heads_option(option_expr) else {
                return Ok(self.invalid_call_with_arg_form(
                    "ReplacePart",
                    args,
                    "expected a Heads -> True/False option as the third argument",
                ));
            };
            heads
        } else {
            ReplacePartHeadsMode::Automatic
        };

        let Some(rules) = parse_replace_part_rules(&args[1], heads) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        self.apply_replace_part_rules_owned(args[0].clone(), rules)
    }

    fn apply_replace_part_rules_owned(
        &mut self,
        expr: Expr,
        rules: Vec<ReplacePartRule>,
    ) -> Result<Expr, EvalError> {
        let Some((first, rest)) = rules.split_first() else {
            return Ok(expr);
        };

        let rule = first.clone();
        let rest = rest.to_vec();
        let targets_result = self.resolve_rule_targets_owned(expr.clone(), rule.clone());
        self.drive_internal_value(targets_result, move |session, targets| {
            if targets.is_empty() {
                session.apply_replace_part_rules_owned(expr, rest)
            } else if rule.delayed {
                session.apply_delayed_replace_targets_owned(expr, targets, rest)
            } else {
                let plan = build_replace_plan_from_targets(targets);
                let updated = session.apply_replace_plan_owned(expr, plan)?;
                session.apply_replace_part_rules_owned(updated, rest)
            }
        })
    }

    fn resolve_rule_targets_owned(
        &mut self,
        expr: Expr,
        rule: ReplacePartRule,
    ) -> Result<Vec<ReplacePartTarget>, EvalError> {
        self.collect_rule_targets_owned(
            expr,
            rule.position_specs,
            rule.rhs,
            rule.delayed,
            rule.heads,
            Vec::new(),
            0,
        )
    }

    fn collect_rule_targets_owned(
        &mut self,
        expr: Expr,
        position_specs: Vec<ReplacePartPositionSpec>,
        rhs: Expr,
        delayed: bool,
        heads: ReplacePartHeadsMode,
        mut collected: Vec<ReplacePartTarget>,
        index: usize,
    ) -> Result<Vec<ReplacePartTarget>, EvalError> {
        let Some(position_spec) = position_specs.get(index).cloned() else {
            return Ok(resolve_conflicting_replace_part_targets(collected));
        };

        let targets_result = self.collect_position_spec_targets_owned(
            expr.clone(),
            position_spec,
            rhs.clone(),
            delayed,
            heads,
        );
        self.drive_internal_value(targets_result, move |session, mut targets| {
            collected.append(&mut targets);
            session.collect_rule_targets_owned(
                expr,
                position_specs,
                rhs,
                delayed,
                heads,
                collected,
                index + 1,
            )
        })
    }

    fn collect_position_spec_targets_owned(
        &mut self,
        expr: Expr,
        position_spec: ReplacePartPositionSpec,
        rhs: Expr,
        delayed: bool,
        heads: ReplacePartHeadsMode,
    ) -> Result<Vec<ReplacePartTarget>, EvalError> {
        match position_spec {
            ReplacePartPositionSpec::Exact(selectors) => Ok(collect_exact_paths_for_position_spec(
                &expr, &selectors, heads,
            )
            .into_iter()
            .map(|path| ReplacePartTarget {
                path,
                rhs: rhs.clone(),
                delayed,
            })
            .collect()),
            ReplacePartPositionSpec::Pattern(pattern_expr) => {
                self.collect_pattern_position_targets_owned(expr, pattern_expr, rhs, delayed, heads)
            }
        }
    }

    fn collect_pattern_position_targets_owned(
        &mut self,
        expr: Expr,
        pattern_expr: Expr,
        rhs: Expr,
        delayed: bool,
        heads: ReplacePartHeadsMode,
    ) -> Result<Vec<ReplacePartTarget>, EvalError> {
        let pattern_result = self.resolve_matcher_pattern_expr_owned(pattern_expr);
        self.drive_internal_value(pattern_result, move |session, resolved_pattern| {
            let pattern = wrap_replace_part_pattern_position(resolved_pattern);
            let include_heads = replace_part_pattern_includes_heads(&pattern, heads);
            let candidates = collect_replace_part_candidate_paths(&expr, include_heads);
            session.collect_pattern_position_targets_from_candidates_owned(
                pattern,
                candidates,
                rhs,
                delayed,
                Vec::new(),
                0,
            )
        })
    }

    fn collect_pattern_position_targets_from_candidates_owned(
        &mut self,
        pattern: Expr,
        candidates: Vec<Vec<isize>>,
        rhs: Expr,
        delayed: bool,
        mut collected: Vec<ReplacePartTarget>,
        index: usize,
    ) -> Result<Vec<ReplacePartTarget>, EvalError> {
        let Some(path) = candidates.get(index).cloned() else {
            return Ok(collected);
        };

        let position = replace_part_position_expr(&path);
        let match_result = self.match_expr_owned(pattern.clone(), position, BTreeMap::new());
        self.drive_internal_value(match_result, move |session, outcome| {
            if outcome.matched {
                collected.push(ReplacePartTarget {
                    path,
                    rhs: substitute_rule_bindings(&rhs, &outcome.bindings),
                    delayed,
                });
            }

            session.collect_pattern_position_targets_from_candidates_owned(
                pattern,
                candidates,
                rhs,
                delayed,
                collected,
                index + 1,
            )
        })
    }

    fn apply_delayed_replace_targets_owned(
        &mut self,
        expr: Expr,
        targets: Vec<ReplacePartTarget>,
        remaining_rules: Vec<ReplacePartRule>,
    ) -> Result<Expr, EvalError> {
        let Some((first_target, rest_targets)) = targets.split_first() else {
            return self.apply_replace_part_rules_owned(expr, remaining_rules);
        };

        let rhs_result = self.evaluate(first_target.rhs.clone());
        let first_path = first_target.path.clone();
        let rest_targets = rest_targets.to_vec();
        self.drive_evaluated_value::<Expr, Expr, _>(rhs_result, move |session, rhs| {
            let updated = replace_exact_path(expr, &first_path, rhs);
            session.apply_delayed_replace_targets_owned(updated, rest_targets, remaining_rules)
        })
    }

    fn apply_replace_plan_owned(
        &mut self,
        expr: Expr,
        mut plan: ReplacePlanNode,
    ) -> Result<Expr, EvalError> {
        if let Some(action) = plan.action.take() {
            return match action {
                ReplacePlanAction::Immediate(rhs) => Ok(rhs),
            };
        }

        match expr {
            Expr::Atom(_) => {
                if let Some(head_plan) = plan.children.remove(&0) {
                    let replaced_head =
                        self.apply_replace_plan_owned(structural_head_expr(&expr), head_plan)?;
                    Ok(Expr::call(replaced_head, Vec::new()))
                } else {
                    Ok(expr)
                }
            }
            Expr::Call { head, mut args } => {
                let mut replaced_head = head.as_ref().clone();
                if let Some(head_plan) = plan.children.remove(&0) {
                    replaced_head = self.apply_replace_plan_owned(replaced_head, head_plan)?;
                }

                for (index, arg) in args.iter_mut().enumerate() {
                    if let Some(child_plan) = plan.children.remove(&(index as isize + 1)) {
                        *arg = self.apply_replace_plan_owned(arg.clone(), child_plan)?;
                    }
                }

                Ok(Expr::call(replaced_head, args))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExtractPositionSpec {
    position_specs: Vec<Vec<PartSelector>>,
    multiple: bool,
}

fn parse_part_selectors(exprs: &[Expr]) -> Option<Vec<PartSelector>> {
    exprs.iter().map(parse_part_selector).collect()
}

fn parse_part_selector(expr: &Expr) -> Option<PartSelector> {
    if let Some(selector) = parse_scalar_part_selector(expr) {
        return Some(selector);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    Some(PartSelector::Multi(
        args.iter()
            .map(parse_part_selector)
            .collect::<Option<Vec<_>>>()?,
    ))
}

fn parse_scalar_part_selector(expr: &Expr) -> Option<PartSelector> {
    if let Some(spec) = parse_atomic_part_spec(expr) {
        return Some(PartSelector::Single(spec));
    }

    parse_key_part_selector(expr).map(PartSelector::Key)
}

fn parse_key_part_selector(expr: &Expr) -> Option<Expr> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Key") || args.len() != 1 {
        return None;
    }

    Some(args[0].clone())
}

fn parse_atomic_part_spec(expr: &Expr) -> Option<AtomicPartSpec> {
    if let Some(index) = expr_as_signed_isize(expr) {
        return Some(AtomicPartSpec::Index(index));
    }

    if is_system_symbol_name(expr, heads::structure::ALL) {
        return Some(AtomicPartSpec::All);
    }

    parse_span_part_spec(expr)
}

fn parse_span_part_spec(expr: &Expr) -> Option<AtomicPartSpec> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::SPAN) {
        return None;
    }

    let spec = match args.as_slice() {
        [value] => {
            let endpoint = parse_span_endpoint(value)?;
            PartSpanSpec {
                start: endpoint,
                end: endpoint,
                step: 1,
            }
        }
        [start, end] => PartSpanSpec {
            start: parse_span_endpoint(start)?,
            end: parse_span_endpoint(end)?,
            step: 1,
        },
        [start, end, step] => {
            let step = expr_as_signed_isize(step)?;
            if step == 0 {
                return None;
            }
            PartSpanSpec {
                start: parse_span_endpoint(start)?,
                end: parse_span_endpoint(end)?,
                step,
            }
        }
        _ => return None,
    };

    Some(AtomicPartSpec::Span(spec))
}

fn parse_span_endpoint(expr: &Expr) -> Option<SpanEndpoint> {
    if is_system_symbol_name(expr, heads::structure::ALL) {
        return Some(SpanEndpoint::All);
    }

    expr_as_signed_isize(expr).map(SpanEndpoint::Index)
}

fn apply_part_selectors(expr: &Expr, selectors: &[PartSelector]) -> Option<Expr> {
    let Some((first, rest)) = selectors.split_first() else {
        return Some(expr.clone());
    };

    match select_parts_for_selector(expr, first)? {
        SelectedPartShape::Leaf(selected) => apply_part_selectors(&selected, rest),
        SelectedPartShape::Branch(selected) => Some(selected_shape_to_expr(
            SelectedPartShape::Branch(apply_part_selectors_to_shape(selected, rest)?),
            Some(selected_shape_head_for_expr(expr, first)),
        )),
    }
}

fn selected_shape_head_for_expr(expr: &Expr, selector: &PartSelector) -> Expr {
    if association_entries_from_expr(expr).is_some() && selector_contains_key(selector) {
        system_expr("List")
    } else {
        structural_head_expr(expr)
    }
}

fn selector_contains_key(selector: &PartSelector) -> bool {
    match selector {
        PartSelector::Single(_) => false,
        PartSelector::Key(_) => true,
        PartSelector::Multi(specs) => specs.iter().any(selector_contains_key),
    }
}

fn position_spec_contains_key(selectors: &[PartSelector]) -> bool {
    selectors.iter().any(selector_contains_key)
}

pub(super) fn apply_keyed_part_spec(expr: &Expr, spec_expr: &Expr) -> Option<Expr> {
    let selectors = parse_single_position_spec(spec_expr)?;
    position_spec_contains_key(&selectors).then(|| apply_part_selectors(expr, &selectors))?
}

fn apply_part_selectors_to_shape(
    selected: Vec<SelectedPartShape>,
    selectors: &[PartSelector],
) -> Option<Vec<SelectedPartShape>> {
    selected
        .into_iter()
        .map(|shape| apply_part_selectors_to_selected_shape(shape, selectors))
        .collect()
}

fn apply_part_selectors_to_selected_shape(
    shape: SelectedPartShape,
    selectors: &[PartSelector],
) -> Option<SelectedPartShape> {
    match shape {
        SelectedPartShape::Leaf(expr) => Some(SelectedPartShape::Leaf(apply_part_selectors(
            &expr, selectors,
        )?)),
        SelectedPartShape::Branch(children) => Some(SelectedPartShape::Branch(
            children
                .into_iter()
                .map(|child| apply_part_selectors_to_selected_shape(child, selectors))
                .collect::<Option<Vec<_>>>()?,
        )),
    }
}

fn selected_shape_to_expr(shape: SelectedPartShape, root_head: Option<Expr>) -> Expr {
    match shape {
        SelectedPartShape::Leaf(expr) => expr,
        SelectedPartShape::Branch(children) => Expr::call(
            root_head.unwrap_or_else(|| system_expr("List")),
            children
                .into_iter()
                .map(|child| selected_shape_to_expr(child, None))
                .collect(),
        ),
    }
}

fn select_parts_for_selector(expr: &Expr, selector: &PartSelector) -> Option<SelectedPartShape> {
    match selector {
        PartSelector::Single(spec) => match spec {
            AtomicPartSpec::Index(index) => {
                Some(SelectedPartShape::Leaf(select_exact_part(expr, *index)?))
            }
            AtomicPartSpec::All | AtomicPartSpec::Span(_) => Some(SelectedPartShape::Branch(
                select_parts_for_atomic_spec(expr, spec)?
                    .into_iter()
                    .map(SelectedPartShape::Leaf)
                    .collect(),
            )),
        },
        PartSelector::Key(key) => association_value_by_key(expr, key).map(SelectedPartShape::Leaf),
        PartSelector::Multi(specs) => {
            let mut selected = Vec::new();
            for spec in specs {
                match (spec, select_parts_for_selector(expr, spec)?) {
                    (
                        PartSelector::Single(_) | PartSelector::Key(_),
                        SelectedPartShape::Leaf(expr),
                    ) => {
                        selected.push(SelectedPartShape::Leaf(expr));
                    }
                    (
                        PartSelector::Single(_) | PartSelector::Key(_),
                        SelectedPartShape::Branch(children),
                    ) => {
                        selected.extend(children);
                    }
                    (PartSelector::Multi(_), shape) => selected.push(shape),
                }
            }
            Some(SelectedPartShape::Branch(selected))
        }
    }
}

fn select_parts_for_atomic_spec(expr: &Expr, spec: &AtomicPartSpec) -> Option<Vec<Expr>> {
    select_positioned_parts_for_atomic_spec(expr, spec).map(|parts| {
        parts
            .into_iter()
            .map(|(_, value)| value)
            .collect::<Vec<_>>()
    })
}

fn select_positioned_parts_for_atomic_spec(
    expr: &Expr,
    spec: &AtomicPartSpec,
) -> Option<Vec<(isize, Expr)>> {
    match spec {
        AtomicPartSpec::Index(index) => {
            let value = select_exact_part(expr, *index)?;
            let component = exact_part_component(expr, *index)?;
            Some(vec![(component, value)])
        }
        AtomicPartSpec::All => match expr {
            Expr::Atom(_) => None,
            Expr::Call { args, .. } => Some(
                args.iter()
                    .enumerate()
                    .map(|(index, value)| (index as isize + 1, value.clone()))
                    .collect(),
            ),
        },
        AtomicPartSpec::Span(span) => {
            select_positioned_parts_for_span(expr, *span, part_span_includes_head(*span))
        }
    }
}

fn resolve_span_endpoint(
    endpoint: SpanEndpoint,
    len: usize,
    step: isize,
    include_head: bool,
    is_start: bool,
) -> Option<isize> {
    match endpoint {
        SpanEndpoint::All => {
            let len = isize::try_from(len).ok()?;
            Some(if step > 0 {
                if is_start {
                    if include_head {
                        0
                    } else {
                        1
                    }
                } else {
                    len
                }
            } else if is_start {
                len
            } else {
                if include_head {
                    0
                } else {
                    1
                }
            })
        }
        SpanEndpoint::Index(0) if include_head => Some(0),
        SpanEndpoint::Index(index) => normalize_clamped_structural_endpoint(index, len),
    }
}

fn select_positioned_parts_for_span(
    expr: &Expr,
    spec: PartSpanSpec,
    include_head: bool,
) -> Option<Vec<(isize, Expr)>> {
    match expr {
        Expr::Atom(_) => {
            let components = resolve_span_components(spec, 0, include_head)?;
            Some(
                components
                    .into_iter()
                    .filter_map(|component| {
                        if component == 0 {
                            Some((0, structural_head_expr(expr)))
                        } else {
                            None
                        }
                    })
                    .collect(),
            )
        }
        Expr::Call { args, .. } => {
            let components = resolve_span_components(spec, args.len(), include_head)?;
            Some(
                components
                    .into_iter()
                    .filter_map(|component| {
                        if component == 0 {
                            Some((0, structural_head_expr(expr)))
                        } else {
                            let index = normalize_structural_arg_position(component, args.len())?;
                            Some((component, args[index].clone()))
                        }
                    })
                    .collect(),
            )
        }
    }
}

fn part_span_includes_head(spec: PartSpanSpec) -> bool {
    matches!(spec.start, SpanEndpoint::Index(0)) || matches!(spec.end, SpanEndpoint::Index(0))
}

fn resolve_span_components(
    spec: PartSpanSpec,
    len: usize,
    include_head: bool,
) -> Option<Vec<isize>> {
    let start = resolve_span_endpoint(spec.start, len, spec.step, include_head, true)?;
    let end = resolve_span_endpoint(spec.end, len, spec.step, include_head, false)?;
    let mut current = start;
    let mut components = Vec::new();

    if spec.step > 0 {
        while current <= end {
            components.push(current);
            current += spec.step;
        }
    } else {
        while current >= end {
            components.push(current);
            current += spec.step;
        }
    }

    Some(components)
}

fn normalize_clamped_structural_endpoint(position: isize, len: usize) -> Option<isize> {
    if position == 0 || len == 0 {
        return None;
    }

    let len = isize::try_from(len).ok()?;
    let normalized = if position > 0 {
        position
    } else {
        len + position + 1
    };

    Some(normalized.clamp(1, len))
}

fn parse_extract_position_spec(expr: &Expr) -> Option<ExtractPositionSpec> {
    if is_empty_plain_list_expr(expr) {
        return Some(ExtractPositionSpec {
            position_specs: Vec::new(),
            multiple: true,
        });
    }

    if let Some(position_spec) = parse_single_position_spec(expr) {
        return Some(ExtractPositionSpec {
            position_specs: vec![position_spec],
            multiple: false,
        });
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    let position_specs = args
        .iter()
        .map(parse_single_position_spec)
        .collect::<Option<Vec<_>>>()?;
    Some(ExtractPositionSpec {
        position_specs,
        multiple: true,
    })
}

fn parse_single_position_spec(expr: &Expr) -> Option<Vec<PartSelector>> {
    if let Some(selector) = parse_scalar_part_selector(expr) {
        return Some(vec![selector]);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    if args.is_empty() {
        return Some(Vec::new());
    }

    if args.iter().all(is_plain_list_expr) {
        return None;
    }

    args.iter().map(parse_part_selector).collect()
}

pub(super) fn exact_position_paths_from_expr(
    expr: &Expr,
    spec_expr: &Expr,
) -> Option<Vec<Vec<isize>>> {
    if let Some(position_spec) = parse_single_position_spec(spec_expr) {
        return Some(collect_exact_paths_for_position_spec(
            expr,
            &position_spec,
            ReplacePartHeadsMode::Automatic,
        ));
    }

    let Expr::Call { head, args } = spec_expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    let mut paths = Vec::new();
    for item in args {
        let position_spec = parse_single_position_spec(item)?;
        paths.extend(collect_exact_paths_for_position_spec(
            expr,
            &position_spec,
            ReplacePartHeadsMode::Automatic,
        ));
    }
    Some(paths)
}

fn is_plain_list_expr(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call { head, .. } if is_system_symbol_name(head, heads::structure::LIST)
    )
}

fn is_empty_plain_list_expr(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call { head, args }
            if is_system_symbol_name(head, heads::structure::LIST) && args.is_empty()
    )
}

fn parse_replace_part_rules(
    expr: &Expr,
    heads: ReplacePartHeadsMode,
) -> Option<Vec<ReplacePartRule>> {
    if let Some(rule) = parse_replace_part_rule_with_heads(expr, heads) {
        return Some(vec![rule]);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    args.iter()
        .map(|item| parse_replace_part_rule_with_heads(item, heads))
        .collect()
}

fn parse_replace_part_rule_with_heads(
    expr: &Expr,
    heads: ReplacePartHeadsMode,
) -> Option<ReplacePartRule> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    let delayed = if is_system_symbol_name(head, heads::rules::RULE) && args.len() == 2 {
        false
    } else if is_system_symbol_name(head, heads::rules::RULE_DELAYED) && args.len() == 2 {
        true
    } else {
        return None;
    };

    Some(ReplacePartRule {
        position_specs: parse_replace_part_position_specs(&args[0])?,
        rhs: args[1].clone(),
        delayed,
        heads,
    })
}

fn parse_replace_part_position_specs(expr: &Expr) -> Option<Vec<ReplacePartPositionSpec>> {
    if is_empty_plain_list_expr(expr) {
        return Some(Vec::new());
    }

    if let Some(position_spec) = parse_single_position_spec(expr) {
        return Some(vec![ReplacePartPositionSpec::Exact(position_spec)]);
    }

    let Expr::Call { head, args } = expr else {
        return Some(vec![ReplacePartPositionSpec::Pattern(expr.clone())]);
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return Some(vec![ReplacePartPositionSpec::Pattern(expr.clone())]);
    }

    if !args.iter().any(is_plain_list_expr) {
        return Some(vec![ReplacePartPositionSpec::Pattern(expr.clone())]);
    }

    Some(
        args.iter()
            .map(|item| {
                parse_single_position_spec(item)
                    .map(ReplacePartPositionSpec::Exact)
                    .unwrap_or_else(|| ReplacePartPositionSpec::Pattern(item.clone()))
            })
            .collect(),
    )
}

fn wrap_extracted_value(value: Expr, wrapper: Option<&Expr>) -> Expr {
    wrapper
        .cloned()
        .map(|head| Expr::call(head, vec![value.clone()]))
        .unwrap_or(value)
}

fn select_exact_part(expr: &Expr, index: isize) -> Option<Expr> {
    if index == 0 {
        return Some(structural_head_expr(expr));
    }

    match expr {
        Expr::Atom(_) => None,
        Expr::Call { args, .. } => {
            let normalized = normalize_structural_arg_position(index, args.len())?;
            Some(args[normalized].clone())
        }
    }
}

fn exact_part_component(expr: &Expr, index: isize) -> Option<isize> {
    if index == 0 {
        return Some(0);
    }

    match expr {
        Expr::Atom(_) => None,
        Expr::Call { args, .. } => {
            Some(normalize_structural_arg_position(index, args.len())? as isize + 1)
        }
    }
}

fn normalize_structural_arg_position(position: isize, len: usize) -> Option<usize> {
    if position == 0 {
        return None;
    }

    let len = isize::try_from(len).ok()?;
    let normalized = if position > 0 {
        position - 1
    } else {
        len + position
    };

    (0..len)
        .contains(&normalized)
        .then_some(normalized as usize)
}

fn structural_head_expr(expr: &Expr) -> Expr {
    match expr {
        Expr::Atom(Atom::Integer(_)) => system_expr("Integer"),
        Expr::Atom(Atom::Rational(_)) => system_expr("Rational"),
        Expr::Atom(Atom::String(_)) => system_expr("String"),
        Expr::Atom(Atom::Symbol(_)) => system_expr("Symbol"),
        Expr::Call { head, .. } => head.as_ref().clone(),
    }
}

fn structural_length(expr: &Expr) -> usize {
    match expr {
        Expr::Atom(_) => 0,
        Expr::Call { args, .. } => args.len(),
    }
}

fn structural_depth(expr: &Expr, include_heads: bool) -> usize {
    match expr {
        Expr::Atom(_) => 1,
        Expr::Call { head, args } => {
            let head_depth = include_heads
                .then(|| structural_depth(head, true))
                .unwrap_or(0);
            let arg_depth = args
                .iter()
                .map(|arg| structural_depth(arg, include_heads))
                .max()
                .unwrap_or(0);
            1 + head_depth.max(arg_depth)
        }
    }
}

fn parse_structural_sequence_specs(args: &[Expr]) -> Option<Vec<StructuralSequenceSpec>> {
    args.iter().map(parse_structural_sequence_spec).collect()
}

fn parse_structural_sequence_spec(expr: &Expr) -> Option<StructuralSequenceSpec> {
    if is_system_symbol_name(expr, heads::structure::ALL) {
        return Some(StructuralSequenceSpec::All);
    }

    if is_system_symbol_name(expr, heads::structure::NONE) {
        return Some(StructuralSequenceSpec::None);
    }

    if let Some(count) = expr_as_signed_isize(expr) {
        return Some(StructuralSequenceSpec::Count(count));
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    match args.as_slice() {
        [index] => Some(StructuralSequenceSpec::Single(expr_as_signed_isize(index)?)),
        [start, end] => Some(StructuralSequenceSpec::Range {
            start: expr_as_signed_isize(start)?,
            end: expr_as_signed_isize(end)?,
            step: 1,
        }),
        [start, end, step] => Some(StructuralSequenceSpec::Range {
            start: expr_as_signed_isize(start)?,
            end: expr_as_signed_isize(end)?,
            step: expr_as_signed_isize(step)?,
        }),
        _ => None,
    }
}

fn apply_take_specs(expr: &Expr, specs: &[StructuralSequenceSpec]) -> Option<Expr> {
    let Some((first, rest)) = specs.split_first() else {
        return Some(expr.clone());
    };

    let Expr::Call { .. } = expr else {
        return None;
    };

    let components = resolve_take_components(*first, structural_length(expr))?;
    let selectors = vec![PartSelector::Multi(
        components
            .into_iter()
            .map(|component| PartSelector::Single(AtomicPartSpec::Index(component)))
            .collect(),
    )];
    let selected = apply_part_selectors(expr, &selectors)?;

    if rest.is_empty() {
        Some(selected)
    } else {
        apply_take_drop_rest_to_expr(selected, rest, apply_take_specs)
    }
}

fn apply_drop_specs(expr: &Expr, specs: &[StructuralSequenceSpec]) -> Option<Expr> {
    let Some((first, rest)) = specs.split_first() else {
        return Some(expr.clone());
    };

    let Expr::Call { head, args } = expr else {
        return None;
    };

    let dropped = resolve_take_components(*first, args.len())?
        .into_iter()
        .collect::<BTreeSet<_>>();
    let kept_args = args
        .iter()
        .enumerate()
        .filter(|(index, _)| !dropped.contains(&(*index as isize + 1)))
        .map(|(_, arg)| arg.clone())
        .collect::<Vec<_>>();
    let kept = Expr::call(head.as_ref().clone(), kept_args);

    if rest.is_empty() {
        Some(kept)
    } else {
        apply_take_drop_rest_to_expr(kept, rest, apply_drop_specs)
    }
}

fn apply_take_drop_rest_to_expr(
    expr: Expr,
    rest: &[StructuralSequenceSpec],
    recurse: fn(&Expr, &[StructuralSequenceSpec]) -> Option<Expr>,
) -> Option<Expr> {
    match expr {
        Expr::Atom(_) => None,
        Expr::Call { head, args } => Some(Expr::call(
            head.as_ref().clone(),
            args.iter()
                .map(|arg| recurse(arg, rest))
                .collect::<Option<Vec<_>>>()?,
        )),
    }
}

fn resolve_take_components(spec: StructuralSequenceSpec, len: usize) -> Option<Vec<isize>> {
    match spec {
        StructuralSequenceSpec::All => Some((1..=len).map(|index| index as isize).collect()),
        StructuralSequenceSpec::None => Some(Vec::new()),
        StructuralSequenceSpec::Count(0) => Some(Vec::new()),
        StructuralSequenceSpec::Count(count) if count > 0 => {
            let count = usize::try_from(count).ok()?;
            (count <= len).then(|| (1..=count).map(|index| index as isize).collect())
        }
        StructuralSequenceSpec::Count(count) => {
            let count = count.unsigned_abs();
            if count > len {
                None
            } else {
                Some(
                    ((len - count + 1)..=len)
                        .map(|index| index as isize)
                        .collect(),
                )
            }
        }
        StructuralSequenceSpec::Single(index) => {
            Some(vec![normalize_exact_structural_component(index, len)?])
        }
        StructuralSequenceSpec::Range { start, end, step } => {
            if step == 0 {
                return None;
            }

            let start = normalize_exact_structural_component(start, len)?;
            let end = normalize_exact_structural_component(end, len)?;
            let mut current = start;
            let mut components = Vec::new();

            if step > 0 {
                while current <= end {
                    components.push(current);
                    current += step;
                }
            } else {
                while current >= end {
                    components.push(current);
                    current += step;
                }
            }

            Some(components)
        }
    }
}

fn normalize_exact_structural_component(position: isize, len: usize) -> Option<isize> {
    Some(normalize_structural_arg_position(position, len)? as isize + 1)
}

fn normalize_insert_arg_position(position: isize, len: usize) -> Option<usize> {
    let len = isize::try_from(len).ok()?;
    if position > 0 {
        if position <= len + 1 {
            return Some((position - 1) as usize);
        }
        return None;
    }

    if position < 0 {
        let normalized = len + position + 1;
        if (0..=len).contains(&normalized) {
            return Some(normalized as usize);
        }
    }

    None
}

fn parse_delete_positions(positions_expr: &Expr, len: usize) -> Option<Vec<usize>> {
    if let Some(position) = expr_as_signed_isize(positions_expr) {
        return Some(vec![normalize_structural_arg_position(position, len)?]);
    }

    let Expr::Call { head, args } = positions_expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) || args.is_empty() {
        return None;
    }

    args.iter()
        .map(|expr| normalize_structural_arg_position(expr_as_signed_isize(expr)?, len))
        .collect()
}

fn parse_take_list_counts(counts_expr: &Expr) -> Option<Vec<usize>> {
    let Expr::Call { head, args } = counts_expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    args.iter()
        .map(|arg| {
            let count = expr_as_signed_isize(arg)?;
            (count >= 0).then_some(count as usize)
        })
        .collect()
}

fn apply_take_list_counts(expr: &Expr, counts: &[usize]) -> Option<Vec<Expr>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    let mut offset = 0_usize;
    let mut chunks = Vec::with_capacity(counts.len());
    for count in counts {
        let end = offset.checked_add(*count)?;
        if end > args.len() {
            return None;
        }

        chunks.push(Expr::call(
            head.as_ref().clone(),
            args[offset..end].to_vec(),
        ));
        offset = end;
    }

    Some(chunks)
}

fn unary_operator_form(symbol: &str, item: Expr) -> Expr {
    let slot = Expr::call(system_expr("Slot"), vec![Expr::integer(BigInt::from(1))]);
    Expr::call(
        system_expr("Function"),
        vec![Expr::call(system_expr(symbol), vec![slot, item])],
    )
}

fn selection_name(symbol: &str) -> &'static str {
    match symbol {
        "First" => "first",
        "Last" => "last",
        _ => "requested",
    }
}

fn build_replace_plan_from_targets(targets: Vec<ReplacePartTarget>) -> ReplacePlanNode {
    let mut root = ReplacePlanNode::default();
    for target in targets {
        insert_replace_plan_path(
            &mut root,
            &target.path,
            ReplacePlanAction::Immediate(target.rhs),
        );
    }

    root
}

fn insert_replace_plan_path(node: &mut ReplacePlanNode, path: &[isize], action: ReplacePlanAction) {
    let Some((first, rest)) = path.split_first() else {
        node.action = Some(action);
        return;
    };

    insert_replace_plan_path(node.children.entry(*first).or_default(), rest, action);
}

fn path_is_prefix(prefix: &[isize], path: &[isize]) -> bool {
    prefix.len() <= path.len() && prefix.iter().zip(path).all(|(lhs, rhs)| lhs == rhs)
}

fn resolve_conflicting_replace_part_targets(
    targets: Vec<ReplacePartTarget>,
) -> Vec<ReplacePartTarget> {
    let mut kept = Vec::new();

    'next_target: for target in targets {
        if let Some(existing_index) = kept
            .iter()
            .position(|existing: &ReplacePartTarget| existing.path == target.path)
        {
            kept.remove(existing_index);
            kept.push(target);
            continue;
        }

        for existing in &kept {
            if path_is_prefix(&existing.path, &target.path) {
                continue 'next_target;
            }
        }

        kept.retain(|existing| !path_is_prefix(&target.path, &existing.path));
        kept.push(target);
    }

    kept
}

fn wrap_replace_part_pattern_position(pattern: Expr) -> Expr {
    if replace_part_pattern_is_explicit_list(&pattern) {
        pattern
    } else {
        Expr::call(system_expr("List"), vec![pattern])
    }
}

fn replace_part_pattern_is_explicit_list(expr: &Expr) -> bool {
    match expr {
        Expr::Call { head, .. } if is_system_symbol_name(head, heads::structure::LIST) => true,
        Expr::Call { head, args }
            if is_system_symbol_name(head, heads::patterns::PATTERN) && args.len() == 2 =>
        {
            replace_part_pattern_is_explicit_list(&args[1])
        }
        Expr::Call { head, args }
            if is_system_symbol_name(head, heads::patterns::PATTERN_TEST) && args.len() == 2 =>
        {
            replace_part_pattern_is_explicit_list(&args[0])
        }
        Expr::Call { head, args }
            if is_system_symbol_name(head, heads::patterns::CONDITION) && args.len() == 2 =>
        {
            replace_part_pattern_is_explicit_list(&args[0])
        }
        Expr::Call { head, args }
            if is_system_symbol_name(head, heads::patterns::OPTIONAL) && !args.is_empty() =>
        {
            replace_part_pattern_is_explicit_list(&args[0])
        }
        Expr::Call { head, args } if is_system_symbol_name(head, heads::patterns::ALTERNATIVES) => {
            !args.is_empty() && args.iter().all(replace_part_pattern_is_explicit_list)
        }
        _ => false,
    }
}

fn replace_part_pattern_includes_heads(pattern: &Expr, heads: ReplacePartHeadsMode) -> bool {
    match heads {
        ReplacePartHeadsMode::Include => true,
        ReplacePartHeadsMode::Exclude => false,
        ReplacePartHeadsMode::Automatic => replace_part_pattern_mentions_explicit_zero(pattern),
    }
}

fn replace_part_pattern_mentions_explicit_zero(expr: &Expr) -> bool {
    if expr_as_signed_isize(expr) == Some(0) {
        return true;
    }

    match expr {
        Expr::Atom(_) => false,
        Expr::Call { head, args } => {
            replace_part_pattern_mentions_explicit_zero(head)
                || args.iter().any(replace_part_pattern_mentions_explicit_zero)
        }
    }
}

fn collect_replace_part_candidate_paths(expr: &Expr, include_heads: bool) -> Vec<Vec<isize>> {
    let mut paths = Vec::new();
    collect_replace_part_candidate_paths_inner(expr, include_heads, Vec::new(), &mut paths);
    paths
}

fn collect_replace_part_candidate_paths_inner(
    expr: &Expr,
    include_heads: bool,
    path: Vec<isize>,
    paths: &mut Vec<Vec<isize>>,
) {
    if let Expr::Call { head, args } = expr {
        if include_heads {
            let mut head_path = path.clone();
            head_path.push(0);
            collect_replace_part_candidate_paths_inner(head, include_heads, head_path, paths);
        }

        for (index, arg) in args.iter().enumerate() {
            let mut child_path = path.clone();
            child_path.push(index as isize + 1);
            collect_replace_part_candidate_paths_inner(arg, include_heads, child_path, paths);
        }
    }

    paths.push(path);
}

fn replace_part_position_expr(path: &[isize]) -> Expr {
    Expr::call(
        system_expr("List"),
        path.iter()
            .map(|component| Expr::integer(BigInt::from(*component)))
            .collect(),
    )
}

fn replace_part_path_from_expr(expr: Expr) -> Vec<isize> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(&head, heads::structure::LIST) => args
            .into_iter()
            .map(|component| {
                expr_as_signed_isize(&component)
                    .expect("replace-part continuation paths should store integers")
            })
            .collect(),
        other => panic!("replace-part continuation paths should use List expressions: {other:?}"),
    }
}

fn collect_exact_paths_for_position_spec(
    expr: &Expr,
    selectors: &[PartSelector],
    heads: ReplacePartHeadsMode,
) -> Vec<Vec<isize>> {
    let Some((first, rest)) = selectors.split_first() else {
        return vec![Vec::new()];
    };

    let Some(selected) = select_positioned_parts_for_selector(expr, first, heads) else {
        return Vec::new();
    };

    let mut paths = Vec::new();
    for (prefix, value) in selected {
        for mut tail in collect_exact_paths_for_position_spec(&value, rest, heads) {
            let mut path = prefix.clone();
            path.append(&mut tail);
            paths.push(path);
        }
    }
    paths
}

fn select_positioned_parts_for_selector(
    expr: &Expr,
    selector: &PartSelector,
    heads: ReplacePartHeadsMode,
) -> Option<Vec<(Vec<isize>, Expr)>> {
    match selector {
        PartSelector::Single(spec) => Some(
            select_positioned_parts_for_atomic_spec_with_heads(expr, spec, heads)?
                .into_iter()
                .map(|(component, value)| (vec![component], value))
                .collect(),
        ),
        PartSelector::Key(key) => association_position_and_value_by_key(expr, key)
            .map(|(component, value)| vec![(vec![component, 2], value)]),
        PartSelector::Multi(specs) => {
            let mut selected = Vec::new();
            for spec in specs {
                selected.extend(select_positioned_parts_for_selector(expr, spec, heads)?);
            }
            Some(selected)
        }
    }
}

fn select_positioned_parts_for_atomic_spec_with_heads(
    expr: &Expr,
    spec: &AtomicPartSpec,
    heads: ReplacePartHeadsMode,
) -> Option<Vec<(isize, Expr)>> {
    match spec {
        AtomicPartSpec::Index(0) => match heads {
            ReplacePartHeadsMode::Exclude => None,
            ReplacePartHeadsMode::Automatic | ReplacePartHeadsMode::Include => {
                Some(vec![(0, structural_head_expr(expr))])
            }
        },
        AtomicPartSpec::Index(index) => {
            let value = select_exact_part(expr, *index)?;
            let component = exact_part_component(expr, *index)?;
            Some(vec![(component, value)])
        }
        AtomicPartSpec::All => match expr {
            Expr::Atom(_) => match heads {
                ReplacePartHeadsMode::Include => Some(vec![(0, structural_head_expr(expr))]),
                ReplacePartHeadsMode::Automatic | ReplacePartHeadsMode::Exclude => Some(Vec::new()),
            },
            Expr::Call { args, .. } => {
                let mut parts = Vec::new();
                if matches!(heads, ReplacePartHeadsMode::Include) {
                    parts.push((0, structural_head_expr(expr)));
                }
                parts.extend(
                    args.iter()
                        .enumerate()
                        .map(|(index, value)| (index as isize + 1, value.clone())),
                );
                Some(parts)
            }
        },
        AtomicPartSpec::Span(span) => match expr {
            _ => select_positioned_parts_for_span(
                expr,
                *span,
                replace_part_span_includes_heads(*span, heads),
            ),
        },
    }
}

fn replace_part_span_includes_heads(spec: PartSpanSpec, heads: ReplacePartHeadsMode) -> bool {
    match heads {
        ReplacePartHeadsMode::Include => true,
        ReplacePartHeadsMode::Exclude => false,
        ReplacePartHeadsMode::Automatic => {
            matches!(spec.start, SpanEndpoint::Index(0))
                || matches!(spec.end, SpanEndpoint::Index(0))
        }
    }
}

fn parse_replace_part_heads_option(expr: &Expr) -> Option<ReplacePartHeadsMode> {
    match parse_heads_option(expr)? {
        true => Some(ReplacePartHeadsMode::Include),
        false => Some(ReplacePartHeadsMode::Exclude),
    }
}

pub(super) fn replace_exact_path(expr: Expr, path: &[isize], rhs: Expr) -> Expr {
    let Some((first, rest)) = path.split_first() else {
        return rhs;
    };

    if *first == 0 {
        let replaced_head = replace_exact_path(structural_head_expr(&expr), rest, rhs);
        return match expr {
            Expr::Atom(_) => Expr::call(replaced_head, Vec::new()),
            Expr::Call { args, .. } => Expr::call(replaced_head, args),
        };
    }

    match expr {
        Expr::Atom(_) => expr,
        Expr::Call { head, mut args } => {
            let Some(index) = normalize_structural_arg_position(*first, args.len()) else {
                return Expr::Call { head, args };
            };

            let replaced_child = replace_exact_path(args[index].clone(), rest, rhs);
            args[index] = replaced_child;
            Expr::Call { head, args }
        }
    }
}
