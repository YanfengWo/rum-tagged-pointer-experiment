use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct InsertPathFailure {
    position: String,
}

impl Session {
    pub(in crate::runtime) fn eval_insert(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr, item, position] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Insert", args, 3));
        };

        let Some(paths) = parse_insert_position_paths(position) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        if matches!(expr, Expr::Atom(_)) {
            return Ok(self.invalid_insert_path_call(
                args,
                InsertPathFailure {
                    position: insert_position_message_part(
                        paths.first().map_or(&[][..], Vec::as_slice),
                    ),
                },
            ));
        }

        let Expr::Call { .. } = expr else {
            return Ok(self.invalid_normal_expression_call("Insert", args));
        };

        let mut canonical_paths = Vec::new();
        for path in paths {
            match canonical_insert_path(expr, &path) {
                Ok(path) => canonical_paths.push(path),
                Err(failure) => return Ok(self.invalid_insert_path_call(args, failure)),
            }
        }

        Ok(insert_canonical_paths(
            expr.clone(),
            item.clone(),
            canonical_paths,
        ))
    }

    fn invalid_insert_path_call(&mut self, args: &[Expr], failure: InsertPathFailure) -> Expr {
        self.push_message(
            "Insert",
            "ins",
            format!(
                "Cannot insert at position {} in {}",
                failure.position, args[0]
            ),
        );
        Expr::call(system_expr("Insert"), args.to_vec())
    }
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

fn parse_insert_position_paths(positions_expr: &Expr) -> Option<Vec<Vec<isize>>> {
    if let Some(position) = expr_as_signed_isize(positions_expr) {
        return Some(vec![vec![position]]);
    }

    let Expr::Call { head, args } = positions_expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) || args.is_empty() {
        return None;
    }

    if args.iter().all(|expr| expr_as_signed_isize(expr).is_some()) {
        return Some(vec![
            args.iter()
                .map(expr_as_signed_isize)
                .collect::<Option<Vec<_>>>()?,
        ]);
    }

    args.iter().map(parse_insert_position_path_list).collect()
}

fn parse_insert_position_path_list(expr: &Expr) -> Option<Vec<isize>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) || args.is_empty() {
        return None;
    }

    args.iter().map(expr_as_signed_isize).collect()
}

fn canonical_insert_path(expr: &Expr, path: &[isize]) -> Result<Vec<usize>, InsertPathFailure> {
    let mut target = expr.clone();
    let mut canonical = Vec::with_capacity(path.len());

    for (depth, position) in path.iter().copied().enumerate() {
        let is_last = depth + 1 == path.len();
        let Expr::Call { args, .. } = &target else {
            return Err(InsertPathFailure {
                position: insert_position_message_part(path),
            });
        };

        if is_last {
            let Some(index) = normalize_insert_arg_position(position, args.len()) else {
                return Err(InsertPathFailure {
                    position: insert_position_message_part(path),
                });
            };
            canonical.push(index);
        } else {
            let Some(index) = normalize_structural_arg_position(position, args.len()) else {
                return Err(InsertPathFailure {
                    position: insert_position_message_part(path),
                });
            };
            canonical.push(index);
            target = args[index].clone();
        }
    }

    Ok(canonical)
}

fn insert_position_message_part(path: &[isize]) -> String {
    format!(
        "{{{}}}",
        path.iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn insert_canonical_paths(expr: Expr, item: Expr, mut paths: Vec<Vec<usize>>) -> Expr {
    paths.sort_by(|left, right| right.cmp(left));
    paths.dedup();
    paths
        .into_iter()
        .fold(expr, |expr, path| insert_canonical_path(expr, &path, &item))
}

fn insert_canonical_path(expr: Expr, path: &[usize], item: &Expr) -> Expr {
    let Some((first, rest)) = path.split_first() else {
        return expr;
    };

    match expr {
        Expr::Atom(_) => expr,
        Expr::Call { head, args } => {
            if rest.is_empty() {
                if *first <= args.len() {
                    let mut args = Expr::unwrap_shared_args(args);
                    args.insert(*first, item.clone());
                    return Expr::Call {
                        head,
                        args: SharedArgs::new(args),
                    };
                }
            } else if *first < args.len() {
                let mut args = Expr::unwrap_shared_args(args);
                args[*first] = insert_canonical_path(args[*first].clone(), rest, item);
                return Expr::Call {
                    head,
                    args: SharedArgs::new(args),
                };
            }
            Expr::Call { head, args }
        }
    }
}
