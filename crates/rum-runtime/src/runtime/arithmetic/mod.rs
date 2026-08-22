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
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match builtin {
            ArithmeticDataBuiltin::Abs => self.eval_abs(head, args),
            ArithmeticDataBuiltin::Ceiling => self.eval_ceiling(head, args),
            ArithmeticDataBuiltin::Denominator => self.eval_denominator(head, args),
            ArithmeticDataBuiltin::Divide => self.eval_divide(head, args),
            ArithmeticDataBuiltin::Factorial => self.eval_factorial(head, args),
            ArithmeticDataBuiltin::Factorial2 => self.eval_factorial2(head, args),
            ArithmeticDataBuiltin::Floor => self.eval_floor(head, args),
            ArithmeticDataBuiltin::FractionalPart => self.eval_fractional_part(head, args),
            ArithmeticDataBuiltin::GCD => self.eval_gcd(head, args),
            ArithmeticDataBuiltin::Identity => builtin_identity(self, head, args),
            ArithmeticDataBuiltin::IntegerPart => self.eval_integer_part(head, args),
            ArithmeticDataBuiltin::LCM => self.eval_lcm(head, args),
            ArithmeticDataBuiltin::Max => self.eval_max(head, args),
            ArithmeticDataBuiltin::Min => self.eval_min(head, args),
            ArithmeticDataBuiltin::MinMax => self.eval_min_max(head, args),
            ArithmeticDataBuiltin::Minus => self.eval_minus(head, args),
            ArithmeticDataBuiltin::Mod => self.eval_mod(head, args),
            ArithmeticDataBuiltin::Numerator => self.eval_numerator(head, args),
            ArithmeticDataBuiltin::Plus => self.eval_plus(head, args),
            ArithmeticDataBuiltin::Power => self.eval_power(head, args),
            ArithmeticDataBuiltin::PowerMod => self.eval_power_mod(head, args),
            ArithmeticDataBuiltin::Quotient => self.eval_quotient(head, args),
            ArithmeticDataBuiltin::Round => self.eval_round(head, args),
            ArithmeticDataBuiltin::Sign => self.eval_sign(head, args),
            ArithmeticDataBuiltin::Subtract => self.eval_subtract(head, args),
            ArithmeticDataBuiltin::Times => self.eval_times(head, args),
            ArithmeticDataBuiltin::Total => self.eval_total(head, args),
        }
    }
}
