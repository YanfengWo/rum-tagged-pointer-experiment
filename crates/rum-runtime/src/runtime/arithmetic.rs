use super::*;
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

    pub(super) fn eval_abs(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("Abs", args, 1));
        }

        let Some(value) = expr_as_exact_rational(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::rational(
            Rational::new(bigint_abs(value.numerator()), value.denominator().clone())
                .expect("absolute exact rational keeps a non-zero denominator"),
        ))
    }

    pub(super) fn eval_sign(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [arg] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Some(value) = expr_as_exact_rational(arg) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let sign = match value.numerator().cmp(&BigInt::from(0_u8)) {
            std::cmp::Ordering::Less => -1_i32,
            std::cmp::Ordering::Equal => 0_i32,
            std::cmp::Ordering::Greater => 1_i32,
        };
        Ok(Expr::integer(BigInt::from(sign)))
    }

    pub(super) fn eval_floor(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [arg] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Some(value) = expr_as_exact_rational(arg) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::integer(floor_exact_rational_to_integer(&value)))
    }

    pub(super) fn eval_ceiling(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [arg] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Some(value) = expr_as_exact_rational(arg) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::integer(ceiling_exact_rational_to_integer(&value)))
    }

    pub(super) fn eval_integer_part(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [arg] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Some(value) = expr_as_exact_rational(arg) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::integer(truncate_exact_rational_to_integer(&value)))
    }

    pub(super) fn eval_fractional_part(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [arg] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Some(value) = expr_as_exact_rational(arg) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let integer_part = Rational::from_integer(truncate_exact_rational_to_integer(&value));
        Ok(Expr::rational(value.sub(&integer_part)))
    }

    pub(super) fn eval_minus(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [arg] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        self.combine_multiplicative_args(
            &system_expr("Times"),
            &[Expr::integer((-1_i32).into()), arg.clone()],
        )
    }

    pub(super) fn eval_round(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let (value_expr, multiple_expr) = match args {
            [value] => (value, None),
            [value, multiple] => (value, Some(multiple)),
            _ => return Ok(self.invalid_call_with_arg_range("Round", args, 1, 2)),
        };

        let Some(value) = expr_as_exact_rational(value_expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let multiple = match multiple_expr {
            None => Rational::from_integer(BigInt::from(1_u8)),
            Some(multiple_expr) => {
                let Some(multiple) = expr_as_exact_rational(multiple_expr) else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                };
                if multiple.numerator() == &BigInt::from(0_u8) {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                }
                multiple
            }
        };

        let scaled = value
            .div(&multiple)
            .expect("zero multiple is rejected before exact division");
        let rounded = round_exact_rational_to_integer(&scaled);
        Ok(Expr::rational(
            Rational::from_integer(rounded).mul(&multiple),
        ))
    }

    pub(super) fn eval_factorial(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("Factorial", args, 1));
        }

        let Some(value) = expr_as_exact_integer(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        if value < BigInt::from(0_u8) {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }
        self.enforce_integer_work_limit_for_factorial(&value)?;

        let one = BigInt::from(1_u8);
        let mut remaining = value;
        let mut result = one.clone();
        let mut checks_until_resource_poll = 0_usize;
        while remaining > one {
            result *= remaining.clone();
            remaining -= BigInt::from(1_u8);

            if checks_until_resource_poll == 0 {
                self.enforce_active_resource_limits_for_expr(&Expr::integer(result.clone()))?;
                checks_until_resource_poll = 256;
            } else {
                checks_until_resource_poll -= 1;
            }
        }

        Ok(Expr::integer(result))
    }

    pub(super) fn eval_factorial2(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("Factorial2", args, 1));
        }

        let Some(value) = expr_as_exact_integer(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        if value < BigInt::from(0_u8) {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }
        self.enforce_integer_work_limit_for_factorial(&value)?;

        let one = BigInt::from(1_u8);
        let two = BigInt::from(2_u8);
        let mut remaining = value;
        let mut result = one.clone();
        let mut checks_until_resource_poll = 0_usize;
        while remaining > one {
            result *= remaining.clone();
            remaining -= two.clone();

            if checks_until_resource_poll == 0 {
                self.enforce_active_resource_limits_for_expr(&Expr::integer(result.clone()))?;
                checks_until_resource_poll = 256;
            } else {
                checks_until_resource_poll -= 1;
            }
        }

        Ok(Expr::integer(result))
    }

    pub(super) fn eval_total(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }

        let Expr::Call {
            head: list_head,
            args: items,
        } = &args[0]
        else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        if !is_system_symbol_name(list_head, "List") {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }

        self.evaluate(Expr::call_with_shared_args(
            system_expr("Plus"),
            items.clone(),
        ))
    }

    pub(super) fn eval_denominator(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("Denominator", args, 1));
        }

        match &args[0] {
            Expr::Atom(Atom::Integer(_)) => Ok(Expr::integer(1_i32.into())),
            Expr::Atom(Atom::Rational(value)) => Ok(Expr::integer(value.denominator().clone())),
            _ => Ok(Expr::call(head.clone(), args.to_vec())),
        }
    }

    pub(super) fn eval_numerator(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("Numerator", args, 1));
        }

        match &args[0] {
            Expr::Atom(Atom::Integer(value)) => Ok(Expr::integer(value.to_bigint())),
            Expr::Atom(Atom::Rational(value)) => Ok(Expr::integer(value.numerator().clone())),
            _ => Ok(Expr::call(head.clone(), args.to_vec())),
        }
    }

    fn enforce_integer_work_limit_for_factorial(&self, value: &BigInt) -> Result<(), EvalError> {
        let one = BigInt::from(1_u8);
        let work_units = if value > &one {
            value - one
        } else {
            BigInt::from(0_u8)
        };
        let limit = self.resource_limits.integer_work_limit;
        if work_units > BigInt::from(limit) {
            return Err(EvalError::IntegerWorkLimitExceeded { limit });
        }

        Ok(())
    }

    pub(super) fn eval_mod(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Mod", args, 2));
        }

        let (Some(lhs), Some(rhs)) = (
            expr_as_exact_integer(&args[0]),
            expr_as_exact_integer(&args[1]),
        ) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(mod_integer(&lhs, &rhs)
            .map(Expr::integer)
            .unwrap_or_else(|| Expr::call(head.clone(), args.to_vec())))
    }

    pub(super) fn eval_quotient(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Quotient", args, 2));
        }

        let (Some(lhs), Some(rhs)) = (
            expr_as_exact_integer(&args[0]),
            expr_as_exact_integer(&args[1]),
        ) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(floor_quotient(&lhs, &rhs)
            .map(Expr::integer)
            .unwrap_or_else(|| Expr::call(head.clone(), args.to_vec())))
    }

    pub(super) fn eval_gcd(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.is_empty() {
            return Ok(Expr::integer(0_i32.into()));
        }

        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            let Some(value) = expr_as_exact_integer(arg) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            values.push(value);
        }

        let mut gcd = bigint_abs(&values[0]);
        for value in &values[1..] {
            gcd = bigint_gcd(gcd, bigint_abs(value));
        }

        Ok(Expr::integer(gcd))
    }

    pub(super) fn eval_lcm(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.is_empty() {
            return Ok(Expr::integer(1_i32.into()));
        }

        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            let Some(value) = expr_as_exact_integer(arg) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            values.push(value);
        }

        let zero = BigInt::from(0_u8);
        let mut lcm = bigint_abs(&values[0]);

        for value in &values[1..] {
            let normalized = bigint_abs(value);
            if lcm == zero || normalized == zero {
                lcm = zero.clone();
                break;
            }

            let divisor = bigint_gcd(lcm.clone(), normalized.clone());
            lcm = (lcm / divisor) * normalized;
        }

        Ok(Expr::integer(lcm))
    }

    pub(super) fn eval_max(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_exact_extremum(head, args, true)
    }

    pub(super) fn eval_min(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_exact_extremum(head, args, false)
    }

    pub(super) fn eval_min_max(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [arg] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let (min, max) = match arg {
            Expr::Call {
                head: list_head,
                args: items,
            } if is_system_symbol_name(list_head, "List") => {
                let mut operands = Vec::new();
                flatten_list_operands(items.as_ref(), &mut operands);
                match operands.as_slice() {
                    [] => {
                        let min_head = system_expr("Min");
                        let max_head = system_expr("Max");
                        (
                            self.eval_min(&min_head, &[])?,
                            self.eval_max(&max_head, &[])?,
                        )
                    }
                    [only] => ((*only).clone(), (*only).clone()),
                    _ => {
                        let min_head = system_expr("Min");
                        let max_head = system_expr("Max");
                        let orderless = AttributeSet::from_attribute(Attribute::Orderless);
                        let extrema_args = operands.into_iter().cloned().collect();
                        let extrema_args = self.normalize_call_args_by_attributes(
                            &min_head,
                            extrema_args,
                            orderless,
                        );
                        (
                            self.eval_min(&min_head, &extrema_args)?,
                            self.eval_max(&max_head, &extrema_args)?,
                        )
                    }
                }
            }
            _ => (arg.clone(), arg.clone()),
        };

        Ok(Expr::call(system_expr("List"), vec![min, max]))
    }

    fn eval_exact_extremum(
        &mut self,
        head: &Expr,
        args: &[Expr],
        select_max: bool,
    ) -> Result<Expr, EvalError> {
        let mut operands = Vec::new();
        flatten_list_operands(args, &mut operands);

        let Some(first) = operands.first().copied() else {
            return Ok(if select_max {
                Expr::call(
                    system_expr("Times"),
                    vec![Expr::integer((-1_i32).into()), system_expr("Infinity")],
                )
            } else {
                system_expr("Infinity")
            });
        };

        let Some(mut best_value) = expr_as_exact_rational(first) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let mut best_expr = first;

        for operand in operands.iter().skip(1).copied() {
            let Some(value) = expr_as_exact_rational(operand) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };

            let ordering = compare_exact_rational_values(&value, &best_value);
            if (select_max && ordering == std::cmp::Ordering::Greater)
                || (!select_max && ordering == std::cmp::Ordering::Less)
            {
                best_value = value;
                best_expr = operand;
            }
        }

        Ok(best_expr.clone())
    }

    pub(super) fn eval_power_mod(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 3 {
            return Ok(self.invalid_call_with_exact_arg_count("PowerMod", args, 3));
        }

        let (Some(base), Some(exponent), Some(modulus)) = (
            expr_as_exact_integer(&args[0]),
            expr_as_exact_integer(&args[1]),
            expr_as_exact_integer(&args[2]),
        ) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        if exponent < BigInt::from(0_u8) {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }

        Ok(power_mod_integer(&base, &exponent, &modulus)
            .map(Expr::integer)
            .unwrap_or_else(|| Expr::call(head.clone(), args.to_vec())))
    }

    pub(super) fn combine_additive_args(
        &self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
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

    pub(super) fn combine_multiplicative_args(
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

pub(super) fn expr_as_exact_integer(expr: &Expr) -> Option<BigInt> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => Some(value.to_bigint()),
        _ => None,
    }
}

