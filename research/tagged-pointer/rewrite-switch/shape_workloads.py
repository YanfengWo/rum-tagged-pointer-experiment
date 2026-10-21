"""Predeclared positive/negative-level controls for redundant depth scans."""
from workloads import wrapper

DEFINITIONS = [(f"chain-{depth}", "f[" * depth + "1" + "]" * depth)
               for depth in (16, 32, 64)]
for height in (5, 7, 9):
    tree = "1"
    for _ in range(height):
        tree = f"b[{tree},{tree}]"
    DEFINITIONS.append((f"binary-{height}", tree))
LABELS = [f"shape-{name}-{levels}" for name, _ in DEFINITIONS
          for levels in ("positive", "negative")]


def source(timed=True):
    text = wrapper(timed).replace(
        "resultAlias = result;", "resultAlias = {result}; result = resultAlias;"
    ).replace("SameQ[resultAlias, expected]", "SameQ[resultAlias, {expected}]")
    for name, expr in DEFINITIONS:
        text += f"shapeInput = {expr}; shapeAlias = shapeInput;\n"
        for label, levels in (("positive", "{0, Infinity}"),
                              ("negative", "{-128, -1}")):
            text += (f'flatMeasure["shape-{name}-{label}", '
                     f'Replace[shapeInput, shapeAbsent -> 0, {levels}], shapeInput];\n')
        text += f"If[SameQ[shapeInput,{expr}] && SameQ[shapeAlias,{expr}],Null,Exit[2]];\n"
    return text + 'Print["ISO_SHAPE_PASS"];\n'
