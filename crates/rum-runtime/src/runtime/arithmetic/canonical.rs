use super::super::*;
use super::exact::*;
use std::borrow::Cow;

impl Session {
    pub(super) fn eval_plus(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if let Some(value) = fast_integer_plus(args) {
            return Ok(value);
        }

        self.combine_additive_args(head, args)
    }

    pub(super) fn eval_times(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        self.combine_multiplicative_args(head, args)
    }

    pub(super) fn eval_power(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Power", args, 2));
        }

        if let Some(base) = expr_as_exact_rational(&args[0]) {
            let Some(exponent) = expr_as_signed_integer_value(&args[1]) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };

            if exponent == BigInt::from(0_u8) && base.is_zero() {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            }

            return Ok(base
                .pow(&exponent)
                .map(Expr::rational)
                .unwrap_or_else(|| Expr::call(head.clone(), args.to_vec())));
        }

        if let Some((base, inner_exponent)) = split_power_base_exp(&args[0]) {
            if let Some(outer_exponent) = expr_as_signed_integer_value(&args[1]) {
                let exponent = inner_exponent.mul(&Rational::from_integer(outer_exponent));
                return Ok(build_power_from_base_exp(base, exponent));
            }
        }

        if let Expr::Call {
            head: times_head,
            args: factors,
        } = &args[0]
        {
            if is_system_symbol_name(times_head, "Times")
                && expr_as_exact_rational(&args[1]).is_some()
            {
                let mut numeric = Rational::one();
                let mut symbolic = Vec::new();
                for factor in factors.iter() {
                    if let Some(value) = expr_as_exact_rational(factor) {
                        numeric = numeric.mul(&value);
                    } else {
                        symbolic.push(factor.clone());
                    }
                }

                let power_head = system_expr("Power");
                let times_head = system_expr("Times");
                let numeric_part = if numeric.is_one() {
                    Expr::integer(1_i32.into())
                } else {
                    self.eval_power(&power_head, &[Expr::rational(numeric), args[1].clone()])?
                };
                if expr_as_signed_integer_value(&args[1]).is_some() {
                    let mut parts = Vec::with_capacity(symbolic.len() + 1);
                    parts.push(numeric_part);
                    for factor in symbolic {
                        parts.push(self.eval_power(&power_head, &[factor, args[1].clone()])?);
                    }
                    return self.combine_multiplicative_args(&times_head, &parts);
                }

                let symbolic_part = match symbolic.as_slice() {
                    [] => Expr::integer(1_i32.into()),
                    [factor] => self.eval_power(&power_head, &[factor.clone(), args[1].clone()])?,
                    _ => Expr::call(
                        power_head,
                        vec![Expr::call(times_head.clone(), symbolic), args[1].clone()],
                    ),
                };
                return self
                    .combine_multiplicative_args(&times_head, &[numeric_part, symbolic_part]);
            }
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(super) fn eval_subtract(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Subtract", args, 2));
        }

        let negated_rhs = self.combine_multiplicative_args(
            &system_expr("Times"),
            &[Expr::integer((-1_i32).into()), args[1].clone()],
        )?;
        self.combine_additive_args(&system_expr("Plus"), &[args[0].clone(), negated_rhs])
    }

    pub(super) fn eval_divide(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Divide", args, 2));
        }

        if let (Some(lhs), Some(rhs)) = (
            expr_as_exact_rational(&args[0]),
            expr_as_exact_rational(&args[1]),
        ) {
            return Ok(lhs
                .div(&rhs)
                .map(Expr::rational)
                .unwrap_or_else(|| Expr::call(head.clone(), args.to_vec())));
        }

        let reciprocal = self.eval_power(
            &system_expr("Power"),
            &[args[1].clone(), Expr::integer((-1_i32).into())],
        )?;
        self.combine_multiplicative_args(&system_expr("Times"), &[args[0].clone(), reciprocal])
    }

    /// Canonically combines arguments using the kernel's Plus semantics.
    pub fn combine_additive_args(&self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let mut numeric = Rational::zero();
        let mut combined = Vec::new();

        for arg in args {
            if let Some(value) = expr_as_exact_rational(arg) {
                numeric = numeric.add(&value);
            } else {
                combined.push(arg.clone());
            }
        }

        if let Some(rebuilt) = combine_simple_additive_terms(&combined) {
            if let Some(rebuilt) = rebuilt {
                combined = rebuilt;
            }
        } else if !combined.is_empty() {
            let mut grouped: Vec<(Expr, Rational)> = Vec::with_capacity(combined.len());
            let mut rebuild_combined = false;
            for term in &combined {
                let (coefficient, kernel) = split_coeff_kernel(term);
                if let Some((_, existing)) = grouped
                    .iter_mut()
                    .find(|(candidate, _)| candidate == kernel.as_ref())
                {
                    *existing = existing.add(&coefficient);
                    rebuild_combined = true;
                } else {
                    if split_coeff_kernel_requires_rebuild(term, &coefficient, kernel.as_ref()) {
                        rebuild_combined = true;
                    }
                    grouped.push((kernel.into_owned(), coefficient));
                }
            }

            if rebuild_combined {
                combined = Vec::with_capacity(grouped.len() + usize::from(!numeric.is_zero()));
                for (kernel, coefficient) in grouped {
                    if coefficient.is_zero() {
                        continue;
                    }

                    if coefficient.is_one() {
                        combined.push(kernel);
                    } else {
                        let coefficient = Expr::rational(coefficient);
                        combined.push(match kernel {
                            Expr::Call { head, args } if is_system_symbol_name(&head, "Times") => {
                                let mut factors = Vec::with_capacity(args.len() + 1);
                                factors.push(coefficient);
                                factors.extend(args);
                                Expr::call(system_expr("Times"), factors)
                            }
                            kernel => Expr::call(system_expr("Times"), vec![coefficient, kernel]),
                        });
                    }
                }
            }
        }

        if !numeric.is_zero() {
            combined.push(Expr::rational(numeric));
        }

        let normalized = self.normalize_call_args_by_attributes(
            head,
            combined,
            self.attribute_set_for_head(head),
        );

        Ok(match normalized.as_slice() {
            [] => Expr::integer(0_i32.into()),
            [only] => only.clone(),
            _ => Expr::call(head.clone(), normalized),
        })
    }

    /// Canonically combines arguments using the kernel's Times semantics.
    pub fn combine_multiplicative_args(
        &self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let mut numeric = Rational::one();
        let mut combined = Vec::new();

        for arg in args {
            if let Some(value) = expr_as_exact_rational(arg) {
                numeric = numeric.mul(&value);

                if numeric.is_zero() {
                    return Ok(Expr::integer(0_i32.into()));
                }
            } else {
                combined.push(arg.clone());
            }
        }

        if let Some(rebuilt) = combine_simple_multiplicative_factors(&combined) {
            if let Some(rebuilt) = rebuilt {
                combined = rebuilt;
            }
        } else if !combined.is_empty() {
            let mut grouped: Vec<(Expr, Rational)> = Vec::with_capacity(combined.len());
            let mut rebuild_combined = false;
            for factor in &combined {
                let (base, exponent) = split_base_exp(factor);
                if let Some((_, existing)) = grouped
                    .iter_mut()
                    .find(|(candidate, _)| candidate == base.as_ref())
                {
                    *existing = existing.add(&exponent);
                    rebuild_combined = true;
                } else {
                    if split_base_exp_requires_rebuild(factor, base.as_ref(), &exponent) {
                        rebuild_combined = true;
                    }
                    grouped.push((base.into_owned(), exponent));
                }
            }

            if rebuild_combined {
                combined = Vec::with_capacity(grouped.len() + usize::from(!numeric.is_one()));
                for (base, exponent) in grouped {
                    if exponent.is_zero() {
                        continue;
                    }

                    combined.push(build_power_from_base_exp(base, exponent));
                }
            }
        }

        if !numeric.is_one() || combined.is_empty() {
            combined.push(Expr::rational(numeric));
        }

        let normalized = self.normalize_call_args_by_attributes(
            head,
            combined,
            self.attribute_set_for_head(head),
        );

        Ok(match normalized.as_slice() {
            [] => Expr::integer(1_i32.into()),
            [only] => only.clone(),
            _ => Expr::call(head.clone(), normalized),
        })
    }
}

