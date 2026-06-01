use super::*;

pub(in crate::runtime::structure_ops) fn select_positioned_parts_for_span(
    expr: &Expr,
    spec: PartSpanSpec,
) -> Option<Vec<(isize, Expr)>> {
    match expr {
        Expr::Atom(_) => {
            resolve_span_components(spec, 0)?;
            Some(Vec::new())
        }
        Expr::Call { args, .. } => {
            let components = resolve_span_components(spec, args.len())?;
            Some(
                components
                    .into_iter()
                    .map(|component| {
                        let index = normalize_structural_arg_position(component, args.len())?;
                        Some((component, args[index].clone()))
                    })
                    .collect::<Option<Vec<_>>>()?,
            )
        }
    }
}

pub(super) fn resolve_span_components(spec: PartSpanSpec, len: usize) -> Option<Vec<isize>> {
    if spec.step == 0 {
        return None;
    }

    let len = isize::try_from(len).ok()?;
    if spec.step > 0 {
        let start = resolve_positive_span_start(spec.start, len)?;
        let end = resolve_positive_span_end(spec.end, len)?;
        resolve_ordered_span_components(start, end, spec.step)
    } else {
        let start = resolve_negative_span_start(spec.start, len)?;
        let end = resolve_negative_span_end(spec.end, len)?;
        resolve_ordered_span_components(start, end, spec.step)
    }
}

fn resolve_ordered_span_components(start: isize, end: isize, step: isize) -> Option<Vec<isize>> {
    let mut current = start;
    let mut components = Vec::new();

    if step > 0 {
        if start > end {
            return (start == end + 1).then_some(Vec::new());
        }
        while current <= end {
            components.push(current);
            current += step;
        }
    } else {
        if start < end {
            return (start + 1 == end).then_some(Vec::new());
        }
        while current >= end {
            components.push(current);
            current += step;
        }
    }

    Some(components)
}

fn resolve_positive_span_start(endpoint: SpanEndpoint, len: isize) -> Option<isize> {
    match endpoint {
        SpanEndpoint::All => Some(1),
        SpanEndpoint::UpTo(limit) => Some(resolve_upto_start_boundary(limit, len)),
        SpanEndpoint::Index(0) => Some(len + 1),
        SpanEndpoint::Index(index) if index > 0 => (index <= len + 1).then_some(index),
        SpanEndpoint::Index(index) => {
            let normalized = len + index + 1;
            (1..=len).contains(&normalized).then_some(normalized)
        }
    }
}

fn resolve_positive_span_end(endpoint: SpanEndpoint, len: isize) -> Option<isize> {
    match endpoint {
        SpanEndpoint::All => Some(len),
        SpanEndpoint::UpTo(limit) => Some(resolve_upto_end_boundary(limit, len)),
        SpanEndpoint::Index(0) => Some(0),
        SpanEndpoint::Index(index) if index > 0 => (index <= len).then_some(index),
        SpanEndpoint::Index(index) => {
            let normalized = len + index + 1;
            (0..=len).contains(&normalized).then_some(normalized)
        }
    }
}

fn resolve_negative_span_start(endpoint: SpanEndpoint, len: isize) -> Option<isize> {
    if len == 0 {
        return None;
    }

    match endpoint {
        SpanEndpoint::All => Some(1),
        SpanEndpoint::UpTo(Some(0)) => None,
        SpanEndpoint::UpTo(limit) => Some(resolve_upto_end_boundary(limit, len)),
        SpanEndpoint::Index(0) => None,
        SpanEndpoint::Index(index) if index > 0 => (index <= len).then_some(index),
        SpanEndpoint::Index(index) => {
            let normalized = len + index + 1;
            (1..=len).contains(&normalized).then_some(normalized)
        }
    }
}

fn resolve_negative_span_end(endpoint: SpanEndpoint, len: isize) -> Option<isize> {
    if len == 0 {
        return None;
    }

    match endpoint {
        SpanEndpoint::All => Some(len),
        SpanEndpoint::UpTo(Some(0)) => None,
        SpanEndpoint::UpTo(limit) => Some(resolve_upto_end_boundary(limit, len)),
        SpanEndpoint::Index(0) => None,
        SpanEndpoint::Index(index) if index > 0 => (index <= len + 1).then_some(index),
        SpanEndpoint::Index(index) => {
            let normalized = len + index + 1;
            (1..=len).contains(&normalized).then_some(normalized)
        }
    }
}

fn resolve_upto_start_boundary(limit: Option<isize>, len: isize) -> isize {
    match limit {
        None => len,
        Some(0) => len + 1,
        Some(limit) if len == 0 => limit,
        Some(limit) => limit.min(len),
    }
}

fn resolve_upto_end_boundary(limit: Option<isize>, len: isize) -> isize {
    match limit {
        None => len,
        Some(limit) => limit.min(len),
    }
}
