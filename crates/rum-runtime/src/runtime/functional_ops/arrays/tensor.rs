use super::*;

pub(super) fn apply_array_reduce(
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

pub(super) fn array_reduce_full_index(
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

pub(super) fn compute_inner(
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

pub(super) fn parse_tensor_node(expr: &Expr, expected_head: &Expr) -> TensorNode {
    match expr {
        Expr::Call { head, args } if head.as_ref() == expected_head => TensorNode::Array(
            args.iter()
                .map(|arg| parse_tensor_node(arg, expected_head))
                .collect(),
        ),
        _ => TensorNode::Scalar(expr.clone()),
    }
}

pub(super) fn tensor_shape(node: &TensorNode) -> Option<Vec<usize>> {
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

pub(super) fn flatten_tensor_scalars(node: &TensorNode) -> Vec<Expr> {
    let mut flat = Vec::new();
    flatten_tensor_scalars_into(node, &mut flat);
    flat
}

pub(super) fn flatten_tensor_scalars_into(node: &TensorNode, flat: &mut Vec<Expr>) {
    match node {
        TensorNode::Scalar(expr) => flat.push(expr.clone()),
        TensorNode::Array(items) => {
            for item in items {
                flatten_tensor_scalars_into(item, flat);
            }
        }
    }
}

pub(super) fn tensor_scalar_at<'a>(flat: &'a [Expr], shape: &[usize], index: &[usize]) -> Option<&'a Expr> {
    let offset = tensor_flat_offset(shape, index)?;
    flat.get(offset)
}

pub(super) fn tensor_flat_offset(shape: &[usize], index: &[usize]) -> Option<usize> {
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

pub(super) fn enumerate_indices(shape: &[usize]) -> Vec<Vec<usize>> {
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

pub(super) fn decode_grouped_axis_index(mut index: usize, shape: &[usize]) -> Vec<usize> {
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

pub(super) fn rebuild_tensor_expr(head: &Expr, shape: &[usize], flat: &[Expr]) -> Expr {
    let mut offset = 0usize;
    rebuild_tensor_expr_from_offset(head, shape, flat, &mut offset)
}

pub(super) fn rebuild_tensor_expr_from_offset(
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
