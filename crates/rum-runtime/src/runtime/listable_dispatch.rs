use super::*;

enum ThreadArgSelection {
    All,
    Sparse(Vec<usize>),
    Dense(Vec<bool>),
}

impl ThreadArgSelection {
    const DENSE_THRESHOLD: usize = 16;

    fn new(spec: &ThreadArgSpec, arg_count: usize) -> Option<Self> {
        if arg_count == 0 {
            return None;
        }
        if matches!(spec, ThreadArgSpec::All) {
            return Some(Self::All);
        }

        let indices = resolve_thread_arg_indices(spec, arg_count);
        if indices.is_empty() {
            return None;
        }
        if indices.len() <= Self::DENSE_THRESHOLD {
            return Some(Self::Sparse(indices));
        }

        let mut mask = vec![false; arg_count];
        for index in indices {
            mask[index] = true;
        }
        Some(Self::Dense(mask))
    }

    fn contains(&self, index: usize) -> bool {
        match self {
            Self::All => true,
            Self::Sparse(indices) => indices.contains(&index),
            Self::Dense(mask) => mask[index],
        }
    }
}

impl Session {
    pub(super) fn dispatch_listable_stage(
        &mut self,
        head: &Expr,
        args: &[Expr],
        attributes: AttributeSet,
    ) -> Result<ThreadingDispatch, EvalError> {
        if !attributes.contains(Attribute::Listable) {
            return Ok(ThreadingDispatch::NotApplicable);
        }

        self.thread_call_args(
            head,
            args,
            &system_expr("List"),
            &system_expr("List"),
            &ThreadArgSpec::All,
        )
    }

    /// Threads selected call arguments over a common head and wrapper.
    pub fn thread_call_args(
        &mut self,
        head: &Expr,
        args: &[Expr],
        thread_head: &Expr,
        wrapper_head: &Expr,
        arg_spec: &ThreadArgSpec,
    ) -> Result<ThreadingDispatch, EvalError> {
        let Some(selected_args) = ThreadArgSelection::new(arg_spec, args.len()) else {
            return Ok(ThreadingDispatch::NotApplicable);
        };

        let mut target_len = None;
        let mut thread_lengths = Vec::new();

        for (index, arg) in args.iter().enumerate() {
            if !selected_args.contains(index) {
                continue;
            }

            let Some(items) = call_items_with_head(arg, thread_head) else {
                continue;
            };

            thread_lengths.push(items.len());

            match target_len {
                Some(expected_len) if expected_len != items.len() => {
                    self.emit_thread_length_message(head, thread_head, &thread_lengths);
                    return Ok(ThreadingDispatch::Mismatch);
                }
                Some(_) => {}
                None => target_len = Some(items.len()),
            }
        }

        let Some(target_len) = target_len else {
            return Ok(ThreadingDispatch::NotApplicable);
        };

        let threaded = (0..target_len)
            .map(|index| {
                Expr::call(
                    head.clone(),
                    args.iter()
                        .enumerate()
                        .map(
                            |(arg_index, arg)| match call_items_with_head(arg, thread_head) {
                                Some(items) if selected_args.contains(arg_index) => {
                                    items[index].clone()
                                }
                                None => arg.clone(),
                                Some(_) => arg.clone(),
                            },
                        )
                        .collect(),
                )
            })
            .collect::<Vec<_>>();

        Ok(ThreadingDispatch::Threaded(Expr::call(
            wrapper_head.clone(),
            threaded,
        )))
    }

    /// Emits the standard threading length diagnostic for a failed thread.
    pub fn emit_thread_length_message(
        &mut self,
        call_head: &Expr,
        thread_head: &Expr,
        thread_lengths: &[usize],
    ) {
        let detail = if is_system_symbol_name(thread_head, "List") {
            format!(
                "cannot thread `{}` over list arguments of lengths {}",
                call_head,
                format_list_lengths(thread_lengths)
            )
        } else {
            format!(
                "cannot thread `{}` over arguments with head `{}` of lengths {}",
                call_head,
                thread_head,
                format_list_lengths(thread_lengths)
            )
        };

        self.push_message("Thread", "tdlen", detail);
    }
}
