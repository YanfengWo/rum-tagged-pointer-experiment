# Brief 271 — Sealed packages, batch 3a: snapshot-by-re-registration + re-link

## Goal

Implement design §6 (`docs/dev/sealed-scoping-design.md`, 2026-07-19
version) — the three-clause snapshot contract: (1) the held
`DefinePackage` form is each unit's snapshot representation (emitted
first, registry order; all unit-owned definition keys skipped from the
per-symbol renderer); (2) sealed-keyed state never persists (restore
returns units to as-registered privates); (3) sealed references inside
NON-sealed emitted state are detected — identity resolvable to a
current unit → wrapped in a re-link marker that a restore-mode-only
rewrite resolves onto the freshly minted sealed symbols; identity
orphaned (its unit re-registered or gone) → the snapshot FAILS LOUDLY.
Unit-owned public/package symbols' CURRENT definitions are re-emitted
marker-wrapped after the unit forms (so post-registration public
mutations persist and their sealed references re-link). Plus:
`Save`/`DumpSave` refuse sealed-referencing values; `Information` under
`Diagnostic` gains a `"SealedUnit"` row for sealed symbols. When you
are done, a session holding sealed packages and sealed-headed values in
user variables round-trips through the host snapshot with the package
working, the seal intact, identities FRESH, and sealed-free sessions'
snapshot bytes UNCHANGED from today.

Design note (director ruling, record in your report): the design doc's
"stale-epoch refusal" is implemented as snapshot-time ORPHAN refusal —
a snapshot is internally self-consistent (units and markers written
together), so restore-side staleness cannot arise; an in-session value
referencing a superseded registration's identity shows up at snapshot
time as an identity absent from every current `private_index` and is
refused there. The marker carries the owning unit context(s) (+ epoch
for diagnostics only).

## Context pointers

- Read `AGENTS.md`, the design doc §6 + §7 + §8, and
  `docs/dev/sealed-scoping/grounding-findings.md` Q6. Batch-1/2 code:
  `runtime/scoping/packages.rs` (`SourceUnit` — `held_wrapper`, `epoch`,
  `private_index`, `public`, `package_scope`; `symbol_is_sealed_private`;
  `sealed_symbols_visible`).
- Snapshot renderer: `runtime/session/session_api.rs`
  `snapshot_definitions_source` (~217) — enumerates definition keys and
  renders source text via the same render helpers `Save` uses
  (`package_tools.rs` `render_saved_symbols` family). Host wrapper:
  `crates/rum-host/src/session.rs` `snapshot_to_bytes` (~187) /
  `restore_from_bytes` (~200, reset + `eval_script_request`).
- The body rewrite helper for the restore-mode re-link is the same
  `substitute_lexical_bindings` used at registration (map = per unit:
  `Symbol::new(private_context, name)` → the fresh sealed symbol from
  the CURRENT `private_index` after re-registration).
- The marker builtin follows the `DefinePackage` wiring class (catalog
  `builtin_catalog/scoping.rs` + adapter `builtin_adapters.rs` + session
  method in `packages.rs`).

## The mechanism, pinned (implement exactly this)

**Restore mode.** New transient session field
`restoring_snapshot: bool` (declare + init false; it is EXECUTION
state — do NOT add it to the internal snapshot/restore structs, and
document that with a one-line comment). New kernel entry
`Session::eval_snapshot_restore_request(source)` = the existing script
evaluation path bracketed by setting/clearing the flag (clear on ALL
exits, including error — use a guard or explicit match, no early-return
leak). `HostSession::restore_from_bytes` switches to it. No other
caller sets the flag.

