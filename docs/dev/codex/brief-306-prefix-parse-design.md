# brief-306 — design a prefix-parse path for linear script loading

## Goal

Design — do not implement — a way for `split_script_into_forms` to run in
linear time **with its observable form splitting preserved by construction**,
rather than by a property that has to hold. Deliver a reviewable plan plus
drafted implementation briefs.

**Scope: NONE — no source change.** Do not edit any source file. Reading,
grepping, and throwaway probes with the release `rum-cli` are in scope; leave no
probe files in the repository. Do not run the microbench scripts.

## Settled background — do not re-derive or re-litigate

brief-305 proposed skipping `parse_expr` while an incremental delimiter/string
balance stayed open. **That design is dead.** Its load-bearing property —
"balance open ⟹ the parse error is an `is_incomplete()` class" — was disproved,
and the director has independently confirmed the counter-examples:

| input | actual parse result |
|---|---|
| `f[1 ,, 2` | `expected expression, found ,` — hard error |
| `f[g[1 )` | `expected ,, found )` — hard error |
| `f[1}` | `expected ,, found }` — hard error |
| `f[; x` | `expected expression, found ;` — hard error |
| `f[1 2` | `incomplete input: expected ]` |
| `1 2` (top level) | **`Ok`** — it is `Times[1, 2]`, value 2 |

The reason the early spot-checks passed: juxtaposition (`a b` == `Times[a, b]`,
`parser/front.rs:203-215`) rescues adjacent operands so the parse survives to
EOF and reports `expected ]`. Real grammar conflicts die *before* EOF. brief-305
classified the counter-examples into three kinds; only two are mirrorable in a
lexical scanner, and the third — grammar conflicts inside an open delimiter,
where `f[1 ,, 2` differs from the legal `f[1 , 2` by one comma — is not a
lexically enumerable set. Do not attempt to revive that approach.

The user has chosen the remaining sound direction: **move the splitting decision
into the parser**, so that each form is parsed once and the boundaries follow
from parsing rather than from a heuristic oracle.

## The current mechanism, and the two costs

`crates/rum-runtime/src/runtime/context_and_load_helpers.rs:100-149` appends one
line at a time to `current`, calls `parse_expr(&current)` on the whole
accumulation, and branches four ways:

| branch | action |
|---|---|
| `Ok(_)` | push `current` as a form, clear |
| `Err(EmptyInput)` | clear, push nothing |
| `Err(e) if e.is_incomplete()` | keep accumulating |
| `Err(_)` (hard error) | push `current` as a form anyway, clear |

The tail block after the loop repeats this, except an incomplete tail becomes an
error.

Two separate costs, and the plan should address both explicitly:

1. **The quadratic**: an unterminated form re-parses its whole prefix per line.
   Measured by the director: `data = {` + N element lines + `}` takes
   0.026 / 0.071 / 0.273 / 1.096 / 4.478 s at N = 500 / 1000 / 2000 / 4000 /
   8000, and **7.75 s at 10 000** — versus **0.01 s** for the same data on one
   line. A control of 10 000 independent one-line statements is linear.
2. **The double parse**: every form is parsed a second time, because
   `split_script_into_forms` returns `Vec<String>` and
   `eval_text_with_terminated_handling` (`session/session_eval.rs:184-200`)
   calls `parse_expr(input)` again before lowering. Even a perfectly linear
   splitter leaves this 2× in place.

## Questions the plan must answer, each with `path:line` evidence

1. **The parser entry point.** What is the smallest honest way to expose "parse
   one expression starting here, and report how much input it consumed"? Today
   `parse_expr` insists on consuming everything and reports `TrailingInput`
   otherwise (`parser/front.rs:50-62` area — confirm). Does a prefix entry mean
   a new public function, a parameter, or a different return type? What does it
   do about trivia, comments, and the shebang line?
