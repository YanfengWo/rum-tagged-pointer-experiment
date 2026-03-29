use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
enum TensorNode {
    Scalar(Expr),
    Array(Vec<TensorNode>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ArrayFilterPadding {
    Uniform(ArrayFilterPaddingLeaf),
    PerAxis(Vec<ArrayFilterAxisPadding>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ArrayFilterAxisPadding {
    before: ArrayFilterPaddingLeaf,
    after: ArrayFilterPaddingLeaf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ArrayFilterPaddingLeaf {
    Fixed,
    Periodic,
    Constant(Expr),
    None,
    Cycle(Vec<Expr>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ArrayFilterSpec {
    Radii(Vec<usize>),
    Template(ArrayFilterTemplate),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ArrayFilterTemplate {
    shape: Vec<usize>,
    center: Vec<usize>,
    mask: Vec<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ArrayReduceSpec {
    Elementwise,
    Grouped(Vec<Vec<usize>>),
}

impl Session {
    pub(in crate::runtime) fn eval_inner(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let (f, left, right, g) = match args {
            [f, left, right] => (f, left, right, system_expr("Plus")),
            [f, left, right, g] => (f, left, right, g.clone()),
            _ => return Ok(self.invalid_call_with_arg_range("Inner", args, 3, 4)),
        };

        let Some(result) = compute_inner(self, f, left, right, &g)? else {
            return Ok(Expr::call(system_expr("Inner"), args.to_vec()));
        };

        Ok(result)
    }

    pub(in crate::runtime) fn eval_array_filter(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let (operator, array, radius_spec, padding_expr) = match args {
            [operator, array, radius_spec] => (operator, array, radius_spec, None),
            [operator, array, radius_spec, padding_expr] => {
                (operator, array, radius_spec, Some(padding_expr))
            }
            _ => return Ok(self.invalid_call_with_arg_range("ArrayFilter", args, 3, 4)),
        };

        let Some((shape, flat)) = parse_rectangular_list_array(array) else {
            return Ok(self.invalid_call_with_arg_form(
                "ArrayFilter",
                args,
                "expected a rectangular List array as the second argument",
            ));
        };
        let Some(spec) = parse_array_filter_spec(radius_spec, shape.len()) else {
            return Ok(self.invalid_call_with_arg_form(
                "ArrayFilter",
                args,
                "expected a non-negative integer radius, a matching list of non-negative integer radii, or a matching rectangular 0/1 template array as the third argument",
            ));
        };
        let padding = match padding_expr {
            None => ArrayFilterPadding::Uniform(ArrayFilterPaddingLeaf::Fixed),
            Some(padding_expr) => {
                let Some(parsed) = parse_array_filter_padding(padding_expr, shape.len()) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "ArrayFilter",
                        args,
                        "expected Padding -> value, Padding -> None, Padding -> \"Fixed\", Padding -> \"Periodic\", a cyclic padding list, or matching per-axis padding pairs as the fourth argument",
                    ));
                };
                parsed
            }
        };

        apply_array_filter(self, operator, &shape, &flat, &spec, &padding)
    }

    pub(in crate::runtime) fn eval_array_reduce(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [operator, array, dims_spec] = args else {
            return Ok(self.invalid_call_with_arg_range("ArrayReduce", args, 3, 3));
        };

        let Some((shape, flat)) = parse_rectangular_list_array(array) else {
            return Ok(self.invalid_call_with_arg_form(
                "ArrayReduce",
                args,
                "expected a rectangular List array as the second argument",
            ));
        };
        let Some(spec) = parse_array_reduce_spec(dims_spec, shape.len()) else {
            return Ok(self.invalid_call_with_arg_form(
                "ArrayReduce",
                args,
                "expected a positive integer dimension, a dimension Span, a list of dimensions, or a grouped dimension list as the third argument",
            ));
        };

        apply_array_reduce(self, operator, &shape, &flat, &spec)
    }
}

fn parse_array_filter_padding(expr: &Expr, rank: usize) -> Option<ArrayFilterPadding> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return None;
    }

    let symbol = expr_as_symbol(&args[0])?;
    if symbol.name() != "Padding" {
        return None;
    }

    if let Some(per_axis) = parse_array_filter_per_axis_padding(&args[1], rank) {
        return Some(ArrayFilterPadding::PerAxis(per_axis));
    }

    parse_array_filter_padding_leaf(&args[1]).map(ArrayFilterPadding::Uniform)
}

fn parse_array_filter_per_axis_padding(
    expr: &Expr,
    rank: usize,
) -> Option<Vec<ArrayFilterAxisPadding>> {
    let items = call_items_with_head(expr, &system_expr("List"))?;
    if items.len() != rank {
        return None;
    }
    if !items
        .iter()
        .any(|item| call_items_with_head(item, &system_expr("List")).is_some())
    {
        return None;
    }

    let mut paddings = Vec::with_capacity(rank);
    for item in items {
        let side_items = call_items_with_head(item, &system_expr("List"))?;
        let axis_padding = match side_items {
            [leaf] => {
                let leaf = parse_array_filter_padding_leaf(leaf)?;
                ArrayFilterAxisPadding {
                    before: leaf.clone(),
                    after: leaf,
                }
            }
            [before, after] => ArrayFilterAxisPadding {
                before: parse_array_filter_padding_leaf(before)?,
                after: parse_array_filter_padding_leaf(after)?,
            },
            _ => return None,
        };
        paddings.push(axis_padding);
    }

    Some(paddings)
}

fn parse_array_filter_padding_leaf(expr: &Expr) -> Option<ArrayFilterPaddingLeaf> {
    match expr {
        value if expr_as_symbol(value).is_some_and(|symbol| symbol.name() == "None") => {
            Some(ArrayFilterPaddingLeaf::None)
        }
        Expr::Atom(Atom::String(value)) if value == "Fixed" => Some(ArrayFilterPaddingLeaf::Fixed),
        Expr::Atom(Atom::String(value)) if value == "Periodic" => {
            Some(ArrayFilterPaddingLeaf::Periodic)
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "List") && !args.is_empty() => {
            Some(ArrayFilterPaddingLeaf::Cycle(args.clone()))
        }
        other => Some(ArrayFilterPaddingLeaf::Constant(other.clone())),
    }
}

fn parse_rectangular_list_array(expr: &Expr) -> Option<(Vec<usize>, Vec<Expr>)> {
    let Expr::Call { head, .. } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, "List") {
        return None;
    }

    let tensor = parse_tensor_node(expr, &system_expr("List"));
    let shape = tensor_shape(&tensor)?;
    let flat = flatten_tensor_scalars(&tensor);
    Some((shape, flat))
}

fn parse_array_filter_spec(expr: &Expr, rank: usize) -> Option<ArrayFilterSpec> {
    if let Some(radius) = expr_as_non_negative_usize(expr) {
        return Some(ArrayFilterSpec::Radii(vec![radius; rank]));
    }

    if let Some(items) = call_items_with_head(expr, &system_expr("List")) {
        if items.len() == rank {
            if let Some(radii) = items.iter().map(expr_as_non_negative_usize).collect() {
                return Some(ArrayFilterSpec::Radii(radii));
            }
        }
    }

    parse_array_filter_template(expr, rank).map(ArrayFilterSpec::Template)
}

fn parse_array_filter_template(expr: &Expr, rank: usize) -> Option<ArrayFilterTemplate> {
    let (shape, flat) = parse_rectangular_list_array(expr)?;
    if shape.len() != rank || shape.contains(&0) {
        return None;
    }

    let mask = flat
        .into_iter()
        .map(|item| match item {
            Expr::Atom(Atom::Integer(value)) if value == BigInt::from(1_u8) => Some(true),
            Expr::Atom(Atom::Integer(value)) if value == BigInt::from(0_u8) => Some(false),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    // For even-sized templates, Wolfram anchors neighborhoods with maximal
    // overhang at the beginning side, which corresponds to floor(len/2).
    let center = shape.iter().map(|len| len / 2).collect();

    Some(ArrayFilterTemplate {
        shape,
        center,
        mask,
    })
}

fn parse_array_reduce_spec(expr: &Expr, rank: usize) -> Option<ArrayReduceSpec> {
    if let Some(dimension) = expr_as_positive_usize(expr) {
        return (dimension <= rank).then_some(ArrayReduceSpec::Grouped(vec![vec![dimension - 1]]));
    }

    if let Some(span) = parse_array_reduce_span(expr, rank) {
        return Some(ArrayReduceSpec::Grouped(vec![span]));
    }

    let items = call_items_with_head(expr, &system_expr("List"))?;
    if items.is_empty() {
        return Some(ArrayReduceSpec::Elementwise);
    }

    if items
        .iter()
        .all(|item| call_items_with_head(item, &system_expr("List")).is_some())
    {
        let mut seen = std::collections::HashSet::new();
        let mut groups = Vec::with_capacity(items.len());
        for group_expr in items {
            let group_items = call_items_with_head(group_expr, &system_expr("List"))?;
            if group_items.is_empty() {
                return None;
            }

            let mut group = Vec::with_capacity(group_items.len());
            for item in group_items {
                let dimension = expr_as_positive_usize(item)?;
                if dimension > rank {
                    return None;
                }
                let zero_based = dimension - 1;
                if !seen.insert(zero_based) {
                    return None;
                }
                group.push(zero_based);
            }
            group.sort_unstable();
            groups.push(group);
        }

        return Some(ArrayReduceSpec::Grouped(groups));
    }

    let mut seen = std::collections::HashSet::new();
    let mut dimensions = Vec::with_capacity(items.len());
    for item in items {
        let dimension = expr_as_positive_usize(item)?;
        if dimension > rank {
            return None;
        }
        let zero_based = dimension - 1;
        if !seen.insert(zero_based) {
            return None;
        }
        dimensions.push(zero_based);
    }
    dimensions.sort_unstable();
    Some(ArrayReduceSpec::Grouped(vec![dimensions]))
}

fn parse_array_reduce_span(expr: &Expr, rank: usize) -> Option<Vec<usize>> {
    let items = call_items_with_head(expr, &system_expr("Span"))?;

    let (start, end) = match items {
        [end] => (1usize, parse_array_reduce_span_endpoint(end, rank, rank)?),
        [start, end] => (
            parse_array_reduce_span_endpoint(start, 1, rank)?,
            parse_array_reduce_span_endpoint(end, rank, rank)?,
        ),
        [start, end, step] if expr_as_positive_usize(step) == Some(1) => (
            parse_array_reduce_span_endpoint(start, 1, rank)?,
            parse_array_reduce_span_endpoint(end, rank, rank)?,
        ),
        _ => return None,
    };

    (start <= end).then(|| ((start - 1)..end).collect())
}

fn parse_array_reduce_span_endpoint(expr: &Expr, all_value: usize, rank: usize) -> Option<usize> {
    if is_system_symbol_name(expr, "All") {
        return Some(all_value);
    }

    let value = expr_as_positive_usize(expr)?;
    (value <= rank).then_some(value)
}

fn array_filter_window_shape(spec: &ArrayFilterSpec) -> Vec<usize> {
    match spec {
        ArrayFilterSpec::Radii(radii) => radii
            .iter()
            .map(|radius| radius.saturating_mul(2).saturating_add(1))
            .collect(),
        ArrayFilterSpec::Template(template) => template.shape.clone(),
    }
}

fn array_filter_window_center(spec: &ArrayFilterSpec) -> Vec<usize> {
    match spec {
        ArrayFilterSpec::Radii(radii) => radii.clone(),
        ArrayFilterSpec::Template(template) => template.center.clone(),
    }
}

fn array_filter_padding_leaf<'a>(
    padding: &'a ArrayFilterPadding,
    axis: usize,
    before: bool,
) -> &'a ArrayFilterPaddingLeaf {
    match padding {
        ArrayFilterPadding::Uniform(leaf) => leaf,
        ArrayFilterPadding::PerAxis(axis_paddings) => {
            let axis_padding = &axis_paddings[axis];
            if before {
                &axis_padding.before
            } else {
                &axis_padding.after
            }
        }
    }
}

fn array_filter_axis_trim(
    padding: &ArrayFilterPadding,
    axis: usize,
    window_len: usize,
    center: usize,
) -> (usize, usize) {
    let before_trim = if matches!(
        array_filter_padding_leaf(padding, axis, true),
        ArrayFilterPaddingLeaf::None
    ) {
        center
    } else {
        0
    };
    let after_trim = if matches!(
        array_filter_padding_leaf(padding, axis, false),
        ArrayFilterPaddingLeaf::None
    ) {
        window_len.saturating_sub(center + 1)
    } else {
        0
    };

    (before_trim, after_trim)
}

fn array_filter_template_includes(spec: &ArrayFilterSpec, window_index: &[usize]) -> bool {
    match spec {
        ArrayFilterSpec::Radii(_) => true,
        ArrayFilterSpec::Template(template) => {
            let offset = tensor_flat_offset(&template.shape, window_index)
                .expect("template positions should follow the template shape");
            template.mask[offset]
        }
    }
}

fn apply_array_filter(
    session: &mut Session,
    operator: &Expr,
    shape: &[usize],
    flat: &[Expr],
    spec: &ArrayFilterSpec,
    padding: &ArrayFilterPadding,
) -> Result<Expr, EvalError> {
    let list_head = system_expr("List");
    let window_shape = array_filter_window_shape(spec);
    let window_center = array_filter_window_center(spec);
    let axis_trims = window_shape
        .iter()
        .zip(window_center.iter())
        .enumerate()
        .map(|(axis, (window_len, center))| {
            array_filter_axis_trim(padding, axis, *window_len, *center)
        })
        .collect::<Vec<_>>();
    let output_shape = shape
        .iter()
        .zip(axis_trims.iter())
        .map(|(len, (before_trim, after_trim))| {
            if *len >= before_trim + after_trim {
                len - before_trim - after_trim
            } else {
                0
            }
        })
        .collect::<Vec<_>>();

    let mut outputs = Vec::new();
    for output_index in enumerate_indices(&output_shape) {
        let center = output_index
            .iter()
            .zip(axis_trims.iter())
            .map(|(index, (before_trim, _after_trim))| index + before_trim)
            .collect::<Vec<_>>();
        let mut window_index = Vec::with_capacity(shape.len());
        let block = build_array_filter_block(
            &list_head,
            flat,
            shape,
            &center,
            spec,
            &window_shape,
            &window_center,
            padding,
            0,
            &mut window_index,
        )
        .expect("validated array neighborhoods should rebuild");
        outputs.push(session.evaluate(Expr::call(operator.clone(), vec![block]))?);
    }

    Ok(rebuild_tensor_expr(&list_head, &output_shape, &outputs))
}

fn build_array_filter_block(
    head: &Expr,
    flat: &[Expr],
    shape: &[usize],
    center: &[usize],
    spec: &ArrayFilterSpec,
    window_shape: &[usize],
    window_center: &[usize],
    padding: &ArrayFilterPadding,
    axis: usize,
    window_index: &mut Vec<usize>,
) -> Option<Expr> {
    if axis == shape.len() {
        if !array_filter_template_includes(spec, window_index) {
            return Some(Expr::call(system_expr("Missing"), Vec::new()));
        }

        let source_index = window_index
            .iter()
            .enumerate()
            .map(|(window_axis, position)| {
                let center_position = isize::try_from(center[window_axis]).ok()?;
                let window_position = isize::try_from(*position).ok()?;
                let anchor = isize::try_from(window_center[window_axis]).ok()?;
                Some(center_position + window_position - anchor)
            })
            .collect::<Option<Vec<_>>>()?;
        return array_filter_scalar_at(flat, shape, &source_index, padding);
    }

    let mut items = Vec::with_capacity(window_shape[axis]);
    for position in 0..window_shape[axis] {
        window_index.push(position);
        items.push(build_array_filter_block(
            head,
            flat,
            shape,
            center,
            spec,
            window_shape,
            window_center,
            padding,
            axis + 1,
            window_index,
        )?);
        window_index.pop();
    }

    Some(Expr::call(head.clone(), items))
}

fn array_filter_scalar_at(
    flat: &[Expr],
    shape: &[usize],
    index: &[isize],
    padding: &ArrayFilterPadding,
) -> Option<Expr> {
    let mut normalized = Vec::with_capacity(index.len());
    array_filter_scalar_at_recursive(flat, shape, index, padding, 0, &mut normalized)
}

fn array_filter_scalar_at_recursive(
    flat: &[Expr],
    shape: &[usize],
    index: &[isize],
    padding: &ArrayFilterPadding,
    axis: usize,
    normalized: &mut Vec<usize>,
) -> Option<Expr> {
    if axis == index.len() {
        return tensor_scalar_at(flat, shape, normalized).cloned();
    }

    let len = *shape.get(axis)?;
    let len_isize = isize::try_from(len).ok()?;
    let raw_index = *index.get(axis)?;
    let in_bounds = (0..len_isize).contains(&raw_index);
    let leaf = array_filter_padding_leaf(padding, axis, raw_index < 0);

    let maybe_value = match leaf {
        ArrayFilterPaddingLeaf::None => {
            if !in_bounds {
                return None;
            }
            Some(usize::try_from(raw_index).ok()?)
        }
        ArrayFilterPaddingLeaf::Fixed => {
            if len == 0 {
                return None;
            }
            let clamped = raw_index.clamp(0, len_isize - 1);
            Some(usize::try_from(clamped).ok()?)
        }
        ArrayFilterPaddingLeaf::Periodic => {
            if len == 0 {
                return None;
            }
            let periodic = raw_index.rem_euclid(len_isize);
            Some(usize::try_from(periodic).ok()?)
        }
        ArrayFilterPaddingLeaf::Constant(expr) if !in_bounds => return Some(expr.clone()),
        ArrayFilterPaddingLeaf::Constant(_) => Some(usize::try_from(raw_index).ok()?),
        ArrayFilterPaddingLeaf::Cycle(values) if !in_bounds => {
            return Some(array_filter_cycle_value(values, raw_index, len_isize)?);
        }
        ArrayFilterPaddingLeaf::Cycle(_) => Some(usize::try_from(raw_index).ok()?),
    };

    normalized.push(maybe_value?);
    let value = array_filter_scalar_at_recursive(flat, shape, index, padding, axis + 1, normalized);
    normalized.pop();
    value
}

fn array_filter_cycle_value(values: &[Expr], raw_index: isize, len_isize: isize) -> Option<Expr> {
    if values.is_empty() {
        return None;
    }

    let offset = if raw_index < 0 {
        usize::try_from(-raw_index - 1).ok()?
    } else {
        usize::try_from(raw_index - len_isize).ok()?
    };

    Some(values[offset % values.len()].clone())
}

fn apply_array_reduce(
    session: &mut Session,
    operator: &Expr,
    shape: &[usize],
    flat: &[Expr],
    spec: &ArrayReduceSpec,
) -> Result<Expr, EvalError> {
    let list_head = system_expr("List");

    let ArrayReduceSpec::Grouped(groups) = spec else {
        let mapped = flat
            .iter()
            .map(|item| session.evaluate(Expr::call(operator.clone(), vec![item.clone()])))
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(rebuild_tensor_expr(&list_head, shape, &mapped));
    };

    let rank = shape.len();
    let reduced_dims = groups.iter().flatten().copied().collect::<Vec<_>>();
    let kept_dims = (0..rank)
        .filter(|axis| !reduced_dims.contains(axis))
        .collect::<Vec<_>>();
    let output_shape = kept_dims
        .iter()
        .map(|axis| shape[*axis])
        .collect::<Vec<_>>();
    let kept_indices = enumerate_indices(&output_shape);
    let grouped_shape = groups
        .iter()
        .map(|group| group.iter().map(|axis| shape[*axis]).product())
        .collect::<Vec<_>>();
    let grouped_indices = enumerate_indices(&grouped_shape);

    let mut outputs = Vec::with_capacity(kept_indices.len());
    for kept_index in kept_indices {
        let mut values = Vec::with_capacity(grouped_indices.len());
        for grouped_index in &grouped_indices {
            let full_index = array_reduce_full_index(
                rank,
                &kept_dims,
                &kept_index,
                groups,
                grouped_index,
                shape,
            );
            let value = tensor_scalar_at(flat, shape, &full_index)
                .expect("validated array reductions should resolve a scalar")
                .clone();
            values.push(value);
        }

        let operator_arg = rebuild_tensor_expr(&list_head, &grouped_shape, &values);
        outputs.push(session.evaluate(Expr::call(operator.clone(), vec![operator_arg]))?);
    }

    Ok(rebuild_tensor_expr(&list_head, &output_shape, &outputs))
}

fn array_reduce_full_index(
    rank: usize,
    kept_dims: &[usize],
    kept_index: &[usize],
    groups: &[Vec<usize>],
    grouped_index: &[usize],
    shape: &[usize],
) -> Vec<usize> {
    let mut full_index = vec![0usize; rank];
    for (position, axis) in kept_dims.iter().copied().enumerate() {
        full_index[axis] = kept_index[position];
    }

    for (group_position, group) in groups.iter().enumerate() {
        let group_shape = group.iter().map(|axis| shape[*axis]).collect::<Vec<_>>();
        let decoded = decode_grouped_axis_index(grouped_index[group_position], &group_shape);
        for (axis_position, axis) in group.iter().copied().enumerate() {
            full_index[axis] = decoded[axis_position];
        }
    }

    full_index
}

fn compute_inner(
    session: &mut Session,
    f: &Expr,
    left: &Expr,
    right: &Expr,
    g: &Expr,
) -> Result<Option<Expr>, EvalError> {
    let (
        Expr::Call {
            head: left_head, ..
        },
        Expr::Call {
            head: right_head, ..
        },
    ) = (left, right)
    else {
        return Ok(None);
    };

    if left_head.as_ref() != right_head.as_ref() {
        return Ok(None);
    }

    let head = left_head.as_ref().clone();
    let left_tensor = parse_tensor_node(left, &head);
    let right_tensor = parse_tensor_node(right, &head);
    let Some(left_shape) = tensor_shape(&left_tensor) else {
        return Ok(None);
    };
    let Some(right_shape) = tensor_shape(&right_tensor) else {
        return Ok(None);
    };
    let Some((&contract_dim, right_suffix)) = right_shape.split_first() else {
        return Ok(None);
    };
    let Some((&left_contract_dim, left_prefix)) = left_shape.split_last() else {
        return Ok(None);
    };

    if left_contract_dim != contract_dim {
        return Ok(None);
    }

    let left_flat = flatten_tensor_scalars(&left_tensor);
    let right_flat = flatten_tensor_scalars(&right_tensor);
    let output_shape = left_prefix
        .iter()
        .chain(right_suffix.iter())
        .copied()
        .collect::<Vec<_>>();
    let output_indices = enumerate_indices(&output_shape);
    let left_prefix_rank = left_prefix.len();
    let mut output_scalars = Vec::with_capacity(output_indices.len());

    for output_index in output_indices {
        let (left_index, right_index) = output_index.split_at(left_prefix_rank);
        let mut terms = Vec::with_capacity(contract_dim);

        for contract_index in 0..contract_dim {
            let mut lhs = left_index.to_vec();
            lhs.push(contract_index);
            let mut rhs = vec![contract_index];
            rhs.extend_from_slice(right_index);

            let Some(lhs_value) = tensor_scalar_at(&left_flat, &left_shape, &lhs) else {
                return Ok(None);
            };
            let Some(rhs_value) = tensor_scalar_at(&right_flat, &right_shape, &rhs) else {
                return Ok(None);
            };
            let term = session.evaluate(Expr::call(
                f.clone(),
                vec![lhs_value.clone(), rhs_value.clone()],
            ))?;
            terms.push(term);
        }

        let reduced = session.evaluate(Expr::call(g.clone(), terms))?;
        output_scalars.push(reduced);
    }

    Ok(Some(rebuild_tensor_expr(
        &head,
        &output_shape,
        &output_scalars,
    )))
}

fn parse_tensor_node(expr: &Expr, expected_head: &Expr) -> TensorNode {
    match expr {
        Expr::Call { head, args } if head.as_ref() == expected_head => TensorNode::Array(
            args.iter()
                .map(|arg| parse_tensor_node(arg, expected_head))
                .collect(),
        ),
        _ => TensorNode::Scalar(expr.clone()),
    }
}

fn tensor_shape(node: &TensorNode) -> Option<Vec<usize>> {
    match node {
        TensorNode::Scalar(_) => Some(Vec::new()),
        TensorNode::Array(items) => {
            let Some(first) = items.first() else {
                return Some(vec![0]);
            };
            let child_shape = tensor_shape(first)?;
            for item in items.iter().skip(1) {
                if tensor_shape(item)? != child_shape {
                    return None;
                }
            }

            let mut shape = vec![items.len()];
            shape.extend(child_shape);
            Some(shape)
        }
    }
}

fn flatten_tensor_scalars(node: &TensorNode) -> Vec<Expr> {
    let mut flat = Vec::new();
    flatten_tensor_scalars_into(node, &mut flat);
    flat
}

fn flatten_tensor_scalars_into(node: &TensorNode, flat: &mut Vec<Expr>) {
    match node {
        TensorNode::Scalar(expr) => flat.push(expr.clone()),
        TensorNode::Array(items) => {
            for item in items {
                flatten_tensor_scalars_into(item, flat);
            }
        }
    }
}

fn tensor_scalar_at<'a>(flat: &'a [Expr], shape: &[usize], index: &[usize]) -> Option<&'a Expr> {
    let offset = tensor_flat_offset(shape, index)?;
    flat.get(offset)
}

fn tensor_flat_offset(shape: &[usize], index: &[usize]) -> Option<usize> {
    if shape.len() != index.len() {
        return None;
    }

    if shape.is_empty() {
        return Some(0);
    }

    let mut offset = 0usize;
    let mut stride = shape.iter().product::<usize>();
    for (&axis_len, &axis_index) in shape.iter().zip(index.iter()) {
        if axis_index >= axis_len || axis_len == 0 {
            return None;
        }
        stride /= axis_len;
        offset += axis_index * stride;
    }

    Some(offset)
}

fn enumerate_indices(shape: &[usize]) -> Vec<Vec<usize>> {
    if shape.is_empty() {
        return vec![Vec::new()];
    }

    let suffixes = enumerate_indices(&shape[1..]);
    let mut indices = Vec::new();
    for first in 0..shape[0] {
        for suffix in &suffixes {
            let mut index = Vec::with_capacity(shape.len());
            index.push(first);
            index.extend_from_slice(suffix);
            indices.push(index);
        }
    }
    indices
}

fn decode_grouped_axis_index(mut index: usize, shape: &[usize]) -> Vec<usize> {
    if shape.is_empty() {
        return Vec::new();
    }

    let mut decoded = vec![0usize; shape.len()];
    for axis in (0..shape.len()).rev() {
        let len = shape[axis];
        if len == 0 {
            decoded[axis] = 0;
        } else {
            decoded[axis] = index % len;
            index /= len;
        }
    }
    decoded
}

fn rebuild_tensor_expr(head: &Expr, shape: &[usize], flat: &[Expr]) -> Expr {
    let mut offset = 0usize;
    rebuild_tensor_expr_from_offset(head, shape, flat, &mut offset)
}

fn rebuild_tensor_expr_from_offset(
    head: &Expr,
    shape: &[usize],
    flat: &[Expr],
    offset: &mut usize,
) -> Expr {
    if shape.is_empty() {
        let expr = flat
            .get(*offset)
            .cloned()
            .unwrap_or_else(|| Expr::call(head.clone(), Vec::new()));
        *offset += 1;
        return expr;
    }

    let items = (0..shape[0])
        .map(|_| rebuild_tensor_expr_from_offset(head, &shape[1..], flat, offset))
        .collect();
    Expr::call(head.clone(), items)
}
