use super::*;

fn eval_to_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

#[test]
fn list_plumbing_attributes_match_wk_anchors() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "Attributes[Differences]"),
        "List[NonThreadable, Protected]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Attributes[ConstantArray]"),
        "List[Protected]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Attributes[PadLeft]"),
        "List[Protected]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Attributes[PadRight]"),
        "List[Protected]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Attributes[Subsets]"),
        "List[Protected]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Attributes[Tuples]"),
        "List[Protected]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Attributes[Partition]"),
        "List[Protected]"
    );
}

#[test]
fn differences_covers_successive_and_ordered_plain_list_subset() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "Differences[{1, 4, 9, 16}]"),
        "List[3, 5, 7]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Differences[{1, 4, 9, 16}, 2]"),
        "List[2, 2]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Differences[{1, 4, 9, 16}, 0]"),
        "List[1, 4, 9, 16]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Differences[x]"),
        "Differences[x]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Differences[f[1, 4, 9]]"),
        "Differences[f[1, 4, 9]]"
    );

    let invalid_order = session.eval_text("Differences[{1, 4}, x]").unwrap();
    assert_eq!(
        invalid_order.value.to_string(),
        "Differences[List[1, 4], x]"
    );
    assert!(
        invalid_order
            .messages
            .iter()
            .any(|message| message.symbol == "Differences" && message.tag == "ilsmn")
    );
}

#[test]
fn constant_array_covers_scalar_and_multidimensional_integer_shapes() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "ConstantArray[0, 3]"),
        "List[0, 0, 0]"
    );
    assert_eq!(
        eval_to_string(&mut session, "ConstantArray[x, {2, 2}]"),
        "List[List[x, x], List[x, x]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "ConstantArray[x, 0]"),
        "List[]"
    );
    assert_eq!(
        eval_to_string(&mut session, "ConstantArray[x, {2, 0}]"),
        "List[List[], List[]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "n = 0; {ConstantArray[n = n + 1, 3], n}"),
        "List[List[1, 1, 1], 1]"
    );
    assert_eq!(
        eval_to_string(&mut session, "ConstantArray[x, y]"),
        "ConstantArray[x, y]"
    );
}

#[test]
fn pad_left_covers_padding_truncation_cyclic_and_normal_heads() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "PadLeft[{1, 2}, 5]"),
        "List[0, 0, 0, 1, 2]"
    );
    assert_eq!(
        eval_to_string(&mut session, "PadLeft[{1, 2}, 5, x]"),
        "List[x, x, x, 1, 2]"
    );
    assert_eq!(
        eval_to_string(&mut session, "PadLeft[{1, 2, 3, 4, 5}, 3]"),
        "List[3, 4, 5]"
    );
    assert_eq!(
        eval_to_string(&mut session, "PadLeft[{1, 2}, 5, {7, 8}]"),
        "List[8, 7, 8, 1, 2]"
    );
    assert_eq!(
        eval_to_string(&mut session, "PadLeft[f[1, 2], 5]"),
        "f[0, 0, 0, 1, 2]"
    );
    assert_eq!(
        eval_to_string(&mut session, "PadLeft[{{1}, {2}}, {2, 3}]"),
        "List[List[0, 0, 1], List[0, 0, 2]]"
    );

    let atomic = session.eval_text("PadLeft[x, 5]").unwrap();
    assert_eq!(atomic.value.to_string(), "PadLeft[x, 5]");
    assert!(
        atomic
            .messages
            .iter()
            .any(|message| message.symbol == "PadLeft" && message.tag == "normal")
    );

    let invalid_len = session.eval_text("PadLeft[{1, 2}, x]").unwrap();
    assert_eq!(invalid_len.value.to_string(), "PadLeft[List[1, 2], x]");
    assert!(
        invalid_len
            .messages
            .iter()
            .any(|message| message.symbol == "PadLeft" && message.tag == "ilsm")
    );
}