2. **Boundary semantics, branch by branch.** For each of the four branches
   above, state exactly what the new mechanism produces and why it is the same.
   The hard-error branch is the crux: today a hard error still emits the
   accumulated text as a form so one bad line cannot poison the rest — pinned by
   `runtime/tests/context_and_loader.rs:609`. How does a prefix parser decide
   where that form ends? Note the pinned case `"1 2\n3"` → `["1 2", "3"]` goes
   through the **`Ok`** branch, not the hard-error branch.
3. **Lexing.** One lex of the whole source, or per form? If one, how does the
   token stream survive a form boundary and a lexical error? What does
   `lex_with_trivia` (`rum-syntax/src/lexer/mod.rs`) do on first error, and does
   that constrain the answer?
4. **The double parse.** Can the splitter hand the already-built `SyntaxExpr`
   (or `ParsedSource`) to evaluation instead of a `String`? Inventory the ripple:
   every caller of `split_script_into_forms` and of
   `eval_text_with_terminated_handling`, and what a non-`String` form type would
   cost them. If this is better done as a separate step, say so and say why.
5. **Spans and messages.** Are error spans today relative to the form or to the
   file? Whatever they are, the plan must keep them — quote the current
   behaviour from a probe, not from reading alone.
6. **Existing facilities.** How do `document.rs`'s `ParsedSource` /
   `parse_with_diagnostics` (`rum-syntax/src/document.rs:326`, exported at
   `lib.rs:18-21`) relate to this? Is a multi-expression parse already partly
   built there?
7. **Ripple beyond the splitter.** Every other caller of `parse_expr` across the
   workspace: which are affected by a new entry point, which are not.
8. **`rum-repl`.** Its `is_complete` re-parses the buffer the same way
   (`rum-repl/src/lib.rs:113-129`). Its N is bounded by what a human types, so
   it is **not** a target — but say whether the new entry point would let it
   share the mechanism for free, and leave it out of the first implementation
   brief regardless.
9. **What could still change observably**, if anything, and how a test would
   catch it. If the answer is genuinely "nothing, by construction", show why.

## Deliverable

1. The design, answering all nine questions with evidence.
2. A **step plan**: what lands in which order, each step independently gateable
   and small enough to review in one sitting. Say which step first delivers the
   measured speedup — a plan whose benefit only appears at the last step is
   worse than one that pays off early.
3. **Drafted implementation briefs**, one per step, in `TEMPLATE.md` shape.
   These are raw material for the director to rewrite, not to dispatch as-is.
4. Risks and open questions, including anything you would want the director to
   decide before step 1.

## Pitfalls (read twice)

- **Do not design against a property that has to be proved by sampling.** That
  is exactly what killed brief-305. Prefer a mechanism whose correctness follows
  from construction, and say plainly which parts do not.
- **Do not quietly widen the observable contract.** Form boundaries, error
  messages, spans, and the poisoning-resistance behaviour are all user-visible.
- **Do not over-generalize a correct observation.** Attribute each finding
  exactly as far as the evidence reaches.
- **Findings age.** Confirm structures exist in the tree as it stands now — the
  parser was modified yesterday (three caps split out of `MAX_PARSE_DEPTH`, `;`
  chains made iterative in parser and lowering).
- **Stop and report** rather than inventing a workaround if a question turns out
  to have no clean answer. A well-evidenced "this direction has this cost" is a
  valid deliverable.

## Hard rules

- No source changes. No commits, branches, pushes, or git config changes.
- Touch nothing outside this repo; no network. Commands: file reads, grep/find,
  `python3`, and the release `rum-cli` for throwaway probes. Probes must feed
  stdin from `/dev/null` — the CLI waits for input otherwise.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses.
- Every claim carries `path:line`. A claim you could not confirm is reported as
  unconfirmed, not asserted.

## Report format (your final message)

1. Method — what you read and probed.
2. The design, question by question.
3. The step plan, with the payoff point identified.
4. The drafted implementation briefs.
5. Risks and open questions for the director.
