"""Small rum-owned semantic workloads; no external engine is an oracle."""

SIZES = (64, 256, 1024)
LABELS = ("table-pairs", "append-part", "rewrite-wide")


def program(label, size, control=False):
    if label == "table-pairs":
        setup = "gHotResult = Null;"
        action = f"gHotResult = Table[{{j, j + 1}}, {{j, 1, {size}}}];"
        expected = "{" + ",".join(f"{{{j},{j + 1}}}" for j in range(1, size + 1)) + "}"
        check = f"SameQ[gHotResult, {expected}]"
    elif label == "append-part":
        setup = "gHotOuter = Table[{}, {1000}];"
        action = f"Do[AppendTo[gHotOuter[[1]], i], {{i, 1, {size}}}];"
        check = (
            "Length[gHotOuter] == 1000 && "
            f"SameQ[gHotOuter[[1]], Range[{size}]] && "
            "SameQ[Rest[gHotOuter], Table[{}, {999}]]"
        )
    elif label == "rewrite-wide":
        setup = f"gHotInput = Range[{size}]; gHotResult = Null;"
        action = "gHotResult = Replace[gHotInput, x_Integer :> x + 1, {1}];"
        check = (
            f"SameQ[gHotInput, Range[{size}]] && "
            f"SameQ[gHotResult, Range[2, {size + 1}]]"
        )
    else:
        raise ValueError(label)
    mode = "control" if control else "active"
    return (
        "(* Diagnostic input, not an uninstrumented performance benchmark. *)\n"
        + setup + "\n"
        + 'RumProfileSnapshot["before"];\n'
        + "RumProfileReset[];\n"
        + ("Null;" if control else action) + "\n"
        + 'RumProfileSnapshot["after"];\n'
        + ("" if control else f"If[{check}, Null, Exit[2]];\n")
        + f'Print["RUM_HOTSPOT_PASS\\t{label}\\t{size}\\t{mode}"];\n'
    )
