use super::*;

mod canonical;
mod exact;
mod extremum;
mod factorial;
mod group_index;
mod integer_ops;
mod parts;
mod rounding;

pub use exact::{expr_as_exact_integer, expr_as_exact_rational};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArithmeticDataBuiltin {
    Abs,
    Ceiling,
    Denominator,
    Divide,
    Factorial,
    Factorial2,
    Floor,
    FractionalPart,
    GCD,
    Identity,
    IntegerPart,
    LCM,
    Max,
    Min,
    MinMax,
    Minus,
    Mod,
    Numerator,
    Plus,
    Power,
    PowerMod,
    Quotient,
    Round,
    Sign,
    Subtract,
    Times,
    Total,
}

impl Session {
    /// Dispatches arithmetic-family builtin handlers whose bodies remain kernel-side.
    pub fn eval_arithmetic_data_builtin(
        &mut self,
        builtin: ArithmeticDataBuiltin,
        head: &Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        match builtin {
            ArithmeticDataBuiltin::Abs => self.eval_abs(head, args_slice),
            ArithmeticDataBuiltin::Ceiling => self.eval_ceiling(head, args_slice),
            ArithmeticDataBuiltin::Denominator => self.eval_denominator(head, args_slice),
            ArithmeticDataBuiltin::Divide => self.eval_divide(head, args_slice),
            ArithmeticDataBuiltin::Factorial => self.eval_factorial(head, args_slice),
            ArithmeticDataBuiltin::Factorial2 => self.eval_factorial2(head, args_slice),
            ArithmeticDataBuiltin::Floor => self.eval_floor(head, args_slice),
            ArithmeticDataBuiltin::FractionalPart => self.eval_fractional_part(head, args_slice),
            ArithmeticDataBuiltin::GCD => self.eval_gcd(head, args_slice),
            ArithmeticDataBuiltin::Identity => builtin_identity(self, head, args),
            ArithmeticDataBuiltin::IntegerPart => self.eval_integer_part(head, args_slice),
            ArithmeticDataBuiltin::LCM => self.eval_lcm(head, args_slice),
            ArithmeticDataBuiltin::Max => self.eval_max(head, args_slice),
            ArithmeticDataBuiltin::Min => self.eval_min(head, args_slice),
            ArithmeticDataBuiltin::MinMax => self.eval_min_max(head, args_slice),
            ArithmeticDataBuiltin::Minus => self.eval_minus(head, args_slice),
            ArithmeticDataBuiltin::Mod => self.eval_mod(head, args_slice),
            ArithmeticDataBuiltin::Numerator => self.eval_numerator(head, args_slice),
            ArithmeticDataBuiltin::Plus => self.eval_plus(head, args_slice),
            ArithmeticDataBuiltin::Power => self.eval_power(head, args_slice),
            ArithmeticDataBuiltin::PowerMod => self.eval_power_mod(head, args_slice),
            ArithmeticDataBuiltin::Quotient => self.eval_quotient(head, args_slice),
            ArithmeticDataBuiltin::Round => self.eval_round(head, args_slice),
            ArithmeticDataBuiltin::Sign => self.eval_sign(head, args_slice),
            ArithmeticDataBuiltin::Subtract => self.eval_subtract(head, args_slice),
            ArithmeticDataBuiltin::Times => self.eval_times(head, args_slice),
            ArithmeticDataBuiltin::Total => self.eval_total(head, args_slice),
        }
    }
}
