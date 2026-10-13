# Native wide symbolic Sort

This is an isolated stdlib integration: a default Sort of at least 64 entirely
symbolic operands uses a specialized temporary vector. Mixed/small operands and
custom comparators retain the original path. Main Expr layout is unchanged and
there is no recursive conversion. Three representations keep complete identities
and stable original qualified-name order, including Global/System display ties.

| Representation | Bytes per temporary slot | Safety mechanism |
| --- | ---: | --- |
| Symbol24 | 24 | Original safe Symbol |
| Safe16 | 16 | Permanent text reference + full optional identity |
| Tagged16 | 16 | Private tagged authentic text pointer + numeric word |

The tagged slot is a new static-text owner. It owns no Arc allocation; only its
private text dereference and Send/Sync require unsafe. Its alignment/provenance,
permanent immutable text assumptions and semantic refinement are described in
[SEMANTICS.md](SEMANTICS.md). The previous general owner argument does not
automatically prove this owner. Core and runtime patches contain safe Rust only.

## Full language timing result

Ratios below are median elapsed time relative to the original; below 1 is faster.
Brackets contain the three process ratios, each based on the suite's minimum of
nine repetitions. Complete 112-label results and controls are in
[summary.json](../evidence/native-dispatch/summary.json).

| Symbol Sort width | Symbol24 | Safe16 | Tagged16 |
| --- | --- | --- | --- |
| 4000 | 0.920 [0.911, 0.920, 0.995] | 1.053 [1.053, 1.116, 0.919] | 1.062 [0.913, 1.122, 1.062] |
| 16000 | 0.936 [0.929, 0.936, 0.944] | 0.954 [0.942, 1.034, 0.954] | 0.947 [0.920, 1.015, 0.947] |

The strongest stable signal is about **6.4% less time** for Symbol24 at width
16000. The smaller slots do not beat this safe control, and the 4000 case is
unstable. Unrelated labels also regress: Safe16's StringTrim labels by about
10–16%; Tagged16's wide association DeleteCases median by about 13%. No mode
passes a broad no-regression gate. The causes of cross-path regressions have
not been isolated; code generation effects are a hypothesis, not a finding.

Pairwise checks compare all representations with original qualified-name
comparison across 72 symbols (5184 pairs), then validate full stable output,
admission/fallback and thread transfer. Two native tests and both strict Miri
models pass per mode. All integrations pass workspace and complete language
acceptance tests. See [the shared experiment entry](../native-dispatch/README.md)
for reproduction and timing limitations.

Keep Symbol24 as a narrow candidate for independent repetition and regression
diagnosis. The data does not warrant a whole-expression rewrite or claim of
overall speedup, C parity, portable pointer tagging, or complete soundness proof.