fn fast_integer_plus(args: &[Expr]) -> Option<Expr> {
    let mut iter = args.iter();
    let Some(first) = iter.next() else {
        return Some(Expr::integer(0_i32.into()));
    };

    let mut total = expr_as_exact_integer(first)?;
    for arg in iter {
        total += expr_as_exact_integer(arg)?;
    }

    Some(Expr::integer(total))
}
fn build_power_from_base_exp(base: Expr, exponent: Rational) -> Expr {
    if exponent.is_zero() {
        return Expr::integer(1_i32.into());
    }

    if exponent.is_one() {
        base
    } else {
        Expr::call(system_expr("Power"), vec![base, Expr::rational(exponent)])
    }
}

fn split_base_exp(expr: &Expr) -> (Cow<'_, Expr>, Rational) {
    let Some((base, exponent)) = split_power_base_exp_ref(expr) else {
        return (Cow::Borrowed(expr), Rational::one());
    };

    if !exponent.is_integer() {
        return (Cow::Borrowed(base), exponent);
    }

    let (inner_base, inner_exponent) = split_base_exp(base);
    (inner_base, inner_exponent.mul(&exponent))
}

fn split_base_exp_requires_rebuild(factor: &Expr, base: &Expr, exponent: &Rational) -> bool {
    if exponent.is_zero() {
        return true;
    }

    if exponent.is_one() {
        return base != factor;
    }

    let Expr::Call { head, args } = factor else {
        return true;
    };
    if !is_system_symbol_name(head, "Power") || args.len() != 2 {
        return true;
    }

    let Some(original_exponent) = expr_as_exact_rational(&args[1]) else {
        return true;
    };

    &args[0] != base || &original_exponent != exponent
}

