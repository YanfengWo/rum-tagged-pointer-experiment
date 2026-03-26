use super::*;

#[test]
fn d_handles_polynomial_and_rational_function_rules() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("D[x^3 + y^3, x]")
            .unwrap()
            .value
            .to_string(),
        session.eval_text("3*x^2").unwrap().value.to_string()
    );
    assert_eq!(
        session
            .eval_text("D[1/x + 1/y, x]")
            .unwrap()
            .value
            .to_string(),
        session.eval_text("-1*x^-2").unwrap().value.to_string()
    );
    assert_eq!(
        session
            .eval_text("Together[D[(x + 1)/(x - 1), x]]")
            .unwrap()
            .value
            .to_string(),
        session
            .eval_text("-2*(1 + x^2 - 2*x)^-1")
            .unwrap()
            .value
            .to_string()
    );
}

#[test]
fn d_supports_indexed_variables_and_higher_order_specs() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("D[(u[1] + u)^2, u[1]]")
            .unwrap()
            .value
            .to_string(),
        session.eval_text("2*(u + u[1])").unwrap().value.to_string()
    );
    assert_eq!(
        session
            .eval_text("D[f[x], {x, 2}]")
            .unwrap()
            .value
            .to_string(),
        "Derivative[2][f][x]"
    );
    assert_eq!(
        session
            .eval_text("D[f[x, y], x, y]")
            .unwrap()
            .value
            .to_string(),
        "Derivative[1, 1][f][x, y]"
    );
}

#[test]
fn d_uses_abstract_derivative_forms_for_symbol_headed_calls() {
    let mut session = Session::new();

    assert_eq!(
        session.eval_text("D[f[x], x]").unwrap().value.to_string(),
        "Derivative[1][f][x]"
    );
    assert_eq!(
        session
            .eval_text("D[f[g[x], y], x]")
            .unwrap()
            .value
            .to_string(),
        session
            .eval_text("Derivative[1, 0][f][g[x], y] * Derivative[1][g][x]")
            .unwrap()
            .value
            .to_string()
    );
}

#[test]
fn d_keeps_current_variable_boundary_for_pattern_or_non_symbol_heads() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("D[(u[i_] + 1)^2, u[i_]]")
            .unwrap()
            .value
            .to_string(),
        "D[Power[Plus[1, u[Pattern[i, Blank[]]]], 2], u[Pattern[i, Blank[]]]]"
    );
    assert_eq!(
        session
            .eval_text("D[x^2, h[x][1]]")
            .unwrap()
            .value
            .to_string(),
        "D[Power[x, 2], h[x][1]]"
    );
}
