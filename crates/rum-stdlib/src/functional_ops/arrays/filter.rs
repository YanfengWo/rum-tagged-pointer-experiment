use super::{spec::*, tensor::*, *};

pub(super) fn apply_array_filter(
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

    let mut operator_args = Vec::new();
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
        operator_args.push(block);
    }

    resume_tensor_callback_from_index(
        session,
        TensorCallbackEvalState {
            operator: operator.clone(),
            output_head: list_head,
            output_shape,
            operator_args,
        },
        0,
        Vec::new(),
    )
}

pub(super) fn build_array_filter_block(
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

pub(super) fn array_filter_scalar_at(
    flat: &[Expr],
    shape: &[usize],
    index: &[isize],
    padding: &ArrayFilterPadding,
) -> Option<Expr> {
    let mut normalized = Vec::with_capacity(index.len());
    array_filter_scalar_at_recursive(flat, shape, index, padding, 0, &mut normalized)
}

pub(super) fn array_filter_scalar_at_recursive(
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

pub(super) fn array_filter_cycle_value(
    values: &[Expr],
    raw_index: isize,
    len_isize: isize,
) -> Option<Expr> {
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
