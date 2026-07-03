# Brief 099 — kernel_api v1 + pilot: de-glob the `logic` family

## Goal

Stage-2 implementation, step 1 (`docs/crate-rebalance-plan.md`). Create the
explicit kernel API module that builtin families will compile against, and prove
it end-to-end on the cheapest family: after this brief, `runtime/kernel_api.rs`
exists as the ONLY public path to the promoted kernel items, and the `logic`
family compiles with NO `use super::*` — only explicit imports (kernel_api +
`rum_core`/std). Zero behavior change; this is a visibility/imports-only refactor
with no codegen-relevant delta, so no microbench gate (no hot-path change — the
standing A/B rule does not trigger).

## Context pointers

- Read `AGENTS.md`, `docs/crate-rebalance-plan.md` (Stage 2), and the audit
  artifact `docs/dev/kernel-api-audit-2026-07-03.md` (brief-098) — the promotion
  list and per-family tables there are the data this brief executes on.
- Key facts from the audit:
  - Promotion candidates: the fan-out ≥2 items (24 with ≥3 families + 16 with 2;
    the full per-item table with defining modules is in the artifact).
  - `logic` family = `runtime/logic.rs` + `runtime/logic/*.rs`; 9 distinct free
    items, zero same-family locals, zero other-family coupling — the cheapest
    pilot.
  - Items are currently `pub(in crate::runtime)` inside PRIVATE `mod`s of
    `runtime`; the shared namespace is the glob block in `runtime/mod.rs`.
- Promotion mechanics (Rust): `pub use` cannot re-export an item less visible
  than the re-export, so promoting = flip the item's own declaration to `pub` at
  its definition site, then `pub use` it in `kernel_api`. Because the defining
  `mod`s stay private, the item remains unreachable from outside the crate
  EXCEPT via `kernel_api` — that is the intended single-door design.

## The design (follow it; deviations go in the report)

1. NEW `crates/rum-runtime/src/runtime/kernel_api.rs`: a `pub mod` whose body is
   `pub use` re-exports, grouped by defining module with a `//` section header
   per group. Header doc comment stating: this is the kernel↔stdlib contract;
   families import ONLY from here (plus `rum_core`/`rum_syntax`/std); it grows
   one family conversion at a time; items here become semver-relevant when the
   crate split ships.
2. Contents of kernel_api v1 = the union of:
   (a) every fan-out ≥2 item from the audit artifact's promotion table
   (~40 items — copy the exact list from the artifact);
   (b) whatever ELSE the `logic` pilot's compile errors demand (its 9 items
   likely overlap (a); any fan-out-1 kernel-defined item logic needs is ALSO
   promoted into kernel_api — the API grows by demand, never blanket).
   `Session`, `EvalError`, and other already-`pub` types need no visibility
   change — re-export them in kernel_api anyway so families have one import
   path.
3. For each promoted item: change `pub(in crate::runtime)` → `pub` at the
   definition ONLY (no body/signature changes; a one-line `///` doc comment on
   the item is welcome where its purpose isn't obvious from the name, optional
   otherwise). List every touched definition site in the report.
4. Register `pub mod kernel_api;` (or `mod` + `pub use self::kernel_api;`) in
   `runtime/mod.rs` so `rum_runtime::kernel_api::…` resolves from outside the
   crate. Do NOT add kernel_api to the internal glob block.
5. Pilot conversion: in `runtime/logic.rs` and every file under
   `runtime/logic/`, delete `use super::*;` and replace with explicit imports:
   `use crate::runtime::kernel_api::{…};` plus direct `use rum_core::…` /
   `use rum_syntax::…` / std as needed. Inherent `Session` methods
   (`self.evaluate(…)` etc.) need no import — only free items count. Iterate on
   compile errors until green.