fn combine_simple_multiplicative_factors(args: &[Expr]) -> Option<Option<Vec<Expr>>> {
    if args.is_empty() {
        return Some(None);
    }

    let mut grouped: Vec<(Expr, Rational)> = Vec::with_capacity(args.len());
    let mut rebuild_combined = false;
    for factor in args {
        let (base, exponent) = simple_base_exp(factor)?;
        if let Some((_, existing)) = grouped.iter_mut().find(|(candidate, _)| candidate == base) {
            *existing = existing.add(&exponent);
            rebuild_combined = true;
        } else {
            if split_base_exp_requires_rebuild(factor, base, &exponent) {
                rebuild_combined = true;
            }
            grouped.push((base.clone(), exponent));
        }
    }

    if !rebuild_combined {
        return Some(None);
    }

    let mut combined = Vec::with_capacity(grouped.len());
    for (base, exponent) in grouped {
        if exponent.is_zero() {
            continue;
        }

        combined.push(build_power_from_base_exp(base, exponent));
    }

    Some(Some(combined))
}

fn simple_base_exp(expr: &Expr) -> Option<(&Expr, Rational)> {
    if matches!(expr, Expr::Atom(_)) {
        return Some((expr, Rational::one()));
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, "Power") || args.len() != 2 {
        return None;
    }

    let Expr::Atom(_) = &args[0] else {
        return None;
    };
    if expr_as_exact_rational(&args[0]).is_some() {
        return None;
    }
    let exponent = expr_as_exact_rational(&args[1])?;
    Some((&args[0], exponent))
}

fn split_power_base_exp(expr: &Expr) -> Option<(Expr, Rational)> {
    let (base, exponent) = split_power_base_exp_ref(expr)?;
    Some((base.clone(), exponent))
}

fn split_power_base_exp_ref(expr: &Expr) -> Option<(&Expr, Rational)> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, "Power") || args.len() != 2 {
        return None;
    }

    let exponent = expr_as_exact_rational(&args[1])?;
    if expr_as_exact_rational(&args[0]).is_some() {
        return None;
    }

    Some((&args[0], exponent))
}

