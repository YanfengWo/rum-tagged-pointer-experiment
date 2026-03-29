use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TraversalConfig {
    levels: CasesLevelSpec,
    include_heads: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ApplyFlavor {
    Apply,
    MapApply,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MapLikeFlavor {
    Map,
    Scan,
    MapIndexed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MapTraversalFlavor {
    Map,
    MapAll,
}

enum ApplyNormalization {
    OperatorForm(Expr),
    Call {
        operator: Expr,
        expr: Expr,
        config: TraversalConfig,
    },
    Immediate(Expr),
}

enum MapLikeNormalization {
    OperatorForm(Expr),
    Call {
        operator: Expr,
        expr: Expr,
        config: TraversalConfig,
    },
    Immediate(Expr),
}

enum CommonArgMapNormalization {
    OperatorForm(Expr),
    Call {
        functions: Expr,
        common_expr: Expr,
        config: TraversalConfig,
    },
    Immediate(Expr),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScanCollection {
    parts: Vec<Expr>,
    depth: usize,
}

impl ApplyFlavor {
    fn symbol(self) -> &'static str {
        match self {
            Self::Apply => "Apply",
            Self::MapApply => "MapApply",
        }
    }

    fn default_levels(self) -> CasesLevelSpec {
        match self {
            Self::Apply => CasesLevelSpec::exact(0),
            Self::MapApply => CasesLevelSpec::exact(1),
        }
    }

    fn max_args(self) -> usize {
        match self {
            Self::Apply => 4,
            Self::MapApply => 3,
        }
    }
}

impl MapLikeFlavor {
    fn symbol(self) -> &'static str {
        match self {
            Self::Map => "Map",
            Self::Scan => "Scan",
            Self::MapIndexed => "MapIndexed",
        }
    }
}

impl Session {
    pub(in crate::runtime) fn eval_comap(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match self.normalize_comap_call(head, args) {
            CommonArgMapNormalization::OperatorForm(expr)
            | CommonArgMapNormalization::Immediate(expr) => Ok(expr),
            CommonArgMapNormalization::Call {
                functions,
                common_expr,
                config,
            } => {
                let operator =
                    function_body_operator(Expr::call(slot_expr(), vec![common_expr.clone()]));
                let (mapped, _) = map_parts_with_config(&functions, &operator, config);
                self.evaluate(mapped)
            }
        }
    }

    pub(in crate::runtime) fn eval_comap_apply(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match self.normalize_comap_apply_call(head, args) {
            CommonArgMapNormalization::OperatorForm(expr)
            | CommonArgMapNormalization::Immediate(expr) => Ok(expr),
            CommonArgMapNormalization::Call {
                functions,
                common_expr,
                config,
            } => {
                let operator = function_body_operator(Expr::call(
                    system_expr("Apply"),
                    vec![slot_expr(), common_expr.clone()],
                ));
                let (mapped, _) = map_parts_with_config(&functions, &operator, config);
                self.evaluate(mapped)
            }
        }
    }

    pub(in crate::runtime) fn eval_map_indexed(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match self.normalize_map_like_call(head, args, MapLikeFlavor::MapIndexed) {
            MapLikeNormalization::OperatorForm(expr) | MapLikeNormalization::Immediate(expr) => {
                Ok(expr)
            }
            MapLikeNormalization::Call {
                operator,
                expr,
                config,
            } => Ok(map_indexed_parts_with_config(&expr, &operator, config).0),
        }
    }

    pub(in crate::runtime) fn eval_map(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match self.normalize_map_like_call(head, args, MapLikeFlavor::Map) {
            MapLikeNormalization::OperatorForm(expr) | MapLikeNormalization::Immediate(expr) => {
                Ok(expr)
            }
            MapLikeNormalization::Call {
                operator,
                expr,
                config,
            } => Ok(map_parts_with_config(&expr, &operator, config).0),
        }
    }

    pub(in crate::runtime) fn eval_map_all(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match self.normalize_map_all_call(head, args) {
            MapLikeNormalization::OperatorForm(expr) | MapLikeNormalization::Immediate(expr) => {
                Ok(expr)
            }
            MapLikeNormalization::Call {
                operator,
                expr,
                config,
            } => Ok(map_parts_with_mode(&expr, &operator, config, MapTraversalFlavor::MapAll).0),
        }
    }

    pub(in crate::runtime) fn eval_scan(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match self.normalize_map_like_call(head, args, MapLikeFlavor::Scan) {
            MapLikeNormalization::OperatorForm(expr) | MapLikeNormalization::Immediate(expr) => {
                Ok(expr)
            }
            MapLikeNormalization::Call {
                operator,
                expr,
                config,
            } => {
                let parts = collect_scan_parts_with_config(&expr, config);
                self.evaluate_scan_parts(&operator, parts)
            }
        }
    }

    pub(in crate::runtime) fn eval_apply(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match self.normalize_apply_call(head, args, ApplyFlavor::Apply) {
            ApplyNormalization::OperatorForm(expr) | ApplyNormalization::Immediate(expr) => {
                Ok(expr)
            }
            ApplyNormalization::Call {
                operator,
                expr,
                config,
            } => Ok(apply_heads_with_config(&expr, &operator, config, ApplyFlavor::Apply).0),
        }
    }

    pub(in crate::runtime) fn eval_map_apply(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match self.normalize_apply_call(head, args, ApplyFlavor::MapApply) {
            ApplyNormalization::OperatorForm(expr) | ApplyNormalization::Immediate(expr) => {
                Ok(expr)
            }
            ApplyNormalization::Call {
                operator,
                expr,
                config,
            } => Ok(apply_heads_with_config(&expr, &operator, config, ApplyFlavor::MapApply).0),
        }
    }

    fn normalize_apply_call(
        &mut self,
        head: &Expr,
        args: &[Expr],
        flavor: ApplyFlavor,
    ) -> ApplyNormalization {
        let symbol = flavor.symbol();

        if !(1..=flavor.max_args()).contains(&args.len()) {
            return ApplyNormalization::Immediate(self.invalid_call_with_arg_range(
                symbol,
                args,
                1,
                flavor.max_args(),
            ));
        }

        if args.len() == 1 {
            return ApplyNormalization::OperatorForm(unary_operator_form(symbol, &args[0]));
        }

        if args.len() == 2 && parse_heads_option(&args[1]).is_some() {
            return ApplyNormalization::OperatorForm(unary_operator_form_with_tail(
                symbol,
                args[0].clone(),
                args[1].clone(),
            ));
        }

        let default_levels = flavor.default_levels();

        let (expr, levels, include_heads) = match args {
            [_, expr] => (expr.clone(), default_levels, false),
            [_, expr, third] => {
                if let Some(include_heads) = parse_heads_option(third) {
                    (expr.clone(), default_levels, include_heads)
                } else if flavor == ApplyFlavor::Apply {
                    let Some(levels) = parse_standard_level_spec(third) else {
                        return ApplyNormalization::Immediate(self.invalid_call_with_arg_form(
                            symbol,
                            args,
                            "expected a standard levelspec or Heads -> True/False as the third argument",
                        ));
                    };
                    (expr.clone(), levels, false)
                } else {
                    return ApplyNormalization::Immediate(self.invalid_call_with_arg_form(
                        symbol,
                        args,
                        "expected a Heads -> True/False option as the third argument",
                    ));
                }
            }
            [_, expr, third, fourth] if flavor == ApplyFlavor::Apply => {
                let Some(levels) = parse_standard_level_spec(third) else {
                    return ApplyNormalization::Immediate(self.invalid_call_with_arg_form(
                        symbol,
                        args,
                        "expected a standard levelspec as the third argument",
                    ));
                };
                let Some(include_heads) = parse_heads_option(fourth) else {
                    return ApplyNormalization::Immediate(self.invalid_call_with_arg_form(
                        symbol,
                        args,
                        "expected a Heads -> True/False option as the fourth argument",
                    ));
                };
                (expr.clone(), levels, include_heads)
            }
            _ => return ApplyNormalization::Immediate(Expr::call(head.clone(), args.to_vec())),
        };

        ApplyNormalization::Call {
            operator: args[0].clone(),
            expr,
            config: TraversalConfig {
                levels,
                include_heads,
            },
        }
    }

    fn normalize_map_like_call(
        &mut self,
        head: &Expr,
        args: &[Expr],
        flavor: MapLikeFlavor,
    ) -> MapLikeNormalization {
        let symbol = flavor.symbol();

        if !(1..=4).contains(&args.len()) {
            return MapLikeNormalization::Immediate(
                self.invalid_call_with_arg_range(symbol, args, 1, 4),
            );
        }

        if args.len() == 1 {
            return MapLikeNormalization::OperatorForm(unary_operator_form(symbol, &args[0]));
        }

        if args.len() == 2 && parse_heads_option(&args[1]).is_some() {
            return MapLikeNormalization::OperatorForm(unary_operator_form_with_tail(
                symbol,
                args[0].clone(),
                args[1].clone(),
            ));
        }

        let (expr, levels, include_heads) = match args {
            [_, expr] => (expr.clone(), CasesLevelSpec::exact(1), false),
            [_, expr, third] => {
                if let Some(include_heads) = parse_heads_option(third) {
                    (expr.clone(), CasesLevelSpec::exact(1), include_heads)
                } else {
                    let Some(levels) = parse_standard_level_spec(third) else {
                        return MapLikeNormalization::Immediate(self.invalid_call_with_arg_form(
                            symbol,
                            args,
                            "expected a standard levelspec or Heads -> True/False as the third argument",
                        ));
                    };
                    (expr.clone(), levels, false)
                }
            }
            [_, expr, third, fourth] => {
                let Some(levels) = parse_standard_level_spec(third) else {
                    return MapLikeNormalization::Immediate(self.invalid_call_with_arg_form(
                        symbol,
                        args,
                        "expected a standard levelspec as the third argument",
                    ));
                };
                let Some(include_heads) = parse_heads_option(fourth) else {
                    return MapLikeNormalization::Immediate(self.invalid_call_with_arg_form(
                        symbol,
                        args,
                        "expected a Heads -> True/False option as the fourth argument",
                    ));
                };
                (expr.clone(), levels, include_heads)
            }
            _ => return MapLikeNormalization::Immediate(Expr::call(head.clone(), args.to_vec())),
        };

        MapLikeNormalization::Call {
            operator: args[0].clone(),
            expr,
            config: TraversalConfig {
                levels,
                include_heads,
            },
        }
    }

    fn normalize_map_all_call(&mut self, head: &Expr, args: &[Expr]) -> MapLikeNormalization {
        if !(1..=3).contains(&args.len()) {
            return MapLikeNormalization::Immediate(
                self.invalid_call_with_arg_range("MapAll", args, 1, 3),
            );
        }

        if args.len() == 1 {
            return MapLikeNormalization::OperatorForm(unary_operator_form("MapAll", &args[0]));
        }

        if args.len() == 2 && parse_heads_option(&args[1]).is_some() {
            return MapLikeNormalization::OperatorForm(unary_operator_form_with_tail(
                "MapAll",
                args[0].clone(),
                args[1].clone(),
            ));
        }

        let include_heads = match args {
            [_, _] => false,
            [_, _, option] => {
                let Some(include_heads) = parse_heads_option(option) else {
                    return MapLikeNormalization::Immediate(self.invalid_call_with_arg_form(
                        "MapAll",
                        args,
                        "expected a Heads -> True/False option as the third argument",
                    ));
                };
                include_heads
            }
            _ => return MapLikeNormalization::Immediate(Expr::call(head.clone(), args.to_vec())),
        };

        MapLikeNormalization::Call {
            operator: args[0].clone(),
            expr: args[1].clone(),
            config: TraversalConfig {
                levels: CasesLevelSpec::range(0, None),
                include_heads,
            },
        }
    }

    fn normalize_comap_call(&mut self, head: &Expr, args: &[Expr]) -> CommonArgMapNormalization {
        if !(1..=4).contains(&args.len()) {
            return CommonArgMapNormalization::Immediate(
                self.invalid_call_with_arg_range("Comap", args, 1, 4),
            );
        }

        if args.len() == 1 {
            return CommonArgMapNormalization::OperatorForm(unary_operator_form("Comap", &args[0]));
        }

        if args.len() == 2 && parse_heads_option(&args[1]).is_some() {
            return CommonArgMapNormalization::OperatorForm(unary_operator_form_with_tail(
                "Comap",
                args[0].clone(),
                args[1].clone(),
            ));
        }

        let (functions, common_expr, levels, include_heads) = match args {
            [functions, common_expr] => (
                functions.clone(),
                common_expr.clone(),
                CasesLevelSpec::exact(1),
                false,
            ),
            [functions, common_expr, third] => {
                if let Some(include_heads) = parse_heads_option(third) {
                    (
                        functions.clone(),
                        common_expr.clone(),
                        CasesLevelSpec::exact(1),
                        include_heads,
                    )
                } else {
                    let Some(levels) = parse_standard_level_spec(third) else {
                        return CommonArgMapNormalization::Immediate(
                            self.invalid_call_with_arg_form(
                                "Comap",
                                args,
                                "expected a standard levelspec or Heads -> True/False as the third argument",
                            ),
                        );
                    };
                    (functions.clone(), common_expr.clone(), levels, false)
                }
            }
            [functions, common_expr, levels_expr, option] => {
                let Some(levels) = parse_standard_level_spec(levels_expr) else {
                    return CommonArgMapNormalization::Immediate(self.invalid_call_with_arg_form(
                        "Comap",
                        args,
                        "expected a standard levelspec as the third argument",
                    ));
                };
                let Some(include_heads) = parse_heads_option(option) else {
                    return CommonArgMapNormalization::Immediate(self.invalid_call_with_arg_form(
                        "Comap",
                        args,
                        "expected a Heads -> True/False option as the fourth argument",
                    ));
                };
                (
                    functions.clone(),
                    common_expr.clone(),
                    levels,
                    include_heads,
                )
            }
            _ => {
                return CommonArgMapNormalization::Immediate(Expr::call(
                    head.clone(),
                    args.to_vec(),
                ));
            }
        };

        CommonArgMapNormalization::Call {
            functions,
            common_expr,
            config: TraversalConfig {
                levels,
                include_heads,
            },
        }
    }

    fn normalize_comap_apply_call(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> CommonArgMapNormalization {
        if !(1..=3).contains(&args.len()) {
            return CommonArgMapNormalization::Immediate(self.invalid_call_with_arg_range(
                "ComapApply",
                args,
                1,
                3,
            ));
        }

        if args.len() == 1 {
            return CommonArgMapNormalization::OperatorForm(unary_operator_form(
                "ComapApply",
                &args[0],
            ));
        }

        if args.len() == 2 && parse_heads_option(&args[1]).is_some() {
            return CommonArgMapNormalization::OperatorForm(unary_operator_form_with_tail(
                "ComapApply",
                args[0].clone(),
                args[1].clone(),
            ));
        }

        let include_heads = match args {
            [_, _] => false,
            [_, _, option] => {
                let Some(include_heads) = parse_heads_option(option) else {
                    return CommonArgMapNormalization::Immediate(self.invalid_call_with_arg_form(
                        "ComapApply",
                        args,
                        "expected a Heads -> True/False option as the third argument",
                    ));
                };
                include_heads
            }
            _ => {
                return CommonArgMapNormalization::Immediate(Expr::call(
                    head.clone(),
                    args.to_vec(),
                ));
            }
        };

        CommonArgMapNormalization::Call {
            functions: args[0].clone(),
            common_expr: args[1].clone(),
            config: TraversalConfig {
                levels: CasesLevelSpec::exact(1),
                include_heads,
            },
        }
    }

    fn evaluate_scan_parts(
        &mut self,
        operator: &Expr,
        parts: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        for part in parts {
            let result = self.evaluate(Expr::call(operator.clone(), vec![part]));
            self.suspend_with_frame(result, |_, value| Ok(value))?;
        }

        Ok(system_expr("Null"))
    }
}

fn apply_heads_with_config(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    flavor: ApplyFlavor,
) -> (Expr, usize) {
    apply_heads_recursive(expr, operator, config, flavor, 0)
}

fn apply_heads_recursive(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    flavor: ApplyFlavor,
    level: usize,
) -> (Expr, usize) {
    match expr {
        Expr::Atom(_) => (expr.clone(), 1),
        Expr::Call { head, args } if is_system_symbol_name(head, "Association") => {
            if let Some(entries) = association_entries_from_expr(expr) {
                let mut max_depth = 0usize;
                let mut transformed_entries = Vec::with_capacity(entries.len());

                for (key, value) in entries {
                    let (transformed_value, value_depth) =
                        apply_heads_recursive(&value, operator, config, flavor, level + 1);
                    max_depth = max_depth.max(value_depth);
                    transformed_entries.push((key, transformed_value));
                }

                let depth = 1 + max_depth;
                let rendered =
                    if config.levels.matches(level, depth) && flavor == ApplyFlavor::Apply {
                        Expr::call(
                            operator.clone(),
                            transformed_entries
                                .iter()
                                .map(|(_, value)| value.clone())
                                .collect::<Vec<_>>(),
                        )
                    } else {
                        render_association(transformed_entries)
                    };

                (rendered, depth)
            } else {
                apply_heads_on_standard_call(expr, operator, config, flavor, level)
            }
        }
        Expr::Call { .. } => apply_heads_on_standard_call(expr, operator, config, flavor, level),
    }
}

fn apply_heads_on_standard_call(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    flavor: ApplyFlavor,
    level: usize,
) -> (Expr, usize) {
    let Expr::Call { head, args } = expr else {
        return (expr.clone(), 1);
    };

    let (transformed_head, head_depth) = if config.include_heads {
        apply_heads_recursive(head, operator, config, flavor, level + 1)
    } else {
        (head.as_ref().clone(), 0)
    };

    let mut transformed_args = Vec::with_capacity(args.len());
    let mut max_arg_depth = 0usize;
    for arg in args {
        let (transformed_arg, arg_depth) =
            apply_heads_recursive(arg, operator, config, flavor, level + 1);
        max_arg_depth = max_arg_depth.max(arg_depth);
        transformed_args.push(transformed_arg);
    }

    let depth = 1 + head_depth.max(max_arg_depth);
    let rebuilt = Expr::call(transformed_head, transformed_args.clone());

    if config.levels.matches(level, depth) {
        (Expr::call(operator.clone(), transformed_args), depth)
    } else {
        (rebuilt, depth)
    }
}

fn map_parts_with_config(expr: &Expr, operator: &Expr, config: TraversalConfig) -> (Expr, usize) {
    map_parts_with_mode(expr, operator, config, MapTraversalFlavor::Map)
}

fn map_parts_with_mode(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    flavor: MapTraversalFlavor,
) -> (Expr, usize) {
    map_parts_recursive(expr, operator, config, flavor, 0)
}

fn map_parts_recursive(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    flavor: MapTraversalFlavor,
    level: usize,
) -> (Expr, usize) {
    match expr {
        Expr::Atom(_) => {
            let depth = 1;
            let rendered = if config.levels.matches(level, depth) {
                Expr::call(operator.clone(), vec![expr.clone()])
            } else {
                expr.clone()
            };

            (rendered, depth)
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Association") => {
            if let Some(entries) = association_entries_from_expr(expr) {
                let mut max_depth = 0usize;
                let mut transformed_entries = Vec::with_capacity(entries.len());

                for (key, value) in entries {
                    let (transformed_value, value_depth) =
                        map_parts_recursive(&value, operator, config, flavor, level + 1);
                    max_depth = max_depth.max(value_depth);
                    transformed_entries.push((key, transformed_value));
                }

                let depth = 1 + max_depth;
                let rebuilt = render_association(transformed_entries);
                let rendered = if config.levels.matches(level, depth)
                    && !(flavor == MapTraversalFlavor::MapAll
                        && association_entries_from_expr(expr).is_some())
                {
                    Expr::call(operator.clone(), vec![rebuilt])
                } else {
                    rebuilt
                };

                (rendered, depth)
            } else {
                map_parts_on_standard_call(expr, operator, config, flavor, level)
            }
        }
        Expr::Call { .. } => map_parts_on_standard_call(expr, operator, config, flavor, level),
    }
}

fn map_parts_on_standard_call(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    flavor: MapTraversalFlavor,
    level: usize,
) -> (Expr, usize) {
    let Expr::Call { head, args } = expr else {
        return (expr.clone(), 1);
    };

    let (transformed_head, head_depth) = if config.include_heads {
        map_parts_recursive(head, operator, config, flavor, level + 1)
    } else {
        (head.as_ref().clone(), 0)
    };

    let mut transformed_args = Vec::with_capacity(args.len());
    let mut max_arg_depth = 0usize;
    for arg in args {
        let (transformed_arg, arg_depth) =
            map_parts_recursive(arg, operator, config, flavor, level + 1);
        max_arg_depth = max_arg_depth.max(arg_depth);
        transformed_args.push(transformed_arg);
    }

    let depth = 1 + head_depth.max(max_arg_depth);
    let rebuilt = Expr::call(transformed_head, transformed_args);
    let rendered = if config.levels.matches(level, depth) {
        Expr::call(operator.clone(), vec![rebuilt])
    } else {
        rebuilt
    };

    (rendered, depth)
}

fn map_indexed_parts_with_config(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
) -> (Expr, usize) {
    map_indexed_parts_recursive(expr, operator, config, 0, &[])
}

fn map_indexed_parts_recursive(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    level: usize,
    path: &[isize],
) -> (Expr, usize) {
    match expr {
        Expr::Atom(_) => {
            let depth = 1;
            let rendered = if config.levels.matches(level, depth) {
                Expr::call(
                    operator.clone(),
                    vec![expr.clone(), position_expr_from_path(path)],
                )
            } else {
                expr.clone()
            };
            (rendered, depth)
        }
        Expr::Call { head, .. } if is_system_symbol_name(head, "Association") => {
            if let Some(entries) = association_entries_from_expr(expr) {
                let mut max_depth = 0usize;
                let mut rebuilt_entries = Vec::with_capacity(entries.len());

                for (index, (key, value)) in entries.into_iter().enumerate() {
                    let child_path = extend_isize_path(path, index as isize + 1);
                    let (mapped_value, child_depth) = map_indexed_parts_recursive(
                        &value,
                        operator,
                        config,
                        level + 1,
                        &child_path,
                    );
                    max_depth = max_depth.max(child_depth);
                    rebuilt_entries.push((key, mapped_value));
                }

                let depth = 1 + max_depth;
                let rebuilt = render_association(rebuilt_entries);
                let rendered = if config.levels.matches(level, depth) {
                    Expr::call(
                        operator.clone(),
                        vec![rebuilt, position_expr_from_path(path)],
                    )
                } else {
                    rebuilt
                };

                (rendered, depth)
            } else {
                map_indexed_parts_on_standard_call(expr, operator, config, level, path)
            }
        }
        Expr::Call { .. } => {
            map_indexed_parts_on_standard_call(expr, operator, config, level, path)
        }
    }
}

fn map_indexed_parts_on_standard_call(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    level: usize,
    path: &[isize],
) -> (Expr, usize) {
    let Expr::Call { head, args } = expr else {
        return (expr.clone(), 1);
    };

    let (transformed_head, head_depth) = if config.include_heads {
        let child_path = extend_isize_path(path, 0);
        map_indexed_parts_recursive(head, operator, config, level + 1, &child_path)
    } else {
        (head.as_ref().clone(), 0)
    };

    let mut transformed_args = Vec::with_capacity(args.len());
    let mut max_arg_depth = 0usize;
    for (index, arg) in args.iter().enumerate() {
        let child_path = extend_isize_path(path, index as isize + 1);
        let (transformed_arg, arg_depth) =
            map_indexed_parts_recursive(arg, operator, config, level + 1, &child_path);
        max_arg_depth = max_arg_depth.max(arg_depth);
        transformed_args.push(transformed_arg);
    }

    let depth = 1 + head_depth.max(max_arg_depth);
    let rebuilt = Expr::call(transformed_head, transformed_args);
    let rendered = if config.levels.matches(level, depth) {
        Expr::call(
            operator.clone(),
            vec![rebuilt, position_expr_from_path(path)],
        )
    } else {
        rebuilt
    };

    (rendered, depth)
}

fn collect_scan_parts_with_config(expr: &Expr, config: TraversalConfig) -> Vec<Expr> {
    collect_scan_parts_recursive(expr, config, 0).parts
}

fn collect_scan_parts_recursive(
    expr: &Expr,
    config: TraversalConfig,
    level: usize,
) -> ScanCollection {
    match expr {
        Expr::Atom(_) => {
            let mut parts = Vec::new();
            if config.levels.matches(level, 1) {
                parts.push(expr.clone());
            }

            ScanCollection { parts, depth: 1 }
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Association") => {
            if let Some(entries) = association_entries_from_expr(expr) {
                let mut parts = Vec::new();
                let mut max_depth = 0usize;

                for (_, value) in entries {
                    let child = collect_scan_parts_recursive(&value, config, level + 1);
                    max_depth = max_depth.max(child.depth);
                    parts.extend(child.parts);
                }

                let depth = 1 + max_depth;
                if config.levels.matches(level, depth) {
                    parts.push(expr.clone());
                }

                ScanCollection { parts, depth }
            } else {
                collect_scan_parts_on_standard_call(expr, config, level)
            }
        }
        Expr::Call { .. } => collect_scan_parts_on_standard_call(expr, config, level),
    }
}

fn collect_scan_parts_on_standard_call(
    expr: &Expr,
    config: TraversalConfig,
    level: usize,
) -> ScanCollection {
    let Expr::Call { head, args } = expr else {
        return ScanCollection {
            parts: vec![expr.clone()],
            depth: 1,
        };
    };

    let mut parts = Vec::new();
    let mut max_child_depth = 0usize;

    if config.include_heads {
        let head_result = collect_scan_parts_recursive(head, config, level + 1);
        max_child_depth = max_child_depth.max(head_result.depth);
        parts.extend(head_result.parts);
    }

    for arg in args {
        let child = collect_scan_parts_recursive(arg, config, level + 1);
        max_child_depth = max_child_depth.max(child.depth);
        parts.extend(child.parts);
    }

    let depth = 1 + max_child_depth;
    if config.levels.matches(level, depth) {
        parts.push(expr.clone());
    }

    ScanCollection { parts, depth }
}