pub(super) fn expr_as_exact_rational(expr: &Expr) -> Option<Rational> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => Some(Rational::from_integer(value.to_bigint())),
        Expr::Atom(Atom::Rational(value)) => Some(value.as_ref().clone()),
        _ => None,
    }
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

fn flatten_list_operands<'a>(args: &'a [Expr], operands: &mut Vec<&'a Expr>) {
    for arg in args {
        flatten_list_operand(arg, operands);
    }
}

fn flatten_list_operand<'a>(expr: &'a Expr, operands: &mut Vec<&'a Expr>) {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
            for arg in args.iter() {
                flatten_list_operand(arg, operands);
            }
        }
        _ => operands.push(expr),
    }
}

fn compare_exact_rational_values(lhs: &Rational, rhs: &Rational) -> std::cmp::Ordering {
    let lhs_cross = lhs.numerator() * rhs.denominator();
    let rhs_cross = rhs.numerator() * lhs.denominator();
    lhs_cross.cmp(&rhs_cross)
}

fn floor_exact_rational_to_integer(value: &Rational) -> BigInt {
    floor_quotient(value.numerator(), value.denominator())
        .expect("normalized exact rational has a non-zero denominator")
}

fn ceiling_exact_rational_to_integer(value: &Rational) -> BigInt {
    -floor_quotient(&(-value.numerator().clone()), value.denominator())
        .expect("normalized exact rational has a non-zero denominator")
}