fn split_coeff_kernel(expr: &Expr) -> (Rational, Cow<'_, Expr>) {
    let Expr::Call { head, args } = expr else {
        return (Rational::one(), Cow::Borrowed(expr));
    };
    if !is_system_symbol_name(head, "Times") {
        return (Rational::one(), Cow::Borrowed(expr));
    }

    let mut coefficient = Rational::one();
    let mut rest = Vec::new();
    for arg in args.iter() {
        if let Some(value) = expr_as_exact_rational(arg) {
            coefficient = coefficient.mul(&value);
        } else {
            rest.push(arg);
        }
    }

    let kernel = match rest.as_slice() {
        [] => Cow::Owned(Expr::integer(1_i32.into())),
        [only] => Cow::Borrowed(*only),
        _ => Cow::Owned(Expr::call(
            system_expr("Times"),
            rest.into_iter().cloned().collect(),
        )),
    };

    (coefficient, kernel)
}

fn combine_simple_additive_terms(args: &[Expr]) -> Option<Option<Vec<Expr>>> {
    if args.is_empty() {
        return Some(None);
    }

    let mut grouped: Vec<(Expr, Rational)> = Vec::with_capacity(args.len());
    let mut rebuild_combined = false;
    for term in args {
        let (coefficient, kernel) = simple_coeff_kernel(term)?;
        if let Some((_, existing)) = grouped
            .iter_mut()
            .find(|(candidate, _)| candidate == kernel)
        {
            *existing = existing.add(&coefficient);
            rebuild_combined = true;
        } else {
            if split_coeff_kernel_requires_rebuild(term, &coefficient, kernel) {
                rebuild_combined = true;
            }
            grouped.push((kernel.clone(), coefficient));
        }
    }

    if !rebuild_combined {
        return Some(None);
    }

    let mut combined = Vec::with_capacity(grouped.len());
    for (kernel, coefficient) in grouped {
        if coefficient.is_zero() {
            continue;
        }

        if coefficient.is_one() {
            combined.push(kernel);
        } else {
            combined.push(Expr::call(
                system_expr("Times"),
                vec![Expr::rational(coefficient), kernel],
            ));
        }
    }

    Some(Some(combined))
}

fn simple_coeff_kernel(expr: &Expr) -> Option<(Rational, &Expr)> {
    if matches!(expr, Expr::Atom(_)) {
        return Some((Rational::one(), expr));
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, "Times") || args.len() != 2 {
        return None;
    }

    let coefficient = expr_as_exact_rational(&args[0])?;
    if !matches!(&args[1], Expr::Atom(_)) {
        return None;
    }

    Some((coefficient, &args[1]))
}

fn split_coeff_kernel_requires_rebuild(term: &Expr, coefficient: &Rational, kernel: &Expr) -> bool {
    if coefficient.is_zero() {
        return true;
    }

    if coefficient.is_one() {
        return kernel != term;
    }

    let Expr::Call { head, args } = term else {
        return true;
    };
    if !is_system_symbol_name(head, "Times") || args.is_empty() {
        return true;
    }

    let Some(first_coefficient) = expr_as_exact_rational(&args[0]) else {
        return true;
    };
    if &first_coefficient != coefficient {
        return true;
    }

    match kernel {
        Expr::Call {
            head: kernel_head,
            args: kernel_args,
        } if is_system_symbol_name(kernel_head, "Times") => args[1..] != kernel_args[..],
        _ => args.len() != 2 || &args[1] != kernel,
    }
}
fn expr_as_signed_integer_value(expr: &Expr) -> Option<BigInt> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => Some(value.to_bigint()),
        Expr::Call { head, args } if is_system_symbol_name(head, "Times") && args.len() == 2 => {
            if is_negative_one_expr(&args[0]) {
                expr_as_exact_integer(&args[1]).map(|value| -value)
            } else if is_negative_one_expr(&args[1]) {
                expr_as_exact_integer(&args[0]).map(|value| -value)
            } else {
                None
            }
        }
        _ => None,
    }
}