#[test]
fn pad_right_covers_padding_truncation_cyclic_and_normal_heads() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "PadRight[{1, 2}, 5]"),
        "List[1, 2, 0, 0, 0]"
    );
    assert_eq!(
        eval_to_string(&mut session, "PadRight[{1, 2}, 5, x]"),
        "List[1, 2, x, x, x]"
    );
    assert_eq!(
        eval_to_string(&mut session, "PadRight[{1, 2, 3, 4, 5}, 3]"),
        "List[1, 2, 3]"
    );
    assert_eq!(
        eval_to_string(&mut session, "PadRight[{1, 2}, 5, {7, 8}]"),
        "List[1, 2, 7, 8, 7]"
    );
    assert_eq!(
        eval_to_string(&mut session, "PadRight[f[1, 2], 5]"),
        "f[1, 2, 0, 0, 0]"
    );
    assert_eq!(
        eval_to_string(&mut session, "PadRight[{{1}, {2}}, {2, 3}, x]"),
        "List[List[1, x, x], List[2, x, x]]"
    );

    let atomic = session.eval_text("PadRight[x, 5]").unwrap();
    assert_eq!(atomic.value.to_string(), "PadRight[x, 5]");
    assert!(
        atomic
            .messages
            .iter()
            .any(|message| message.symbol == "PadRight" && message.tag == "normal")
    );

    let invalid_len = session.eval_text("PadRight[{1, 2}, x]").unwrap();
    assert_eq!(invalid_len.value.to_string(), "PadRight[List[1, 2], x]");
    assert!(
        invalid_len
            .messages
            .iter()
            .any(|message| message.symbol == "PadRight" && message.tag == "ilsm")
    );
}

#[test]
fn subsets_covers_wk_anchored_size_specs_and_ordering() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "Subsets[{1, 2, 3}]"),
        "List[List[], List[1], List[2], List[3], List[1, 2], List[1, 3], List[2, 3], List[1, 2, 3]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Subsets[{1, 2, 3}, 2]"),
        "List[List[], List[1], List[2], List[3], List[1, 2], List[1, 3], List[2, 3]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Subsets[{1, 2, 3}, {2}]"),
        "List[List[1, 2], List[1, 3], List[2, 3]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Subsets[{1, 2, 3}, {1, 2}]"),
        "List[List[1], List[2], List[3], List[1, 2], List[1, 3], List[2, 3]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Subsets[f[1, 2], {1}]"),
        "List[f[1], f[2]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Subsets[{1, 2, 3}, All]"),
        "List[List[], List[1], List[2], List[3], List[1, 2], List[1, 3], List[2, 3], List[1, 2, 3]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Subsets[{1, 2, 3}, Infinity]"),
        "List[List[], List[1], List[2], List[3], List[1, 2], List[1, 3], List[2, 3], List[1, 2, 3]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Subsets[{1, 2, 3}, {1, 3, 2}]"),
        "List[List[1], List[2], List[3], List[1, 2, 3]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Subsets[{1, 2, 3}, {1, 3, -1}]"),
        "List[List[1, 2, 3], List[1, 2], List[1, 3], List[2, 3], List[1], List[2], List[3]]"
    );
}

#[test]
fn tuples_covers_wk_anchored_repetition_and_cartesian_forms() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "Tuples[{1, 2}, 2]"),
        "List[List[1, 1], List[1, 2], List[2, 1], List[2, 2]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Tuples[{{1, 2}, {3, 4}}]"),
        "List[List[1, 3], List[1, 4], List[2, 3], List[2, 4]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Tuples[{1, 2}, 0]"),
        "List[List[]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Tuples[{{1, 2}, {}}]"),
        "List[]"
    );
}

