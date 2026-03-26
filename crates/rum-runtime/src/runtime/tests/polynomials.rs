use super::*;

#[test]
fn expand_distributes_products_and_positive_integer_powers() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Expand[(x + 1) * (x - 1)]")
            .unwrap()
            .value
            .to_string(),
        "Plus[-1, Power[x, 2]]"
    );
    assert_eq!(
        session
            .eval_text("Expand[(x + y)^3]")
            .unwrap()
            .value
            .to_string(),
        "Plus[Power[x, 3], Power[y, 3], Times[3, Power[x, 2], y], Times[3, Power[y, 2], x]]"
    );
}

#[test]
fn expand_supports_rational_coefficients_and_indexed_indeterminates() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Expand[(x + 1/2) * (x - 1/3)]")
            .unwrap()
            .value
            .to_string(),
        "Plus[-1/6, Power[x, 2], Times[1/6, x]]"
    );
    assert_eq!(
        session
            .eval_text("Expand[(u[1] + u)^2]")
            .unwrap()
            .value
            .to_string(),
        "Plus[Power[u, 2], Power[u[1], 2], Times[2, u, u[1]]]"
    );
}

#[test]
fn expand_preserves_non_polynomial_or_pattern_bearing_regions() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Expand[(y + 1) * f[(x + 1)^2]]")
            .unwrap()
            .value
            .to_string(),
        "Plus[Times[f[Power[Plus[1, x], 2]], y], f[Power[Plus[1, x], 2]]]"
    );
    assert_eq!(
        session
            .eval_text("Expand[(u[i_] + 1)^2]")
            .unwrap()
            .value
            .to_string(),
        "Power[Plus[1, u[Pattern[i, Blank[]]]], 2]"
    );
}

#[test]
fn coefficient_extracts_exact_monomial_powers_without_requiring_manual_expand() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Coefficient[(x + y)^3, x]")
            .unwrap()
            .value
            .to_string(),
        "Times[3, Power[y, 2]]"
    );
    assert_eq!(
        session
            .eval_text("Coefficient[(x + y)^3, x, 2]")
            .unwrap()
            .value
            .to_string(),
        "Times[3, y]"
    );
    assert_eq!(
        session
            .eval_text("Coefficient[x^2*y + x*y^2 + y, x*y]")
            .unwrap()
            .value
            .to_string(),
        "Plus[x, y]"
    );
    assert_eq!(
        session
            .eval_text("Coefficient[(u[1] + u)^2, u[1]]")
            .unwrap()
            .value
            .to_string(),
        "Times[2, u]"
    );
    assert_eq!(
        session
            .eval_text("Coefficient[x + 1, x, -1]")
            .unwrap()
            .value
            .to_string(),
        "0"
    );
}

#[test]
fn coefficient_list_builds_rectangular_grids_in_the_requested_variable_order() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("CoefficientList[x^2 + 2*x*y + y^2 + 1, x]")
            .unwrap()
            .value
            .to_string(),
        "List[Plus[1, Power[y, 2]], Times[2, y], 1]"
    );
    assert_eq!(
        session
            .eval_text("CoefficientList[x^2 + 2*x*y + y^2 + 1, {x, y}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[1, 0, 1], List[0, 2, 0], List[1, 0, 0]]"
    );
    assert_eq!(
        session
            .eval_text("CoefficientList[(u[1] + u)^2, {u[1], u}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[0, 0, 1], List[0, 2, 0], List[1, 0, 0]]"
    );
}

#[test]
fn coefficient_family_keeps_current_pattern_bearing_boundary_for_now() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Coefficient[(u[i_] + 1)^2, u[i_]]")
            .unwrap()
            .value
            .to_string(),
        "Coefficient[Power[Plus[1, u[Pattern[i, Blank[]]]], 2], u[Pattern[i, Blank[]]]]"
    );
    assert_eq!(
        session
            .eval_text("CoefficientList[(u[i_] + 1)^2, u[i_]]")
            .unwrap()
            .value
            .to_string(),
        "CoefficientList[Power[Plus[1, u[Pattern[i, Blank[]]]], 2], u[Pattern[i, Blank[]]]]"
    );
}

#[test]
fn together_combines_rational_function_terms_over_common_polynomial_denominators() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Together[1/x + 1/y]")
            .unwrap()
            .value
            .to_string(),
        session
            .eval_text("(x + y)/(x*y)")
            .unwrap()
            .value
            .to_string()
    );
    assert_eq!(
        session
            .eval_text("Together[1/(x + 1) + 1/(x - 1)]")
            .unwrap()
            .value
            .to_string(),
        session
            .eval_text("(2*x)/(x^2 - 1)")
            .unwrap()
            .value
            .to_string()
    );
    assert_eq!(
        session
            .eval_text("Together[((x/y)^-1) + 1]")
            .unwrap()
            .value
            .to_string(),
        session.eval_text("(x + y)/x").unwrap().value.to_string()
    );
}

#[test]
fn together_keeps_current_pattern_bearing_boundary_for_now() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Together[1/(u[i_] + 1) + 1/x]")
            .unwrap()
            .value
            .to_string(),
        "Together[Plus[Power[Plus[1, u[Pattern[i, Blank[]]]], -1], Power[x, -1]]]"
    );
}

#[test]
fn factor_extracts_content_common_monomials_and_linear_factors_over_q() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Factor[2*x^2 + 4*x*y + 2*y^2]")
            .unwrap()
            .value
            .to_string(),
        session.eval_text("2*(x + y)^2").unwrap().value.to_string()
    );
    assert_eq!(
        session
            .eval_text("Factor[x^2 - y^2]")
            .unwrap()
            .value
            .to_string(),
        session
            .eval_text("(x - y)*(x + y)")
            .unwrap()
            .value
            .to_string()
    );
    assert_eq!(
        session
            .eval_text("Factor[x^3 - y^3]")
            .unwrap()
            .value
            .to_string(),
        session
            .eval_text("(x - y)*(x^2 + x*y + y^2)")
            .unwrap()
            .value
            .to_string()
    );
}

#[test]
fn factor_handles_symmetric_multivariate_cubic_example() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Factor[x^3 + y^3 + z^3 - 3*x*y*z]")
            .unwrap()
            .value
            .to_string(),
        session
            .eval_text("(x + y + z)*(x^2 + y^2 + z^2 - x*y - x*z - y*z)")
            .unwrap()
            .value
            .to_string()
    );
}

#[test]
fn together_now_cancels_common_polynomial_factors_after_combining() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Together[(x^2 - 1)/(x - 1)]")
            .unwrap()
            .value
            .to_string(),
        session.eval_text("x + 1").unwrap().value.to_string()
    );
    assert_eq!(
        session
            .eval_text("Together[(x^2 - y^2)/(x - y)]")
            .unwrap()
            .value
            .to_string(),
        session.eval_text("x + y").unwrap().value.to_string()
    );
}

#[test]
fn factor_keeps_current_pattern_bearing_boundary_for_now() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Factor[(u[i_] + u[j_])^2]")
            .unwrap()
            .value
            .to_string(),
        "Power[Plus[u[Pattern[i, Blank[]]], u[Pattern[j, Blank[]]]], 2]"
    );
}
