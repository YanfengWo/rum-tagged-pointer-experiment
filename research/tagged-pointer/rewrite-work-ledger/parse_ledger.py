"""Strict parser for completed synchronous work ledgers, with explicit units."""


def parse(path):
    rows, keys = [], set()
    state = "new"
    for line in path.read_text(encoding="utf-8").splitlines():
        parts = line.split("\t")
        assert parts[:2] in (["rum-work", "root"], ["rum-work", "row"]), line
        if parts[1] == "root":
            assert len(parts) == 4 and parts[2] == "1"
            if parts[3] == "begin":
                assert state == "new"
                state = "active"
            else:
                assert parts[3] == "end" and state == "active"
                state = "ended"
        else:
            assert len(parts) == 8 and parts[2] == "1" and state == "active", line
            _, _, _, phase, kind, encoded, amount, calls = parts
            detail = bytes.fromhex(encoded).decode("utf-8")
            amount, calls = int(amount), int(calls)
            assert (phase == "replace" or phase.startswith("replace/")) and amount >= 0 and calls > 0
            key = (phase, kind, detail, amount)
            assert key not in keys, key
            keys.add(key)
            rows.append({"phase": phase, "kind": kind, "detail": detail, "amount": amount,
                         "calls": calls, "units": amount * calls})
    assert state == "ended" and rows
    return rows


def total(rows, kind, detail=None, phase_prefix=None, amount=None, units=False):
    return sum(row["units" if units else "calls"] for row in rows
               if row["kind"] == kind and (detail is None or row["detail"] == detail)
               and (phase_prefix is None or row["phase"].startswith(phase_prefix))
               and (amount is None or row["amount"] == amount))


def describe(rows, size):
    return {
        "traversal_entries": total(rows, "replace-node"),
        "depth_helper_entries": total(rows, "replace-depth-node"),
        "matcher_entries": total(rows, "matcher"),
        "binding_snapshot_entries": total(rows, "binding-snapshot", "capture"),
        "snapshot_named_items": total(rows, "binding-snapshot", "names", units=True),
        "rhs_instantiations": total(rows, "rhs-substitution", "request"),
        "plus_builtin_calls": total(rows, "plus-builtin", "calls"),
        "plus_integer_success": total(rows, "plus-fast-integer", "success"),
        "plus_normalization_rebuilds": total(rows, "normalization-rebuild", "Plus"),
        "plus_sort_comparisons": total(rows, "orderless-sort-comparison", "Plus"),
        "plus_flatten_calls": total(rows, "flatten-call", "Plus"),
        "plus_flatten_nested": total(rows, "flatten-nested-calls", "Plus"),
        "plus_fresh_flatten_outputs": total(rows, "flatten-output-slots", "Plus"),
        "plus_fresh_flatten_capacity_slots": total(rows, "flatten-output-capacity-slots", "Plus", units=True),
        "plus_reused_flatten_outputs": total(rows, "flatten-reuse-slots", "Plus"),
        "plus_reused_flatten_capacity_slots": total(rows, "flatten-reuse-capacity-slots", "Plus", units=True),
        "into_vec_copy_events": total(rows, "into-vec-copy"),
        "into_vec_copy_slots": total(rows, "into-vec-copy", units=True),
        "wide_copy_events": total(rows, "into-vec-copy", amount=size),
        "two_slot_copy_events": total(rows, "into-vec-copy", amount=2),
        "all_fresh_flatten_outputs": total(rows, "flatten-output-slots"),
        "all_reused_flatten_outputs": total(rows, "flatten-reuse-slots"),
    }
