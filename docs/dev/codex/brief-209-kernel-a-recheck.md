# Brief 209 — Re-verification of the unit-8 audit's disputed safety claims

Re-check brief. Your brief-208 audit (`docs/dev/usability-pass/kernel-a-findings.md`)
is otherwise strong work — the count reconciliation, the whole-registry
`Protected` coverage table, the per-file fix map, and the hard-coded-hold
trace all reproduced exactly at the gate. Two of its safety claims did NOT
reproduce, and the evidence sections they cite are absent from the document.
Your run may have been cut short (the report was truncated at the very end),
so this is a genuine re-check, not a reprimand: it is entirely possible that
you found something real and never got to write it down.

**Both outcomes are a passing deliverable.** "NOT REPRODUCED — the claim was
wrong" is exactly as good as "REPRODUCED — here is the trigger." What is NOT
acceptable is a third option: manufacturing a contrived scenario to make the
original claim look right. Do not do that. Follow the evidence.

## The facts (verbatim, no interpretation)

**Fact 1 — two evidence sections are missing.** The findings table cites
`Evidence E15` and `Evidence E16` in twelve rows (the eleven compound
assignments and one `Set` row). The Evidence Appendix in the document you
produced ends at **E14**. E15 and E16 do not exist in the file.

**Fact 2 — what those rows claim.** That the compound-assignment shared final
store has a check-to-store window: the guard runs on the initial target, the
derived update is evaluated afterwards, and `store_held_update`
(`crates/rum-runtime/src/runtime/definitions/definition_updates.rs:352-412`)
then writes without rechecking — so an upvalue that adds protection *during
update evaluation* would leave the target visibly `Protected` while the write
still lands. The `Set` row makes the parallel claim for the association-lvalue
path (`crates/rum-runtime/src/runtime/kernel_data/association_data.rs:366-484`):
selectors evaluate after an initial guard, and the final store does not
recheck.

**Fact 3 — the gate could not reproduce any of it.** Three probes, run on the
current tree, verbatim:

```
$ printf '' | cargo run -q -p rum-cli -- --eval 'ClearAll[t, g]; t = 1; g[x_] := (Protect[t]; 98); AddTo[t, g[0]]; Print[{t, Attributes[t]}]'
{1, {Protected}}
! AddTo::wrsym: Symbol t is Protected.
! AddTo::wrsym: Symbol t is Protected.

$ printf '' | cargo run -q -p rum-cli -- --eval 'ClearAll[L, f2]; L = {1}; f2[] := (Protect[L]; 7); AppendTo[L, f2[]]; Print[{L, Attributes[L]}]'
{{1}, {Protected}}
! AppendTo::wrsym: Symbol L is Protected.
! AppendTo::wrsym: Symbol L is Protected.

$ printf '' | cargo run -q -p rum-cli -- --eval 'ClearAll[a, k]; a = <|"x" -> 1|>; Protect[a]; a["x"] = 99; Print[{a, Attributes[a]}]'
{<|"x" -> 1|>, {Protected}}
! Set::wrsym: Symbol a is Protected.
```

In each case the write was refused and the value is unchanged.

**Fact 4 — your own E4 says the same thing.** The genuine evidence section E4,
which you did write, is titled "All Compound Assignments Fail Closed" and its
transcript shows all eleven forms refusing a protected target.

Note that the gate's `AddTo`/`AppendTo` probes protect the target from a
DOWNVALUE evaluated as part of the update, not from an upvalue on the target
symbol. If the ordering hole is real but only reachable through a different
mechanism, that mechanism is precisely what this brief is asking you to
produce.

## Your task

1. **Settle the compound-assignment claim.** Read the actual path —
   target normalization / guard (`definition_updates.rs:244-304`), update
   evaluation, and the final store (`:352-412`). Determine whether a
   check-to-store window exists at all. If it does, construct a probe that
   TRIGGERS it and paste the verbatim transcript: the exact `--eval` line, the
   output, and the value/attributes afterwards showing a write that landed on a
   visibly protected symbol. If it does not, say NOT REPRODUCED and explain
   what the code actually does (where the recheck happens, or why the window is
   not reachable).
2. **Settle the association-lvalue claim** the same way
   (`association_data.rs:366-484`, the `Set` path in
   `definitions/definitions.rs`).
3. **Settle `SetOptions` on a protected symbol.** Your E5 grouped it with
   `SetAttributes` as a leak. `SetAttributes` REPRODUCED at the gate
   (`Protect[s]` then `SetAttributes[s, Listable]` succeeds → `{Listable,
   Protected}`, no message — this one is real and is already queued for the
   fix). The `SetOptions` half did NOT get verified: the gate probe used an
   unregistered option name and was deflected by `SetOptions::optnf` before
   reaching any protection check. Use a head that HAS registered options, and
   determine whether `SetOptions` on a protected symbol is refused or lands.
4. **Audit your own safety rows.** Every row in the findings table with
   `verdict = gap` and `axis = safety` must be backed by either a verbatim
   probe transcript in the appendix or a `file:line` citation that actually
   supports it. List any row that is not — including rows whose cited evidence
   section exists but does not in fact show the claimed behavior. Report the
   list even if it is empty.

## Deliverable

`docs/dev/usability-pass/kernel-a-recheck.md`, structured as:

- **Verdict table**: one row per disputed claim — `Claim | REPRODUCED /
  NOT REPRODUCED / PARTIAL | Evidence (probe or file:line) | What the code
  actually does`.
- **Transcripts**: verbatim, for every verdict. A REPRODUCED verdict needs a
  copy-pasteable `--eval` line that a reader can run to see the hole. A NOT
  REPRODUCED verdict needs the probe that shows fail-closed plus the code
  citation for where the check happens.
- **Unsupported-rows list** (task 4).
- **Post-mortem, one short paragraph**: if the original claims do not hold, say
  what you can reconstruct about how they got written. If you cannot
  reconstruct it, write "cannot reconstruct" — do not invent a story.

## Scope

- MAY create: `docs/dev/usability-pass/kernel-a-recheck.md` — the ONLY file in
  the diff.
- MUST NOT touch: any crate code, tests, `docs/dev/usability-pass/kernel-a-findings.md`
  (the gate already annotated it in place; leave it alone), the tracker,
  `TODO.md`, or anything else. This is a read-and-probe brief.

## Hard rules

- No code changes, no `git commit`, no branch, no push, no network.
- **Never cite an evidence section that does not exist.** Every citation must
  resolve to a section you actually wrote in this document, or to a real
  `file:line`. This is the specific failure being re-checked; do not repeat it.
- **NOT REPRODUCED is a passing result.** Do not manufacture an artificial
  scenario to rescue the original claim. If the hole needs a contortion no
  script would ever write, say so explicitly and call it out as such rather
  than presenting it as a finding.
- Probe budget: ~15 evals. Feed stdin (`printf '' | …`). Keep the report short
  and evidence-first — verdicts and transcripts, not prose.
- Stop-loss: if a probe misbehaves twice the same way, STOP and report it.

## Report format (your final message — keep it SHORT)

1. The verdict table (4 rows: compound assignment, association lvalue,
   SetOptions, unsupported-rows count).
2. For each REPRODUCED verdict: the trigger probe, verbatim.
3. The unsupported-rows list.
4. Stop-loss / open questions.