**Marker builtin.** `RestoreSealedReferences[units, HoldComplete[form]]`
— kernel builtin, `HoldAllComplete`, `Protected`; `units` = a package
context String or a List of them (epoch may ride along as
`"Pkg`" -> epoch` Rules; accept both, use only the context). Ordinary
evaluation (flag false): message `RestoreSealedReferences::mode`
("only valid inside a snapshot restore") + `$Failed`; the held form is
NOT evaluated. In restore mode: for each named unit look up the CURRENT
registry entry (missing unit → message `RestoreSealedReferences::unit`
+ `$Failed`), build the decoy→sealed map from its `private_index`,
rewrite the held form with `substitute_lexical_bindings`, evaluate the
rewritten form, return Null. Declare every message tag on the spec.

**Snapshot renderer** (`snapshot_definitions_source` — signature
becomes `Result<String, ...>` with a descriptive error type or message
string; update all callers):

1. Emit each `sealed_units` entry's `held_wrapper` rendered as input
   source, in registry order, one statement each.
2. Enumerate the per-symbol state as today, EXCEPT:
   - sealed-keyed symbols (bit-62): SKIP entirely;
   - unit-owned public/package symbols (collect the owned sets from the
     registry): skip in the main pass, then re-emit their CURRENT
     definitions after the unit forms, each statement wrapped
     `RestoreSealedReferences[<owning unit>, HoldComplete[stmt]]`;
   - any OTHER emitted statement whose rendered definition tree
     contains sealed symbols (walk each statement's Exprs with a small
     helper): all such identities resolvable to current units → wrap in
     one marker naming the union of owning unit contexts; any identity
     NOT in a current `private_index` → return the loud error naming
     the symbol and its owner key (the orphan/staleness refusal).
3. A session with no sealed units and no sealed references must produce
   BYTE-IDENTICAL output to the current renderer (pin with a test that
   compares against a pre-change captured string, or by asserting the
   new path's output equals the old rendering for such a session).

**Host surface.** `snapshot_to_bytes` becomes
`Result<Vec<u8>, HostFailure>` (a new honest failure classification —
reuse the closest existing `HostFailureCode` or add a variant; if a
variant is added, the a5 all-capability-pin tests do not care, but
grep for exhaustive matches on the enum — MECHANICAL ripple). Update
every in-tree caller (grep `snapshot_to_bytes`; rum-repl/rum-jupyter/
tests) mechanically: propagate or `.expect` in tests.

**Save/DumpSave refusal.** Using the same tree-walk helper: if an
emitted statement in `Save`/`DumpSave` contains a sealed symbol,
push message `Save::sealedref` (declare on both specs — check how
shared message declarations are done for the Save family; follow the
existing pattern) and return `$Failed` without writing. (Rationale: a
saved file has no restore mode; `Get` would hit the marker refusal
anyway — refuse at the source.)

**Information row.** For a sealed symbol with `Diagnostic` visible,
`information_data_expr_for_symbol` gains one rule
`"SealedUnit" -> "<owning context>"` (registry reverse lookup; absent
for non-sealed symbols and when hidden). No other display change; no
`Definition` output-format change.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/scoping/packages.rs`
  - `crates/rum-runtime/src/runtime/session/session_api.rs`
  - `crates/rum-runtime/src/runtime/model/session.rs`,
    `crates/rum-runtime/src/runtime/session/session_state.rs`
    (the transient flag: declare + init ONLY)
  - `crates/rum-runtime/src/runtime/builtin_catalog/scoping.rs`,
    `crates/rum-runtime/src/runtime/builtin_adapters.rs`
  - `crates/rum-runtime/src/runtime/builtin_catalog/namespace/contexts.rs`
    (REVISION 1: the `Save`/`DumpSave` specs live here since the
    260-261 catalog split — message declarations only)
  - `crates/rum-runtime/src/runtime/package_tools.rs`
  - `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs`
  - `crates/rum-host/src/session.rs` + in-tree `snapshot_to_bytes`
    callers (MECHANICAL; list each in the report)
  - `crates/rum-runtime/src/runtime/tests/sealed_snapshot.rs` (NEW) +
    `crates/rum-runtime/src/runtime/tests/mod.rs`; host round-trip
    tests where they live in `rum-host`
  - `crates/rum-runtime/src/runtime/tests/messages.rs` (vocabulary
    counts, MECHANICAL)
  - `docs/reference/builtins.md` (regenerated; 637 → 638)
  - CONDITIONAL, MECHANICAL ONLY (report each): exhaustive matches on
    `HostFailureCode` if a variant is added; existing snapshot tests
    whose call sites change signature.
- MUST NOT touch: `crates/rum-core/**`, `resolution.rs`, rum-cli/
  rum-repl beyond mechanical signature propagation, `AGENTS.md`,
  `PROGRESS.md`, `TODO.md`, `docs/**` other than the regenerated
  reference, `docs/dev/codex/**` except your report, `.git`,
  `Cargo.lock`.
- No default-path cost: everything here is registration/snapshot/
  restore/inspection cold paths. The evaluator and resolver are
  untouched.

## Deliverable

- The mechanism above.
- Tests (Rust-level; the probe package from batch 1 plus a
  value-producing package, e.g. `mk[x_] := tag[x]` public with `tag`
  private so `thm = Pkg`mk[7]` holds a sealed-headed value):
  1. `snapshot_roundtrip_restores_seal`: register, mutate a private via
     a public setter, snapshot, restore into the session: public
     interface works, private back at as-registered value (clause-2
     pin), spelled private name still a decoy, and the sealed identity
     AFTER restore differs from before (fresh mint, Rust assert).
  2. `snapshot_roundtrip_relinks_leaked_values`: `thm = Pkg`mk[7]`
     before snapshot; after restore `thm`'s head equals the CURRENT
     sealed `tag` (works with the package's own pattern checks), not a
     decoy — the marker path proven.
  3. `snapshot_persists_public_mutation`: post-registration
     redefinition of a public symbol (referencing the private) survives
     restore re-linked.
  4. `snapshot_refuses_orphans`: make `thm`, re-register the package
     (fresh identities), snapshot → loud error naming the orphan.
  5. `marker_refused_in_ordinary_evaluation`: evaluating
     `RestoreSealedReferences[...]` normally → `mode` message +
     `$Failed`, held form side-effect-free.
  6. `sealed_free_snapshot_bytes_unchanged`: byte-parity for a session
     without sealed anything.
  7. `save_refuses_sealed_references`: `Save`/`DumpSave` on a symbol
     whose value holds a sealed head → `sealedref` message, no output.
  8. `information_sealed_unit_row`: Diagnostic Information on the
     sealed symbol includes `"SealedUnit"`; hidden variant does not.
  9. Host-level round-trip through `HostSession::snapshot_to_bytes` /
     `restore_from_bytes` for scenario 2 (the API seam proven).
- Regenerated reference (638 entries).

## rum / Rust pitfalls (read twice)

1. **The restore flag must be exception-safe** — cleared on every exit
   path of the new eval entry; a leaked true flag would let ordinary
   code run markers. Guard pattern, then verify with test 5 run AFTER a
   failed restore.
2. **Wrap statements, do not re-associate them**: a marker wraps ONE
   emitted statement; do not merge statements or change their order
   relative to today's renderer beyond the pinned unit-first layout.
3. **The re-link map is per-unit and name-keyed**: decoy
   `Symbol::new("Pkg`Private`", name)` → current sealed symbol. Do not
   map by identity (the snapshot text has none).
4. **Byte-parity for sealed-free sessions is a hard gate** (test 6);
   run it before and after your renderer refactor.
5. **Message tags declared or the registry assertion fails.**
6. **Stop-loss is real.** Same failure twice → STOP, report, tree
   clean.

## Verification (MANDATORY — run, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

(The 9 rum-jupyter loopback tests fail under the dispatch sandbox —
known; state it if they are your only failures.)

Then the microbench, twice, on the candidate (noise documentation; the
true A/B is the director's gate step):

```
bash scripts/core-language-microbench.sh --output-dir /tmp/brief271_run1
bash scripts/core-language-microbench.sh --output-dir /tmp/brief271_run2
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope; STOP and report instead.
- Nothing outside this repo; no network. Commands: file edits in Scope,
  `cargo`, the release `rum-cli`, the microbench scripts, `python3`.
- Testing Policy hard line; no new `unsafe`; minimal diff; verbatim
  outputs only.

## Report format (your final message)

1. What changed — per file, line ranges, one-line why.
2. Verification — verbatim: fmt, workspace totals, the two microbench
   row sets, the reference entry for the marker builtin.
3. Verdict — the round-trip scenarios' observed behavior; the
   byte-parity result; any deviation from the pinned mechanism.
4. Stop-loss / open questions; CONDITIONAL/mechanical-ripple uses with
   their forcing errors.