fn truncate_exact_rational_to_integer(value: &Rational) -> BigInt {
    value.numerator() / value.denominator()
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

fn round_exact_rational_to_integer(value: &Rational) -> BigInt {
    let numerator = value.numerator();
    let denominator = value.denominator();
    let negative = numerator < &BigInt::from(0_u8);
    let abs_numerator = bigint_abs(numerator);
    let quotient = &abs_numerator / denominator;
    let remainder = &abs_numerator % denominator;
    let twice_remainder = remainder * BigInt::from(2_u8);

    let rounded_abs = if twice_remainder < *denominator {
        quotient
    } else if twice_remainder > *denominator {
        quotient + BigInt::from(1_u8)
    } else if &quotient % BigInt::from(2_u8) == BigInt::from(0_u8) {
        quotient
    } else {
        quotient + BigInt::from(1_u8)
    };

    if negative { -rounded_abs } else { rounded_abs }
}

fn bigint_abs(value: &BigInt) -> BigInt {
    if value < &BigInt::from(0_u8) {
        -value.clone()
    } else {
        value.clone()
    }
}

fn bigint_gcd(mut lhs: BigInt, mut rhs: BigInt) -> BigInt {
    let zero = BigInt::from(0_u8);

    while rhs != zero {
        let remainder = &lhs % &rhs;
        lhs = rhs;
        rhs = remainder;
    }

    bigint_abs(&lhs)
}

fn mod_integer(lhs: &BigInt, rhs: &BigInt) -> Option<BigInt> {
    if rhs == &BigInt::from(0_u8) {
        return None;
    }

    let quotient = floor_quotient(lhs, rhs)?;
    Some(lhs - rhs * quotient)
}

fn floor_quotient(lhs: &BigInt, rhs: &BigInt) -> Option<BigInt> {
    if rhs == &BigInt::from(0_u8) {
        return None;
    }

    let zero = BigInt::from(0_u8);
    let quotient = lhs / rhs;
    let remainder = lhs % rhs;

    if remainder != zero && ((remainder > zero && rhs < &zero) || (remainder < zero && rhs > &zero))
    {
        Some(quotient - BigInt::from(1_u8))
    } else {
        Some(quotient)
    }
}

fn power_mod_integer(base: &BigInt, exponent: &BigInt, modulus: &BigInt) -> Option<BigInt> {
    if modulus == &BigInt::from(0_u8) {
        return None;
    }

    let zero = BigInt::from(0_u8);
    let one = BigInt::from(1_u8);
    let two = BigInt::from(2_u8);
    let modulus_abs = bigint_abs(modulus);
    let mut result = mod_integer(&one, modulus)?;
    let mut factor = mod_integer(base, modulus)?;
    let mut remaining = exponent.clone();

    while remaining > zero {
        if &remaining % &two == one {
            result = mod_integer(&(result * &factor), modulus)?;
        }

        remaining /= &two;

        if remaining > zero {
            factor = mod_integer(&(&factor * &factor), modulus)?;
        }
    }

    let mut reduced = mod_integer(&result, modulus)?;
    if modulus < &zero && reduced > zero {
        reduced -= modulus_abs;
    }

    Some(reduced)
}
