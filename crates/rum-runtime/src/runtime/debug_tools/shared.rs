use super::*;

pub(super) fn render_echo_fragment(expr: &Expr) -> String {
    match expr {
        Expr::Atom(Atom::String(value)) => value.clone(),
        other => other.to_string(),
    }
}
pub(super) fn debug_bool_expr(value: bool) -> Expr {
    if value {
        system_expr("True")
    } else {
        system_expr("False")
    }
}
pub(super) fn elapsed_duration_expr(duration: Duration) -> Expr {
    let numerator = BigInt::from(duration.as_nanos());
    let denominator = BigInt::from(1_000_000_000_u64);
    let rational = Rational::new(numerator, denominator)
        .expect("positive denominator should always build a rational");
    Expr::rational(rational)
}
pub(super) fn runtime_message_emit_expr(message: RuntimeMessage) -> Expr {
    Expr::call(
        system_expr("Hold"),
        vec![Expr::call(
            system_expr("Message"),
            vec![runtime_message_name_expr(message)],
        )],
    )
}
