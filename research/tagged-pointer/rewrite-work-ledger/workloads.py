"""Generate deterministic rewrite diagnostics with full rum-owned checks.

These programs contain no timing operations.  The work ledger is scoped by the
instrumented Replace implementation, so setup and post-result validation are
outside its root.  Expected lists are generated explicitly here rather than
using another rewrite or arithmetic operation as a result oracle.
"""

CASES = (
    "increment",
    "increment-reversed",
    "identity",
    "miss",
    "opaque-body",
    "flat-nested",
)
SIZES = (8, 64, 256, 1024)


def _list(items):
    return "{" + ",".join(items) + "}"


def program(label: str, size: int) -> str:
    """Return one .rum program checking the entire result and both aliases."""
    if label not in CASES:
        raise ValueError(f"unknown rewrite diagnostic case: {label}")
    if not isinstance(size, int) or isinstance(size, bool) or size <= 0:
        raise ValueError("size must be a positive integer")

    original = _list(str(index) for index in range(1, size + 1))
    setup = ""
    if label == "increment":
        rule = "phaseX_Integer :> phaseX + 1"
        expected = _list(str(index + 1) for index in range(1, size + 1))
    elif label == "increment-reversed":
        rule = "phaseX_Integer :> 1 + phaseX"
        expected = _list(str(index + 1) for index in range(1, size + 1))
    elif label == "identity":
        rule = "phaseX_Integer :> phaseX"
        expected = original
    elif label == "miss":
        rule = "phaseMissing -> 0"
        expected = original
    elif label == "opaque-body":
        rule = "phaseX_Integer :> phaseF[phaseX]"
        expected = _list(f"phaseF[{index}]" for index in range(1, size + 1))
    else:
        assert label == "flat-nested"
        setup = "SetAttributes[phaseF, Flat];\n"
        rule = "phaseX_Integer :> phaseF[phaseF[phaseX], 1]"
        expected = _list(f"phaseF[{index},1]" for index in range(1, size + 1))

    checks = (
        ("result", f"SameQ[phaseResult, {expected}]"),
        ("input", f"SameQ[phaseInput, {original}]"),
        ("alias", f"SameQ[phaseAlias, {original}]"),
    )
    validation = "".join(
        f'If[{check}, Null, Print["WORK_LEDGER_FAIL\\t{label}\\t{size}\\t{name}"]; Exit[2]];\n'
        for name, check in checks
    )
    return (
        "(* Deterministic work accounting; no elapsed-time measurements. *)\n"
        + setup
        + f"phaseInput = Range[{size}];\n"
        + "phaseAlias = phaseInput;\n"
        + f"phaseResult = Replace[phaseInput, {rule}, {{1}}];\n"
        + validation
        + f'Print["WORK_LEDGER_PASS\\t{label}\\t{size}"];\n'
    )
