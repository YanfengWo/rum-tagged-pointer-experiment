# Bytecode VM Plan (task-2 of the reopened perf line)

Status: proposed, 2026-07-01. This is the design/feasibility plan for the second
(larger) task of the reopened perf line. Task-1 (hash-consing / value-memo) was
re-measured this session and judged MARGINAL (briefs 067/068: a ~7% live descent
prize behind a hash-consing enabler that taxes 100% of constructions). The user
chose to pivot to task-2. This plan exists so the VM line is INCREMENTAL and
MEASURED like the representation-rewrite arc, not a multi-week big-bang on faith.

## Goal

Cut the distributed per-call dispatch/eval constant — the hard plateau that is the
whole remaining rum/WolframKernel gap on the shared core subset (the Pair `~34x` is
this same constant compounded over the descent and amplified ~1.7x by the WL
control combinators; see `docs/representation-rewrite-plan.md` + PROGRESS
`2026-06-21`). Stretch: beat WK on the supported core subset, validated by
measurement.

## The target (2026-06-21 decomposition — re-measure on v8 in brief-069)

The per-call constant is DISTRIBUTED with no single hotspot (this is why every
single-hotspot theory was refuted: matcher 5%, inline stamp, node alloc, module
frame). On the faithful failing-dispatch profile:

| phase | share | mult/call | nature |
| --- | ---: | ---: | --- |
| `evaluate.total` (fixed-point loop) | 21.1% | 2 | eval-STRATEGY (re-evaluate to fixed point) |
| `evaluate.stability_check` | 7.4% | 4 | eval-STRATEGY (prove stable) |
| `evaluate.once_unstable` | 7.1% | 2 | eval-STRATEGY |
| `prepare_arguments.total` | 6.8% | 1 | dispatch GLUE (VM-addressable) |
| `evaluate_call.dispatch` | 5.9% | 1 | dispatch GLUE (VM-addressable) |
| `down_values` (the actual match) | 5.3% | 1 | IRREDUCIBLE symbolic work |
| long distributed tail | … | | mixed |

These numbers are pre-Layer-A and pre-32-byte-`Expr`; brief-069 re-measures them on
the current v8 binary before any commitment.

## Feasibility crux (read before designing anything)

A bytecode VM is a decisive win for a NUMERIC interpreter because the operation
sequence is STATIC — compiling the tree to bytecode removes tree-walking overhead
for a fixed program. A SYMBOLIC rewriter is different: every call may rewrite,
attributes gate arg evaluation, dispatch is by pattern/down-value lookup, and the
"program" changes as terms rewrite. So the symbolic DISPATCH + MATCH + ARG-EVAL
work is IRREDUCIBLE regardless of representation — a naive "compile the expression
tree to bytecode" would just re-encode "evaluate this call" = the same dispatch.

Where a bytecode VM genuinely helps a symbolic evaluator:
1. **Compiled rule RHS / function bodies.** The WL-level combinators mma-hol leans
   on (`ORELSEC`/`REPEATC`/`TRYCONV`, curried down/sub-value functions) are
   re-interpreted (re-walked, re-dispatched) on every application — the ~1.7x Pair
   amplification. Compiling those bodies to a linear instruction stream once, then
   running the stream, is the highest-value VM-specific lever.
2. **Compiled dispatch glue.** `prepare_arguments` + `evaluate_call.dispatch`
   (~13% combined) is per-call plumbing that a compiled call site could shortcut.
3. **A tighter fixed-point / stability state machine.** The eval-strategy chunk
   (~35%: fixed-point loop 2x/call + 4 stability checks) — BUT this is an
   ALGORITHM change (evaluate-once + prove-stable-cheaply), addressable WITHOUT a
   VM. It is the biggest chunk and likely the cheapest per unit win.

## The addressable-prize split (what brief-069 must produce)

The honest go/no-go for a VM is not "is there a per-call constant" (yes) but "what
does a VM buy OVER the cheaper eval-strategy fixes." So brief-069 partitions the
re-measured constant into three buckets:

- **(i) eval-STRATEGY, no VM needed** — the fixed-point-loop multiplicity +
  redundant stability checks (~35% in the old numbers). If most of this is
  recoverable by evaluating once and proving stability cheaply, it is the
  first, cheapest line — and it SHRINKS the residual a VM would target.
- **(ii) VM-SPECIFIC** — compiled rule bodies (the 1.7x combinator amplification)
  + compiled dispatch glue (~13%). This is the VM's actual prize.
- **(iii) IRREDUCIBLE** — dispatch lookup + pattern match + arg eval (the real
  symbolic work; ~5% match + the necessary arg evaluation). A VM cannot cut this.

Decision after 069: if (ii) is large and (i) is small, a VM is justified. If (i)
dominates, take the strategy fixes FIRST (cheaper, no VM), then re-size (ii).

## Principles (inherited from the representation-rewrite arc)

- **Incremental, never big-bang.** Every step compiles, keeps `cargo test
  --workspace` green, is independently committable and valuable. No half-built VM
  across a commit.
- **Measured every step, downstream-validated on Pair.** A microbench win need not
  translate to Pair (A3 didn't). Every step is gated on BOTH the primitive tier
  AND the bounded Pair descent (`runtime-profile-basic-only-pair4`) — the Pair
  translation is the real acceptance signal.
- **Estimate the win from the profile before building.** brief-069's split sizes
  each lever's ceiling; do not build a lever whose ceiling the profile says is
  small.
- **No new `unsafe` in rum-core/rum-runtime; hot-path changes need a microbench
  A/B** (the standing TEMPLATE/AGENTS rules).
- **No `Expr`/serde/mma-hol changes** as a side effect.

## Incremental path (subject to brief-069's data)

1. **069 (diagnostic):** re-decompose the per-call constant on v8; produce the
   three-way addressable-prize split + each lever's ceiling. NO code.
2. **Then the cheapest-per-win lever first** — most likely the eval-strategy
   fixed-point/stability multiplicity (bucket i), as bounded eval-core briefs,
   each Pair-validated. This is NOT VM work; it banks the biggest chunk cheaply and
   shrinks the VM's residual target.
3. **Then, only if bucket (ii) still justifies it:** a compiled-rule-body VM —
   start with compiling the hot WL control combinators / down-value RHS bodies to
   a linear op stream, behind a feature flag, measured on Pair. This is the
   multi-week core; it gets its own sub-plan once 069 + the strategy fixes have
   sized its real residual prize.

## Risk register

- **A VM may not translate to Pair** (the distributed constant has no single
  hotspot; a compiled dispatch that helps the microbench may wash out on Pair like
  A3 did). Gate every step on Pair, not just the microbench.
- **Big-bang risk.** A partial VM is slower than either end state. Feature-flag and
  keep the tree-walker as the default until a step is proven net-positive on Pair.
- **The biggest lever may not be the VM.** If 069 shows bucket (i) dominates, the
  VM is lower-ROI than the strategy fixes; do those first and re-decide. Do not
  build a VM to satisfy the label if the profile points elsewhere.
- **Irreducibility.** Do not promise cuts to bucket (iii); a VM cannot remove
  symbolic dispatch/match. Over-promising here is the classic mistake.

## First brief

`docs/dev/codex/brief-069-per-call-constant-decomposition.md` — the diagnostic
that produces the addressable-prize split. Nothing gets built until its data is in.