6. Grep gates after: `grep -rn "use super::\*" crates/rum-runtime/src/runtime/logic.rs crates/rum-runtime/src/runtime/logic/`
   → zero hits; `grep -rn "kernel_api" crates/rum-runtime/src/runtime/logic*` →
   present.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/kernel_api.rs` (NEW)
  - `crates/rum-runtime/src/runtime/mod.rs` (the `mod`/`pub use` registration
    lines only)
  - `crates/rum-runtime/src/runtime/logic.rs` and
    `crates/rum-runtime/src/runtime/logic/**` (imports only — no logic changes)
  - The DEFINITION SITES of promoted items across `runtime/` — visibility
    keyword + optional one-line doc comment ONLY, nothing else on those lines'
    items; every touched file listed in the report.
- Files/areas you MUST NOT touch:
  - any OTHER family's files (arithmetic, associations, strings, … keep their
    `use super::*` for now — one family per brief);
  - any behavior: no function body, signature, ordering, or attribute change
    anywhere;
  - `crates/rum-core/**`, serde shapes, other crates (`rum-host`/`rum-cli`/… must
    compile unchanged);
  - anything under any `mma-hol` clone; `AGENTS.md`, `PROGRESS.md`, `TODO.md`,
    `docs/**` except appending your own report; `.git`, `Cargo.toml`/`Cargo.lock`.

## Deliverable

- `kernel_api.rs` (grouped, documented) + promoted definitions + de-globbed
  `logic` family, in the working tree (no commit).
- The report's promoted-item list: item → defining module → what demanded it
  ((a) fan-out list vs (b) logic pilot).
- Verbatim gate outputs (below).

## rum / Rust pitfalls (read twice)

1. **Promote by demand, never blanket.** If you find yourself flipping a
   visibility that neither the fan-out table nor a logic compile error demanded,
   stop — that item does not belong in v1.
2. **`pub` inside a private mod leaks nothing on its own** — the only new public
   surface is what `kernel_api` re-exports plus its module path. Verify with
   `cargo doc -p rum-runtime --no-deps` only if quick; do not chase docs
   perfection.
3. **Do not reorder or "clean up" the glob block** in `runtime/mod.rs`; other
   families still depend on it verbatim.
4. **Import style in logic files**: one `use crate::runtime::kernel_api::{…}`
   per file listing exactly what that file uses — no wildcard
   (`kernel_api::*` would recreate the glob problem).
5. Zero behavior change: any `cargo test --workspace` difference is a defect.
6. Stop-loss: if the logic pilot's demanded item count explodes past ~25 (audit
   says 9), the audit's model is wrong somewhere — STOP and report the demanded
   list instead of promoting through it.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
grep -rn "use super::\*" crates/rum-runtime/src/runtime/logic.rs crates/rum-runtime/src/runtime/logic/   # zero hits
grep -c "pub use" crates/rum-runtime/src/runtime/kernel_api.rs                                            # report the count
```

Paste all outputs verbatim (workspace test totals per crate; the empty grep; the
pub-use count). No microbench: imports/visibility only — assert in your report
that no function body or signature changed (your `git diff --stat` supports it).

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave all changes in
  the working tree; Claude runs the authoritative gate and commits.
- Do NOT modify files outside Scope; if you believe you must, STOP and report.
- Full machine access, but touch NOTHING outside this repo, no network. Only file
  edits within Scope, `cargo`, `grep`, and `python3` for parsing.
- Never edit any `mma-hol` file. **No new `unsafe`.**
- Clean-tree discipline: your working-tree changes are the deliverable, but check
  `git status --short -- crates/` (scoped to crates/) if you need a cleanliness
  probe — Claude may commit docs concurrently.
- Minimal diff; stop-loss on the same failure twice.
- Paste real verbatim outputs only; if something did not reach green, say so.

## Report format (your final message)

1. What changed — kernel_api item count + grouping; promoted definition sites
   (file list); logic files converted, with per-file import lines.
2. Verification — each command with exact verbatim output.
3. Verdict — the pilot's demanded-item count vs the audit's 9; whether the
   incremental-promotion policy held.
4. Stop-loss / open questions (empty if none).
