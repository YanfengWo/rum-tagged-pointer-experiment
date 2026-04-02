use super::*;

pub(super) fn apply_heads_with_config(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    flavor: ApplyFlavor,
) -> (Expr, usize) {
    apply_heads_recursive(expr, operator, config, flavor, 0)
}

pub(super) fn apply_heads_recursive(
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

pub(super) fn apply_heads_on_standard_call(
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

pub(super) fn map_parts_with_config(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
) -> (Expr, usize) {
    map_parts_with_mode(expr, operator, config, MapTraversalFlavor::Map)
}

pub(super) fn map_parts_with_mode(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    flavor: MapTraversalFlavor,
) -> (Expr, usize) {
    map_parts_recursive(expr, operator, config, flavor, 0)
}

pub(super) fn map_parts_recursive(
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

pub(super) fn map_parts_on_standard_call(
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

pub(super) fn map_indexed_parts_with_config(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
) -> (Expr, usize) {
    map_indexed_parts_recursive(expr, operator, config, 0, &[])
}

pub(super) fn map_indexed_parts_recursive(
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

pub(super) fn map_indexed_parts_on_standard_call(
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

pub(super) fn collect_scan_parts_with_config(expr: &Expr, config: TraversalConfig) -> Vec<Expr> {
    collect_scan_parts_recursive(expr, config, 0).parts
}

pub(super) fn collect_scan_parts_recursive(
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

pub(super) fn collect_scan_parts_on_standard_call(
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
