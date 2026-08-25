# brief-327 — capability enforcement completeness audit

## Goal

rum's stated position is that a restricted session can evaluate untrusted code:
capabilities gate every effect, and resource ceilings bound every cost. That
claim is only as strong as its weakest enforcement point. Audit whether it
holds — against behavior, not against declarations.

This is a **diagnostic survey**. Produce a ranked list of enforcement gaps with
`path:line` attribution and reproductions, plus drafted implementation briefs.
**Scope: NONE — no source change.**

Reading, grepping, and throwaway probes with the release `rum-cli` are in scope;
leave no probe files in the repository. Probes must feed stdin from `/dev/null`.

## Why this audit is worth doing carefully

A restricted-session guarantee is the project's differentiator, so a gap in it
matters more than a slow path. Two such gaps have already been found and fixed,
which is the reason to look again rather than to assume:

- The loader reached file reads through `BeginPackage`/`DeclarePackage` without
  passing the `FileRead` check; it was fixed by gating at the `eval_get` /
  `eval_needs` seam.
- `Pause[10^100]` terminated the host process through a `Duration` overflow.

Both were found by auditing *behavior against the declared contract*, not by
reading declarations.

## The model as it stands

Eight capabilities (`crates/rum-runtime/src/runtime/capabilities.rs:2-13`):
`Pure`, `FileRead`, `FileWrite`, `Clock`, `Random`, `Diagnostic`, `Process`,
`Interactive`. A policy either is unrestricted or holds a bitmask; the check is
`is_unrestricted() || allows(capability)`
(`builtins_context_loader.rs:63-68`).

Two boundaries are **not** part of this model and must not be confused with it:

- `Protected` is a mistake-guard and a WL-conformance feature, **not** a sandbox
  boundary. Do not report `Protected` bypasses as capability gaps.
- Per-request isolation and rollback are the host-side story, separate from
  capability gating.

## The question, stated precisely

For each capability, the claim is: **no evaluation reachable under a policy that
withholds it can produce the effect it names.** Audit that claim. A gap is any
path that reaches a gated effect without passing an enforcement point, and a
builtin whose declared capability differs from what its behavior needs.

**Choose your own method.** Build whatever understanding of the system you need
first — trace a request end to end, map the layers, whatever gets you oriented —
then go deep where that understanding says the risk is. The requirements below
are about what a finding must carry, not about how you arrive at it.

Two angles are offered because they have found real gaps here before, not as a
prescribed procedure:

- **Backwards from the effect.** Enumerate where the runtime and stdlib actually
  perform gated effects — file reads and writes, directory listing, clock reads,
  randomness, process control, host input, diagnostic internals — including the
  provider seams rather than only the builtins, then walk back through every
  path that reaches each one. Auditing forward from the gates tends to find only
  the paths someone already thought to gate.
- **Where layers meet.** Both gaps found here so far lived at a seam: the loader
  reaching a file read through a package declaration, and an argument whose
  magnitude terminated the host process. Shared helpers with one gated and one
  ungated caller, effects triggered while evaluating an argument, snapshot and
  restoration paths, and loader or codec paths that call back into evaluation
  are the shapes that label review misses.

## Also in scope

- **Resource ceilings**: whether each ceiling actually bounds the work it names,
  whether an operation can grow past it in a place the ceiling does not measure,
  and whether hitting one leaves the session usable.
- **The host API surface** (`rum-host`): entry points reachable by an embedder
  are a different trust class than script-reachable ones. Say which class each
  finding belongs to; both are worth reporting, and confusing them overstates or
  understates severity.
- **Named policy profiles** (`hermetic`, `scripting`): whether each profile's
  pins match what its documentation promises.

## What each finding must carry

1. **The effect** and the capability that should gate it, with `path:line`.
2. **The path that reaches it ungated**, with each hop.
3. **A reproduction** under a restricted policy, verbatim, or an explicit
   statement that you could not construct one — an unreproduced path is a lead,
   not a finding, and must be labelled.
4. **Trust class**: script-reachable, or host-API-reachable only.
5. **Why the existing enforcement point does not cover it.**
6. **Blast radius**: what an evaluation could actually accomplish through it.

Rank by (severity × reachability). Script-reachable gaps outrank host-API ones.

**Report the negative results too.** Effects you traced and found properly gated
are the more valuable half of this audit: they are what lets the project state
the guarantee with evidence rather than hope.

## Deliverable

1. An inventory of the gated effects you covered, and the gate for each.
2. The ranked gap table.
3. The negative list — effects confirmed gated.
4. A batching plan and drafted implementation briefs in `TEMPLATE.md` shape.
5. Open questions.

## Handling of findings

Report everything to the director in your final message. **Do not** add
reproductions of unfixed gaps to any file in the repository — this repository is
public, and the director decides what is published and when. Drafted briefs
should describe the fix and the affected code, not carry a working exploit.

## Pitfalls (read twice)

- **A gate you can see is rarely the one that is missing.** Whatever method you
  use, make sure it can surface a path nobody thought to gate.
- **A declaration is not evidence.** Confirm behavior with a probe under a
  restricted policy wherever you can construct one.
- **Do not report `Protected`, or ordinary error paths, as capability gaps.**
- **Findings age.** Over a hundred commits landed recently, several in loader,
  codec and evaluation paths; confirm against the current tree.
- **Do not fix anything.** A wrong diagnosis costs one dispatch; a wrong fix in
  this area costs the guarantee's credibility.

## Hard rules

- No source changes. No commits, branches, pushes, or git config changes.
- Touch nothing outside this repo; no network. Commands: file reads, grep/find,
  `python3`, the release `rum-cli`, and `cargo` only if a probe needs a build.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses.
- **Leave no `TODO` or `FIXME` markers.** This repository has none by policy.
- Every claim carries `path:line` or a reproduction. A claim you could not
  confirm is reported as unconfirmed.

## Report format (your final message)

1. Method — how you oriented yourself, what you covered, and what you
   deliberately skipped.
2. The gap table, ranked.
3. The negative list — effects confirmed gated, with the gate for each.
4. The batching plan and drafted briefs.
5. Open questions for the director.