#[test]
fn partition_covers_wk_anchored_block_and_offset_forms() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "Partition[{1, 2, 3, 4, 5}, 2]"),
        "List[List[1, 2], List[3, 4]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Partition[{1, 2, 3, 4, 5}, 2, 1]"),
        "List[List[1, 2], List[2, 3], List[3, 4], List[4, 5]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Partition[{1, 2, 3}, 5]"),
        "List[]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Partition[f[1, 2, 3], 2, 1]"),
        "f[f[1, 2], f[2, 3]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Partition[{a, b, c, d}, 2, 2, {1, 1}, z]"),
        "List[List[a, b], List[c, d]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Partition[{a, b, c, d, e}, 2, 2, {1, 1}, {}]"),
        "List[List[a, b], List[c, d], List[e]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Partition[{a, b, c, d}, 2, 2, {1, 1}]"),
        "List[List[a, b], List[c, d]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Partition[{a, b, c, d}, 2, 2, {2, 1}, z]"),
        "List[List[z, a], List[b, c], List[d, z]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Partition[{a, b, c, d}, 3, 1, {1, 2}, z]"),
        "List[List[a, b, c], List[b, c, d]]"
    );
}

#[test]
fn partition_operator_form_rewrites_to_data_first_call() {
    let mut session = core_session();

    let applied = session.eval_text("Partition[2][{a, b, c, d}]").unwrap();
    assert_eq!(applied.value.to_string(), "List[List[a, b], List[c, d]]");
    assert!(applied.messages.is_empty());

    let inert = session.eval_text("Partition[2]").unwrap();
    assert_eq!(inert.value.to_string(), "Partition[2]");
    assert!(inert.messages.is_empty());
}

#[test]
fn combinatorial_list_ops_symbolically_passthrough_invalid_boundaries() {
    let mut session = core_session();

    let subsets_atom = session.eval_text("Subsets[x]").unwrap();
    assert_eq!(subsets_atom.value.to_string(), "Subsets[x]");
    assert!(
        subsets_atom
            .messages
            .iter()
            .any(|message| message.symbol == "Subsets" && message.tag == "normal")
    );

    let subsets_bad_size = session.eval_text("Subsets[{1, 2}, x]").unwrap();
    assert_eq!(subsets_bad_size.value.to_string(), "Subsets[List[1, 2], x]");
    assert!(
        subsets_bad_size
            .messages
            .iter()
            .any(|message| message.symbol == "Subsets" && message.tag == "nninfseq")
    );

    let tuples_atom = session.eval_text("Tuples[x, 2]").unwrap();
    assert_eq!(tuples_atom.value.to_string(), "Tuples[x, 2]");
    assert!(
        tuples_atom
            .messages
            .iter()
            .any(|message| message.symbol == "Tuples" && message.tag == "normal")
    );

    let tuples_bad_length = session.eval_text("Tuples[{1, 2}, x]").unwrap();
    assert_eq!(tuples_bad_length.value.to_string(), "Tuples[List[1, 2], x]");
    assert!(
        tuples_bad_length
            .messages
            .iter()
            .any(|message| message.symbol == "Tuples" && message.tag == "ilsmn")
    );

    let partition_atom = session.eval_text("Partition[x, 2]").unwrap();
    assert_eq!(partition_atom.value.to_string(), "Partition[x, 2]");
    assert!(
        partition_atom
            .messages
            .iter()
            .any(|message| message.symbol == "Partition" && message.tag == "npart")
    );

    let partition_bad_size = session.eval_text("Partition[{1, 2}, x]").unwrap();
    assert_eq!(
        partition_bad_size.value.to_string(),
        "Partition[List[1, 2], x]"
    );
    assert!(
        partition_bad_size
            .messages
            .iter()
            .any(|message| message.symbol == "Partition" && message.tag == "ilsmp")
    );

    let partition_bad_alignment = session
        .eval_text("Partition[{1, 2}, 2, 1, {0, 1}]")
        .unwrap();
    assert_eq!(
        partition_bad_alignment.value.to_string(),
        "Partition[List[1, 2], 2, 1, List[0, 1]]"
    );
    assert!(
        partition_bad_alignment
            .messages
            .iter()
            .any(|message| message.symbol == "Partition" && message.tag == "align")
    );
}
