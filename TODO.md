# TODO

This file is for active or near-term work only. Completed history and
operational lessons live in [PROGRESS.md](./PROGRESS.md) (append-only log).

## ACTIVE PLAN — cleanup + message-registry line (user ruling 2026-07-17)

Queue, in order; each item gates before the next starts:

1. [x] **Builtin message-tag registry (+ docgen pairing) — COMPLETE
   (briefs 243, 245-249, 252-253; 2026-07-17, one day).** Inventory
   (114 sites, 144 tags, 966 closed pairs) → mechanism
   (`BuiltinSpec.messages` + four-surface merge, user wins, shapes
   frozen) → fills (kernel 304 + stdlib 433/214) → docgen pairing
   (reference renders Messages blocks from the same source) → carriers
   (General 12 + UpTo 2, user ruling; 965 registered pairs, reference
   615 entries) → enforcement (debug assertion at push_message,
   declared-or-exempt rule; zero gaps at landing). Excluded by proof:
   open families + one unreachable ledger row (SetDelayed::argform).
   Follow-up candidates recorded, NOT queued: Take/Drop `UpTo` support
   (WL-conformance gap found by brief-252's stop-loss); catalog file
   splits (namespace/rules/strings/structure/associations.rs entered
   size-warning ranges from the fill).
2. [x] **`vm_rhs_template/` — DELETED (brief-244, ruled 2026-07-17).**
   Evidence: the ON path crashed the guard-adjacent recursion test
   (native stack overflow; OFF green, single-variable discriminator);
   flip-ON preconditions from 2026-07-01 never done; only recorded win
   was a closed-era workload. Gate: workspace 1512/0, zero references,
   A/B no regression (two flagged control labels cleared by a 3-pair
   interleaved re-run; downvalue tier flat-to-better).
3. [x] **rum-jupyter `signature.rs` sha256 debt — DONE (brief-250).**
   Hmac<Sha256> replaces the hand-rolled stack (165→42 lines); frozen
   vector test proves equivalence; deps 070a23d. ZeroMQ ordering flake
   did NOT recur (watch stays at two sightings). Side result: first
   workspace-write sandbox dispatch — works; loopback-bind denial makes
   the 9 native ZeroMQ tests sandbox-blocked (deterministic, not the
   flake); the director's out-of-sandbox gate covers them.
4. [x] **rum-cli feature forwarding — DONE (brief-251).** Four-crate
   chain (rum-host/rum-terminal/rum-repl/rum-cli forward `hash` with
   default-features=false deps); defaults byte-equivalent; NDF
   `cargo tree` shows zero digest crates; Cargo.lock unchanged;
   DIGEST_FORMAT cfg follow-up applied.
5. [ ] **IN PROGRESS: the wl15/wfr design pass** (three-batch plan
   approved 2026-07-17): structured exceptions (folk scope),
   `SubValuesHoldAll`-equivalent, `ReplaceAllUnheld`,
   ConformSchema/SchemaErrors operator forms (the last three share the
   usability pass's hold design appendix).
   - [x] Batch 1 — Take/Drop/TakeDrop `UpTo[n]` specs DONE (brief-254,
     5a2ef4a): clamp semantics, kernel Part-span endpoint parity;
     gate correction narrowed `UpTo::innf` to definitely-invalid
     contents (negative/rational), symbolic stays silent-inert.
     Workspace 1523/0.
   - [~] Batch 2 — the hold half (design doc `docs/dev/wl15-wfr-hold-design.md`;
     rulings A1/B1/B2/C1 recorded 2026-07-17):
     - [x] 2a mechanism — `SubValuesHoldAll` attribute (brief-255,
       8186536): bit 1<<15, compound-head args prepare as HoldAll via
       the existing base-symbol walk; symbol-head calls untouched.
       Workspace 1528/0; A/B no regression (director 3-pair interleaved).
     - [x] 2b-I mutation operator forms (brief-256, acaf1f3): nine
       compound-assignment heads adopt the attribute + one wrapper
       family; unary captures quiet-inert. Workspace 1537/0; A/B clean.
       Two revisions (stdlib file ownership; kernel handler location).
     - [x] 2b-II schema operator forms (brief-257, 476cf19):
       ConformSchema/SchemaErrors → special_with_hold; capture
       evaluates nothing, application byte-parity with direct call
       (validation-time resolution is pre-existing on both forms).
       Workspace 1543/0; A/B clean. One revision (parity-contract probe).
     - [x] 2c `ReplaceAllUnheld` — separate builtin (brief-258,
       48e909f): evaluates target first, then a structural descent mask
       skips held arg positions (original pre-rewrite head attributes;
       compound-head SubValuesHoldAll honored). ReplaceAll byte-
       unchanged. Workspace 1547/0; A/B replaceall-simple -2.53%
       (clean). Director completed the one vocabulary-mirror line Codex
       stop-lossed on. **BATCH 2 CLOSED.**
   - [x] Batch 3 — structured exceptions (E1-E6 green-lit 2026-07-18,
     all per recommendation). **3a core (brief-259, 42a2476):** new
     `EvalError::ExceptionThrown` channel (separate from Throw/Catch by
     construction — Catch never mentions it); five heads (Exception
     inert + ThrowException/CatchExceptions/RegisterExceptionType/
     ExceptionTypes, stdlib-registered via ControlDataBuiltin); string
     type hierarchy in a session-field registry (snapshot round-trip);
     ancestry dispatch (handler order × nearest ancestor,
     suspension-safe); E4 uncaught → in-band Failure["UncaughtException"];
     E5 registration-required fail-closed; E6 WithCleanup runs / Enclose
     passes through. Workspace 1554/0; reference 621. Two brief revisions
     (both correct Codex stop-losses on my brief omissions).
     **3b guide+close-out:** evaluation.md §Structured Exceptions added
     (director-written, examples measured). **BATCH 3 CLOSED.**
   **THE wl15/wfr DESIGN PASS IS COMPLETE** (batches 1+2+3;
   briefs 254-259).

**WON'T-DO ruling (user 2026-07-17): the parked pools are CLOSED, not
parked.** Stdlib parked candidates (`ByteArray` depth audit, compression
codecs, unicode normalization, ipaddress folk, fuzzing, codec follow-ons)
and the embedding line's parked tail (background-eval helper, per-request
rollback API, lighter checkpoint) are all WON'T-DO; re-open only on an
explicit new user ruling. Standing Engineering Debt entries below remain
as records of known limitations, not scheduled work.

## Feature-Gating Policy (ADOPTED 2026-07-07)

External-dependency stdlib families must not be forced on users who do not
want them (ARCHITECTURE.md §No unwanted dependencies). Rule: pure families
always-on; small pure-Rust dep → default-on Cargo feature; heavy/C-lib dep
→ non-default feature; OS-touching/very-heavy → downstream crate via the
`with_builtin_installer` hook.

- [x] brief-160 DONE (3d60497): feature skeleton in `rum-stdlib`; digest
  family behind default-on `hash` (`sha2`/`sha1`/`md-5`/`crc32fast`
  optional). Hex/base64/ByteArray stay always-on. Reference/drift defined
  at default features; `--no-default-features` builds and tests green with
  none of the digest crates in the dependency graph.
- Measured size context (2026-07-07, local A/B, `rum-cli` defaults):
  plain release 8.53 MiB → +strip 7.24 MiB → +lto+codegen-units=1
  6.19 MiB (env-var profile overrides; repo untouched). For comparison:
  libpython3.14 core 5.26 MiB + 60 MB stdlib dir; Lua ~0.3 MiB (no
  batteries). The candidate follow-up landed 2026-07-17 (brief-251):
  rum-cli forwards `hash` end-to-end and is digest-trimmable via
  `--no-default-features`.
- Compression = downstream `rum-compress` crate, AFTER `rum-fs` (C-library
  deps; never in `rum-stdlib`). Heavy encryption beyond hashing = non-
  default feature or downstream.

## Usability Pass — COMPLETE (2026-07-07 → 2026-07-14, briefs 161-230)

Was the mainline; all 10 sweep units are closed — **605 builtins swept**
(reference 613 after the FileNames restore). Canonical per-unit ledgers,
axes, rules, and close-outs: [docs/dev/usability-pass.md](./docs/dev/usability-pass.md);
memory `usability-pass-line.md`. User directives at open (still in force
for any future builtin work): HIGH coverage (all installed builtins, the
WL-inherited surface included) and compatible enhancement (WL names keep
WL-conformant defaults; improvements land as new options or rum-native
siblings). Method: audit briefs produce all-rows findings tables per
unit; gaps triage into small fix briefs (the 158/159 shape).

**HOST-EMBEDDING LINE — concern (b) CLOSED at b4 (2026-07-15); concern (a)
is the NEXT line, deferred to a fresh session
([docs/dev/host-embedding-line.md](./docs/dev/host-embedding-line.md)).**
Two parallel concerns: (a) the general embeddable-language API judged on
general merits; (b) dun's `pure-sandbox` + protocol conformance.
Sequencing (user 2026-07-14): (b) first, (a) only after (b), scenario-design
first.

- [x] b1 (brief-231, 74afadf): structured termination discriminator.
- [x] b2 (brief-232, ee8b788): request-level memory ceiling.
- [x] b3 (brief-233, b245590): stdout-ownership hygiene hard line.
- [x] b4 (brief-234, a3e5a76): crash-only + per-request isolation contract.
- [CANCELLED] b5 (protocol host prototype): user boundary ruling
  2026-07-15 — building a dun plugin is downstream work, not rum's job;
  the sandbox is hardened to downstream-usable strength at b4. Part 1
  (brief-235 codec crate) was reverted (revert `93a93d2`). b1–b4 reviewed
  boundary-clean (generic sandbox primitives, zero dun/editor naming).

**fs/os/net downstream capabilities: DEFERRED until after publication
(user 2026-07-15) — not the next target.** ACTIVE: concern (a) — the
EMBEDDING API LINE. Scenario-design pass landed 2026-07-16
([docs/dev/embedding-api-line.md](./docs/dev/embedding-api-line.md)):
three user-supplied downstream scenario families (Python/AI-infra, Rust/
wasm tooling, tauri typst IDE = dun's ancestor) + in-tree consumer survey
(findings F1-F8). USER RULINGS 2026-07-16: dogfood = stance A (port
rum-cli/rum-repl onto rum-host; cli first with microbench A/B hard
gate); queue signed off as a1 → a2 → a3 → a4 → a5.

- [x] a1 (brief-236): host failure-classification honesty —
  `HostFailure.exit_status` carries `Exit[code]`;
  `HostFailureCode::requires_reset()` contract method (poison truth
  verified: only the guard's caught-panic path poisons); rum-jupyter
  adopts the b1 termination field (string-match killed; a literal
  `TerminatedEvaluation["Cancelled"]` result value no longer
  misclassifies as cancelled). Workspace 1488/0.
- [x] a2a (brief-237): trusted-host contract surface —
  `HostSession::set_print_sink`/`clear_print_sink` (exclusive streaming,
  not tee; does not survive reset), `eval_script`, `eval_file`
  (FileIo-provider path; provider-less = `Other` failure),
  `queue_input_response` (the intended batch-input channel). Zero
  kernel changes. Status-quo pinned: script suspension surfaces as
  `InputRequired` but is NOT resumable (`resume_with_input` fails
  `Other`; remaining forms not evaluated) — documented, future seam if
  a consumer needs it. Workspace 1494/0.
- [x] a2b (brief-238 + revision): `rum-cli` non-interactive paths
  ported onto `HostSession`, byte-preserving (27 CLI tests frozen,
  zero expectation changes). Fixed the two contract gaps the port
  exposed: `HostValue.text` → CanonicalHoldFormTransparent (canonical
  style, one top-level `HoldForm` peeled — `HoldForm[1+1]` renders
  `Plus[1, 1]`; jupyter text/plain inherits the fix);
  `HostFailure.syntax_rendered` carries the full diagnostic.
  `--parse`/`--format` now go straight to `rum-syntax` (no session).
  REPL branch stays raw until a2c (marked). A/B verdict: NO regression
  — Codex's sequential-run STOP was overturned by the director's
  4-pair interleaved re-test (5 suspect labels cleared, median Δ 0.00%,
  `with-local`'s apparent +14% shown to be label bistability: the same
  candidate binary observed at both levels). Workspace 1495/0.
- [x] a2c (brief-239): `rum-repl` ported onto `HostSession`, output
  bytes frozen (REPL + CLI expectations unchanged). User rulings
  applied: `:lower` → contract `lower_dump` (diagnostic-grade, format
  not a stability promise); `:messages` → contract `message_catalog`
  (reuses `HostMessageTemplate`; per-symbol view stays on inspect).
  `:parse`/`:format` go straight to rum-syntax. rum-terminal is now
  the contract-type render-twin crate (7 dead runtime-typed renderers
  deleted, grep-verified consumer-free). **rum-cli and rum-repl no
  longer depend on rum-runtime/rum-stdlib at all — the in-tree hosts
  are pure rum-host consumers; the a2 dogfood arc is CLOSED.**
  Workspace 1498/0.
- [x] a3 (brief-240): typed value bridge, out — `HostTyped` gains
  `Bool` (True/False convenience mapping), ordered `Association`, and
  immediate `Rule` variants; delayed entries (`:>`, `RuleDelayed`)
  honestly stay `Other` (a data view must not present an unevaluated
  rhs as a value); depth-cap discipline uniform. First real consumer:
  jupyter execute results gain a conservative `application/json` MIME
  (user-approved wire addition — only fully-representable trees:
  Integer/Str/Bool/List/string-keyed deduplicated Association;
  existing three MIME entries byte-unchanged). Workspace 1506/0.
- [x] a4 (brief-241): typed value bridge, in — `expr_from_typed`
  construction (depth-guarded at MAX_TYPED_DEPTH; `Other` rejected as
  output-only; BigInteger/Rational validated; rationals normalize) +
  `HostSession::call(head, args)`, the injection-safe invocation path
  (head + `Symbol` args resolve via the session's real
  `resolve_symbol_name` — short names find System builtins exactly as
  source would; injection pinned by a hostile-string test). One
  kernel seam: `eval_expr_request` = eval_request's post-lower tail
  extracted (behavior frozen, verified). A/B: no regression — Codex's
  interleaved run flagged 3 labels on razor-thin per-label spreads
  (1-11µs); the director's independent interleaved re-run measured
  those labels' true spreads at 5-15% with deltas +0.6~0.9%, median
  0.00%, bidirectional scatter. Workspace 1519/0.
- [x] a5 (brief-242): named profiles, minimal set per user ruling —
  `hermetic()` (determinism contract; shares sandboxed()'s
  implementation today, promises may diverge) + `scripting()` (the
  named trusted-local preset = default()'s semantics; rum-cli adopts
  it). Each of the three named profiles pinned by an
  all-8-capability probe test (a ninth Capability variant now fails
  compilation until every profile takes a stance) + hermetic
  determinism smoke (RandomInteger streams replay across sessions).
  Workspace 1523/0.

**EMBEDDING API LINE (concern a) — COMPLETE (2026-07-16, briefs
236-242).** All five queue items landed and gated in one day. The
in-tree hosts are pure rum-host consumers (cli/repl dropped
rum-runtime/rum-stdlib deps entirely); the typed bridge is
bidirectional and injection-safe; the contract carries exit status,
syntax diagnostics, reset classification, streaming prints,
script/file eval, queued input, lower_dump, message_catalog, and three
named capability-pinned profiles. Canonical doc:
docs/dev/embedding-api-line.md. Next direction DECIDED 2026-07-17: the
ACTIVE PLAN at the top of this file (fs/os/net remain deferred until
after publication per the 2026-07-15 ruling; the line doc's parked queue
tail is closed WON'T-DO by the same 2026-07-17 ruling).

- [x] brief-161 DONE: strings audit — 101 rows / 99 names, findings in
  `docs/dev/usability-pass/strings-findings.md`; triage + batch queue in
  the tracker (crypto batch BLOCKED on a user decision: hand-rolled
  SHA-256 in `strings/encoding.rs` + toy `Encrypt` vs the
  no-kernel-crypto policy).
- [x] brief-162: correct stop-loss (Listable would change nested/mixed
  inert forms; pinned current behavior in a test). Resolution in the
  tracker: align upward.
- [x] brief-162b DONE (c474840): attribute hygiene — 18 `Protected`
  pattern symbols; `Listable` case conversion with kernel threading
  owning list mapping (net code reduction). Workspace 1342/0.
- [x] brief-163 DONE (b5e8920): crypto surface plan A — toy
  `Encrypt`/`Decrypt`/`EncryptedObject` removed, hand-rolled SHA-256
  deleted, `Hash` onto RustCrypto behind `hash` with
  SHA1/SHA512/MD5/CRC32; reference 602; workspace 1343/0; NDF green.
- [x] brief-164 DONE (e221abc): operator forms for the eight
  WL-documented string builtins via the existing callable-wrapper
  chain; microbench A/B twice — no reproducible delta (noise floor
  13-53% on this machine's sub-ms labels; structural cost = one
  fast-fail branch).
- [x] brief-165 DONE (f762e2d): Overlaps parity + options registration;
  workspace 1345/0. (Watch item: one transient rum-jupyter test failure
  in Codex's first workspace pass, green on rerun — if it recurs, brief
  it together with the signature.rs sha256 debt.)
- [x] brief-166 DONE (559d4ab): text-shaping options + WL forms;
  workspace 1345/0. (Hygiene note from its assess-on-touch:
  `text_shape.rs` in warning range; split boundaries recorded in the
  report — option parsing / format helpers / catalog subfamilies.)
- [x] brief-167 DONE (46f6e96): url/argv/encoding ergonomics; workspace
  1345/0. Gate lesson: reference LIMITS notes live in `docgen.rs` —
  include it in MAY when a brief changes documented limits.
- [x] brief-168 DONE (944a6de): strict modes; workspace 1345/0.
  **UNIT 1 (strings) CLOSED** — deferred/debt ledger in the tracker.
- [x] brief-169 DONE (992e904): unit-2 audit, 89/89; triage + five-batch
  queue in the tracker (operator forms frozen pending perf thaw).
- [x] brief-170 DONE (226bceb): Heads registration for the map/apply
  eight; workspace 1345/0. PERF MORATORIUM LIFTED same day — unit-2
  operator forms thawed as brief-175 (after 171), microbench A/B gate
  restored.
- [x] brief-171+171b DONE (2d018ca): spec flexibility six-pack;
  171's inverted Partition anchoring caught at gate (output-semantics
  review, not just green tests) and corrected in 171b; workspace
  1345/0.
- [x] brief-175 DONE (d1f0b4a): 14 operator forms + wrapper non-Call
  early exit; A/B twice (3+3 Codex, 2+2 independent) — zero regressions,
  early exit IMPROVES symbol-head labels (downvalue-exact-hit -7.1%);
  workspace 1348/0.
- [x] brief-172 DONE (5528ea2): collection comparison controls (seven
  builtins; GatherBy silent-singleton bug fixed); workspace 1351/0.
- [x] brief-173 DONE (5044e0b): fail-closed diagnostics for
  Inner/Dot/MapAt/SubsetMap (crisp error-vs-symbolic seam); 1353/0.
- [x] brief-174 DONE (2249bfd): SelectPositions/Duplicates/BinarySearch;
  workspace 1356/0. **UNIT 2 (structure+functional) CLOSED** —
  ledger in the tracker.
- [x] brief-176 DONE (d87bc97): unit-3 audit, 83/83; healthier family
  (no silent bugs); four-batch triage in the tracker.
- [x] brief-177 DONE (55629d0): comparison controls; workspace 1358/0.
- [x] brief-178 DONE (02750b1): array predicate test/rank forms;
  workspace 1358/0.
- [x] brief-179 DONE (63c1363): ten predicate operator forms (A/B
  clean); OrderedQ[p]/Lookup[key,default] excluded (data-form
  collision); workspace 1360/0.
- [x] brief-180 DONE (828d774): KeyGroupBy/AssociationKeyDeflatten+
  Flatten/MergeNested; workspace 1366/0. **UNIT 3 CLOSED** (271
  builtins swept across units 1-3).
- [x] brief-181 DONE (191a9e6): unit-4 audit, 63/63; triage in the
  tracker. Standout: `Exit` is Capability::Pure but requests host
  termination — USER DECISION pending on the capability relabel.
- [x] brief-182 DONE (5733f2f): rewrite operator forms (five; A/B
  clean); workspace 1367/0.
- [x] brief-183 DONE (1af0a9d): relabel `Exit` from `Capability::Pure`
  to `Capability::Process` — sandboxed() drops it, trusted CLI keeps it
  (Exit[7] still exits 7); rum-host asserts both directions. Workspace
  1368/0. Memory: [[exit-capability-fix]].
- [x] brief-184 DONE (29f5fd3): rewrite options — `ReplaceRepeated`
  `MaxIterations -> Automatic` (per-call runaway budget, clamped to the
  session ceiling; malformed fails closed; safety) + `Replace`
  `Heads -> False` (levelspec head traversal, Cases/DeleteCases parity;
  flexibility). No A/B gate; defaults byte-identical. Reuses the shared
  `parse_heads_option_with_default` helper. Workspace 1371/0.
- [x] brief-185 DONE (f18c17b): `Sum`/`Product` malformed-iterator
  `argform` parity with `Do`/`Table` (mirror the shared `parse_do_spec`
  `None` verdict; inert return byte-identical, only a message added).
  Cross-crate ripple: a rum-stdlib test asserting the old silence was
  updated (necessary, not scope creep). Workspace 1373/0.
- [x] **UNIT 4 CLOSED (2026-07-07):** 181 audit + 182-185. Close-out
  decisions in the tracker: `Equivalent` Flat/OneIdentity REJECTED
  (conservative — WL surface unconfirmed, no oracle per policy, folk
  scope gains nothing, touches planning); `Implies` short-circuit stays
  REJECTED. Separate future candidate: boolean-head attribute hygiene
  (missing `Protected`; `And`/`Or` `{HoldAll}` vs WL — likely
  intentional, own design pass). Units 1-4 = 334 builtins swept.
- [x] brief-186 DONE (unit 5 AUDIT): math families 62/62 →
  `docs/dev/usability-pass/math-findings.md`. Folk-scope held (Factor/
  Together/Expand/rationals-only = OK-intended). Triage + audit
  corrections in the tracker. Key correction: negative/zero args are NOT
  a blanket fail-close — WL DEFINES some (PartitionsP/Q[-1]->0), so the
  fix returns WL's value where defined, messages only where WL errors,
  defers where uncertain.
  USER decision 2026-07-07 (singularities): 1/0, 0^0, (-1)!, zero
  div/mod/mult stay INERT (folk-conservative) — NO domain messages, NO
  special-value arithmetic; only clean WL values fixed. Reshaped the
  fix batches accordingly (arithmetic-singularity fail-closed batch
  DROPPED; FromDigits list range-check = WL-conformant, not a gap).
- [x] briefs 187-192 DONE: 187 argument/arity validation (ceb58b5), 188
  clean WL values Mod[m,0]/PartitionsP-Q[-1] (cb2c173), 189
  Coefficient/CoefficientList Laurent-leak guard (bb41439), 190
  Commonest[list,n]+Quantile[list,{q..}] (e67cfdc), 191 Total level forms
  (95426db), 192 PowerMod modular inverse (1d5e885). Full detail +
  close-out decisions in the tracker (unit-5 section) and
  [[usability-pass-line]].
- [x] brief-193 DONE (54e76d3): attribute hygiene CONSERVATIVE — Protected
  (GCD/LCM/Max/Min/Abs) + NumericFunction (Max/Min/Abs). Zero eval-cost,
  no reordering (FullForm[Max[x,3]] stays "Max[x, 3]"), reductions
  unchanged; runtime attribute mirror synced; reference regenerated.
  Orderless/Flat/OneIdentity REJECTED (no-default-cost). Workspace 1385/0.
- [x] **UNIT 5 CLOSED (2026-07-07):** 186 audit + 187-193. Units 1-5 =
  396 builtins swept. Close-out decisions in the tracker (singularities
  inert per user; Orderless/Flat/OneIdentity rejected; FromDigits list
  form WL-conformant).
- [x] **UNIT 6 CLOSED (2026-07-08):** 194 audit + 195-199 (digest-format
  fail-closed, Protected sweep, option discovery, datetime ergonomics,
  DatePlus operator form). Spun out the **stateful↔pure separation line**
  (briefs 200-202, COMPLETE): `AbsoluteTime[]` = the sole numeric
  wall-clock reader; `FromDateList` = pure converter; `Now` removed;
  `SeedRandom` split → `ReseedRandom[]` + `SeedRandom[seed]`. Standing
  rule: stateful reads are single-purpose symbols gated by a SPEC-level
  capability — never crammed into a pure symbol, never a runtime check.
- [x] **UNIT 7 CLOSED (2026-07-08):** 203 audit + 204-207 (`Protected` on
  all 11 random heads; nested `{dims}` for RandomInteger/RandomChoice;
  weighted RandomChoice, exact rationals; `WhitespaceCharacters` +
  RandomPassword option discovery). Units 1-7 = 446 builtins swept, all
  fixes compat; reference 612. Deferrals in the tracker.
- [ ] **UNIT 8 (kernel A, 55) — AUDIT LANDED (brief-208, c640d8c; 2026-07-12).**
  First kernel unit. Kernel list derived: reference 612 − stdlib catalog 453 =
  159 kernel builtins; unit 8 = 55, leaving 104 for units 9-10. Findings:
  `docs/dev/usability-pass/kernel-a-findings.md`.
  **Headline**: the `Protected` enforcement surface is only 129/612 —
  `Plus[a_,b_] := 42` succeeds and `1+2` then returns `42`; all 55 unit-8
  names lack `Protected`, the protection machinery (`Set`, `Protect`,
  `Unprotect`) included. Enforcement point itself works (`RandomInteger`
  rejects). 208 carries a whole-registry `Protected` coverage section + the
  per-file fix map. Boundary kept honest: `Protected` is a mistake-guard + WL
  conformance, NOT a sandbox boundary (real plugin isolation = the host line).
  - **REAL BUG found (gate reversal, bdb125f):** `ApplyTo[x, f]` where `f`
    protects `x` mid-update lands the write on a visibly `Protected` symbol
    with no message (`x = 101`). Root cause: `store_held_update` writes without
    re-running the protection guard that the check step already uses; `ApplyTo`
    is the one compound assignment whose update (`f[current]`) runs user code
    in the store window. NOTE: my first gate pass wrongly ruled this "false"
    from a too-weak probe — see CLAUDE.md diagnosis lesson.
  - [x] **brief-209 = fix batch 1a DONE (b0c7113, 2026-07-12):** re-run
    `ensure_symbol_not_protected_for_write` at the top of `store_held_update`
    (operator threaded through; Symbol + Part owner) + `ApplyTo` regression
    test. Gate-reproduced: bypass closed, legitimate `ApplyTo` unaffected,
    Part-owner covered. Workspace 1400/0. Not a hot path; no A/B.
  - [x] **brief-210 = fix batch 2 DONE (5872796, 2026-07-12):** whole-registry
    `Protected` sweep, 129/612 → 608/612 (only `$AssertFunction`/`$ContextPath`/
    `$Packages`/`$Path` stay open). One `BuiltinSpec::protected()` const builder
    + `.protected()` per catalog entry (stdlib catalog + kernel builtin_catalog
    + mirror lockstep + regenerated reference). `Plus`/`If`/`Set`/… no longer
    redefinable by accident; WL-conformant + reversible via `Unprotect`.
    Gate: SWEEP-OK (exactly the 4 exclusions unprotected), workspace 1400/0.
    Four tests updated for the new (correct) attribute sets — necessary ripple,
    each value verified empirically (the test-control session's `If`/`Plus`
    keep harness attrs while `Identity`/`HoldAllComplete` fall through to the
    now-protected catalog).
  - [x] **brief-211 = fix batch 3 DONE (d6cbd34, 2026-07-12):** closed the
    three protection-enforcement holes — `$ContextPath`/`$Packages`/`$Path`
    special setters, `SetAttributes`/`ClearAttributes`, and `SetOptions` (the
    last reachable only with a known option; `optnf` had masked it). All reuse
    `ensure_symbol_not_protected_for_write`; the `ClearAttributes[f, Protected]`
    + `Unprotect` escape hatches preserved. Five regression tests + nine
    `Unprotect` ripple edits. Gate: workspace 1405/0, four probes reproduced.
  - [x] **brief-212 = fix batch 4 DONE (87b9067, 2026-07-12):** round-trippable
    value-table assignment. `OwnValues`/`DownValues`/`UpValues`/`SubValues[f] =
    rules` install the rules as `f`'s table (atomic replace). NOTE: the audit's
    "silent false round-trip" (E7) had ALREADY been closed by batch 2's sweep
    (the protected inspector head made it fail loudly); this batch is the
    feature half. New `value_table_assignment.rs` reuses
    `store_definition_target` (matcher-compiling install) +
    `classify_tagged_definition` (head-matching identical to `f /: … := …`).
    Validate-all-then-replace → no partial tables. Cross-symbol copy fails
    closed (head is `f`, not `g`); working form is `(DownValues[f] /. f -> g)`.
    DefaultValues deferred. Gate: workspace 1415/0 (10 new tests); A/B clean —
    unaffected controls moved MORE than the affected path (noise floor above
    the delta).
  - [x] **brief-213 = fix batch 5 DONE (868d2c6, 2026-07-12), closing:**
    (a) wildcard name-pattern targets — `Clear`/`ClearAll`/`Remove`/`Protect`/
    `Unprotect` share one normalizer and the matcher already existed (it is
    what `Names` uses), so one change gave all five pattern support;
    per-target protection preserved, no-match is a silent no-op creating no
    symbol. (b) `Information` gains `{HoldFirst, Protected}` (it used to
    EVALUATE its argument while sibling `Definition` held correctly), the
    builtin catalog summary as its `Usage` fallback, and a `Messages` field.
    Workspace 1418/0.
- [ ] **DEFERRED from unit 8 — builtin message-tag registry.** `Messages[Set]`
  is empty even right after `Set::argx` fires: builtin diagnostics come from
  ~108 ad-hoc `push_message` sites with NO static catalog of a builtin's
  possible tags/templates. Building one is its own project and pairs with
  docgen (the reference should list message tags from the same source).
  Batch 5's new `Messages` field lights up for free once it exists.
- [x] **UNIT 8 CLOSED (2026-07-12):** 208 audit (55/55) + fixes 209-213.
  Units 1-8 = **501 builtins swept**. Protection surface 129/612 → 608/612;
  one real bug found and fixed at the gate (`ApplyTo` protection bypass);
  three enforcement holes closed; value tables made round-trippable.
- [x] **UNIT 9 CLOSED (2026-07-13):** 214 audit (47/47) + fixes 215-222
  (three user rulings: Module error-path reclamation w/ mma-hol gate,
  staged `With`, replacement-Sequence splice). Evaluator hold correctness
  (`Unevaluated` restore, `HoldAll` Sequence splice, `ReleaseHold`
  one-layer). Units 1-9 = **548 swept**.
- [x] **UNIT 10 CLOSED (2026-07-14):** 223 audit (57/57) + fixes 224-230.
  Real loader `FileRead` bypass fixed at the seam; real `Pause[10^100]`
  panic fixed; three capability rulings (Input → `Capability::Interactive`;
  TestReport → Clock kept; FileNames → `FileRead` glob restored). **USABILITY
  PASS COMPLETE — 605 builtins swept, all 10 units closed.** Next: the
  host-embedding line (header above).
- [x] rum-jupyter `signature.rs` hand-rolled `sha256_compress` (found by
  brief-163's sweep) — RESOLVED 2026-07-17 (brief-250): ported onto the
  workspace `Hmac<Sha256>`; see ACTIVE PLAN item 3.

## Release Transformation — COMPLETE (2026-07-06)

- [x] The whole line (M0 + briefs 124-131 + M1/M1.5/M2): differential-testing
  harness and external-engine scripts removed (archived privately),
  closed-line docs and historical dev briefs archived, narrative/policy
  documents rewritten (DISCLAIMER, README, AGENTS, testing-strategy,
  ARCHITECTURE, v0.1 boundary), wording scan across crates and kept docs,
  generated builtin reference + drift test, nine-chapter language guide,
  rustdoc + Cargo metadata, the two guide-surfaced bugs fixed (native stack
  guard; ReplaceRepeated limit-as-value), final sweep green, bundle cold
  backup taken. Process docs live in git history and the private archive.
- [ ] Publication — **deferred indefinitely (2026-07-06)**: rum publishes
  only after it has diverged far enough from its notation-family origins.
  Growth happens in rum's own namespace and directions (embedded scripting,
  sandbox safety, rapid script development — the elisp/guile/lua/python
  family). Mechanics (hosting, name, tag, `repository` metadata, CI) decided
  then.

## Stdlib Expansion For Scripting — COMPLETE (2026-07-07)

Evidence base: brief-135 (2026-07-06) wrote six real scripting tasks in
current rum (all six completed, with workarounds) and mapped the
string-pattern surface. Batches, evidence-ranked:

- [x] brief-136 DONE (ba4ad60): collection/text odds-and-ends the tasks reached for —
  `Tally`, `MaxBy`/`MinBy`, `TakeLargestBy`/`TakeSmallestBy`, `ReverseSort`/
  `ReverseSortBy`, `GatherBy`, `Riffle`, exact `Mean`, `StringPadRight`,
  `StringLines`.
- [x] brief-137 DONE (d233622): string-pattern CONTRACT + fixes. The engine already
  supports far more than documented (`~~`, alternatives, blanks, `..`,
  character classes, `Except`, `Shortest`/`Longest`, named binding — all
  work, none cataloged). Catalog + guide section; implement the four inert
  atoms (`PunctuationCharacter`, `StartOfLine`, `EndOfLine`, `WordBoundary`);
  `StringSplit` `IgnoreCase`; fix four bug-like edges (`EndOfString` in
  Replace/Position, empty-string `RepeatedNull` cases, `Shortest[".."]`
  MatchQ). No regex engine — symbolic string patterns are the rum way.
- [x] brief-138 DONE (a067bda): in-language JSON codec, Pure, rum-native names
  (string↔expr; objects→Association, JSON decimals→exact rationals; folk
  scope with documented limits). This is the CORNERSTONE of the codec
  family — it sets the family conventions: `From*`/`To*` named pairs,
  fail-closed parsing (reject loudly what the folk subset does not support,
  never misparse silently), error style, option shapes.
- [x] CODEC FAMILY LINE COMPLETE (2026-07-07, briefs 145-148 + matrix): Codec family after JSON (config-format conversion is a target
  application; one shared value model makes conversion compositional, e.g.
  `ToToml[FromJson[s]]`): `FromIni`/`ToIni` (document one dialect),
  `FromToml`/`ToToml` (after the date/time batch — TOML has first-class
  datetimes), `FromXml`/`ToXml` (symbolic `XmlElement[tag, attrs, children]`
  tree; no DTD), `FromYaml`/`ToYaml` LAST (sane block subset only,
  fail-closed; full YAML is a non-goal). CSV-from-string alongside.
- [x] Hash/encoding family DONE (98da945, brief-142): `Sha256`/`Sha512`/`Sha1`/`Md5` via the
  RustCrypto crates (never hand-rolled; deps allowed by the dependency
  policy's security/format clause; live in rum-stdlib, Pure — hashes are
  deterministic byte functions), `Crc32` via `crc32fast`; self-implemented
  `ToHexString`/`FromHexString`/`ToBase64`/`FromBase64` + `ByteArrayQ` to
  round out the byte toolbox. Hex-string default output; `Md5`/`Sha1`
  limits-noted as non-collision-resistant. Hashing yes, encryption/signing
  never in kernel. One-time network step (cargo fetch) done by Claude.
- [x] brief-139 DONE (8c11068): rum-cli pipeline citizenship (host-side only, zero
  kernel impact): non-TTY stdin enters filter mode (no banner/prompt; `-`
  reads the program from stdin); piped DATA reaches `--eval` scripts as a
  host-bound `$ScriptInput` string; stream discipline pinned by tests
  (results+Print→stdout, messages→stderr, documented exit codes); EPIPE
  from a closed downstream exits quietly; streaming Print flush documented
  as a limitation (candidate follow-up). Payoff with 138:
  `cat a.json | rum-cli -e 'FromJson[$ScriptInput] // …' > out`.
- [x] brief-140 DONE (d80a640): Random family — kernel PRNG in session state with a
  FIXED default seed (Pure by the audit rule; deterministic by default),
  `SeedRandom`/`RandomInteger`/`RandomChoice`/`RandomSample`; entropy tap
  via `Capability::Random` + an `EntropySource` provider in rum-host
  (FileIo-seam pattern); `RandomBytes` for password-grade material with
  honest non-CSPRNG-in-kernel boundaries; no `RandomReal` (rationals only).
- [x] brief-141 DONE (d672a79): date/time folk subset — pure parse/format/arithmetic
  on timestamp values; current-time readers are `Clock`.
- [x] brief-144 DONE (53292ac): Tokenizer mechanism for highlighting and text tooling (S/M):
  `TokenizeWith[rules, str]` — an error-tolerant span driver over the
  (brief-137) string-pattern language: rules as symbolic data
  (`{"class" -> pattern, …}`), longest-match + rule priority, unmatched
  input becomes `"text"` spans, output `{class, {start, end}}` with the
  `StringPosition` 1-based convention; plus `RumTokens[str]` exposing
  rum-syntax's own span/trivia lexer. Grammar DEFINITIONS are rum packages
  (dun highlighting plugins), never stdlib code — no grammar zoo. Codec
  family stays fail-closed/value-oriented; this is the tolerant/span-
  oriented counterpart.
- [x] Perl-scan additions folded into brief-143 (fe77af7) — notes: (2026-07-07; Perl core + CPAN River proxies mostly
  CONFIRMED the plan): `UrlEncode`/`UrlDecode`/`UrlParse` (pure string ops —
  corrects the Python sweep's blanket urllib exclusion; the parse half is
  pure), `ParseArgs[args, spec]` (folk argv→Association, downstream of
  `$ScriptArgs`), tab expand + text-table alignment fold into the textwrap
  batch; `Memoize` NOT needed (the `f[x_] := f[x] = …` idiom covers it —
  add one guide sentence). Ecosystem scans CLOSED: further stdlib surveys
  (Ruby/Go) expected to re-confirm, not extend.
- [x] StringForm mini-audit RESOLVED (brief-143): eager backtick-slot formatting is the contract now. Original note: cataloged but inert for the standard
  placeholder call (StringForm["`1` + `2`", 3, 4] does not evaluate) —
  another semi-dark surface; decide contract (implement or remove).
- [x] Python/Perl tier-1 batch DONE (fe77af7, brief-143) — original sweep notes:
  Python-stdlib sweep candidates (2026-07-06, pure/safe filter over
  docs.python.org/3/library): tier-1 small batch — `textwrap`-style
  `StringWrap`/`StringIndent`/`StringDedent`; `html` escape/unescape;
  `shlex`-style `ShellSplit`/`ShellQuote` (pure string ops; pairs with the
  future rum-os `RunCommand`); pure path-string helpers rounding out the
  existing family (`FileExtension`, `FileBaseName`, `FileNameSplit`); exact
  `statistics` subset (`Median`, `Mode`, `Quantile` — exact rationals).
  Tier-1b algorithms — `graphlib`-style `TopologicalSort` (+ cycle error;
  edges as Rule pairs) and `difflib`-style folk LCS diff
  (`SequenceDiff`/line diff; LongestCommonSubsequence already exists).
  Attach elsewhere: `hmac` folds into the hash brief; UUID
  (parse/format pure + v4 from the PRNG/entropy) attaches to brief-140;
  `$ScriptArgs` attaches to brief-139. Parked with conditions: unicode
  normalization (table weight), compression codecs (after ByteArray audit),
  `ipaddress` folk, glob-match-on-strings, binary search. Excluded by
  design: everything float (`math`/`cmath`/`decimal`/float statistics),
  concurrency, network protocol stacks, GUI, ambient os/io, pickle
  (DumpSave exists), sqlite.
- [x] Multiline string literals DONE (81ff8e5, brief-156): """ escaped +
  \'\'\' raw (TOML-mirror), formatter-verbatim, derivative syntax pinned.
- [x] Schema/validation helpers DONE (48ceecf, brief-155): SchemaErrors +
  ConformSchema — patterns are the type language.
- [x] CLOSED WON'T-DO (user ruling 2026-07-17; was "candidates parked"):
  `ByteArray` depth audit; compression codecs; unicode normalization;
  ipaddress folk; fuzzing; codec follow-ons.
- [ ] WL15 survey outcomes (2026-07-07, docs/dev/wl15-core-survey.md):
  (a) `SubValuesHoldAll`-equivalent — worth doing, FROZEN until the
  perf moratorium lifts (argument-evaluation path); design with the
  kernel units 8-10 audit. (b) Structured-exceptions line
  (ThrowException/CatchExceptions-style, folk scope) — design-first
  candidate, pending user green light. (c) Incremental/lazy
  collections — parked (stateful handles vs snapshot/reset).
  (d) Structured Package Format: loader half = rum-fs territory
  (never kernel); scoping half (declarative Public/Package/Private,
  private-by-default per source unit) = required design input for the
  unit-10 package-surface audit; relevant to dun plugins.
- [ ] WFR survey (2026-07-07, docs/dev/wfr-core-survey.md; idea-level
  absorption only, sources never read): unit-2 batch candidates
  `SelectPositions`/`Duplicates`/`BinarySearch`; unit-3 candidates
  `KeyGroupBy`/`AssociationKeyDeflatten`(+flatten twin)/`MergeNested`
  (config layering — pairs with codecs+ConformSchema);
  `ReplaceAllUnheld` defers to the units-8-10 hold design.
- [ ] Downstream `rum-os` (process/env; post-launch, alongside `rum-fs`):
  process execution is the maximal ambient authority and never enters the
  kernel — kernel only reserves a `Capability::Process` variant (do this in
  a nearby brief). Design sketch recorded: structured
  `RunCommand[{argv…}]` → `<|"exit", "stdout", "stderr"|>` with capped
  capture; argv-first (anti-injection), explicit `RunShell` for shell
  semantics; stdin via `"Input"` option; folk pipelines by materialized
  stdout→stdin, OS-level fd pipes only on demand; child processes wired to
  eval cancel/timeout (kill process group).

## After the Release Cut

- [x] **Toolchain / MSRV — SETTLED (2026-07-05): stay on Rust `1.85`.** It is
  Debian stable's default rustc (zero toolchain burden for adopters and
  distro builds) and the edition-2024 floor, so `rust-version = "1.85"` is
  the public MSRV. Post-`1.85` language features are not used;
  `cargo-llvm-cov` stays at the matching `0.6.21`. If an upgrade is ever
  reconsidered, re-baseline both microbench suites in the same session.
- [x] HARDENING COVERAGE LINE (A-half) COMPLETE 2026-07-07 (briefs 149-152): **Coverage rebuild** (the integration-depth gap left by removing the
  external acceptance corpus): `cargo-llvm-cov`-driven buildout prioritizing
  the host lifecycle (interrupt/cancel, wall-clock timeout, poison recovery,
  `reset()`, snapshot/restore round-trips, typed `HostValue`, capability
  sandbox, `Send`/concurrency), then error/resource-limit paths
  (recursion/iteration/parse-depth/match-depth `TerminatedEvaluation` tags,
  genuine-cycle handling), then a rum-native integration/stress corpus (deep
  recursion, large rewriting workloads, cancellation under load).
- [ ] Filesystem surface = the future downstream `rum-fs` repository (seed
  implementation archived by brief-132): scoped-fs roots first, then a
  coherent text-file set and a folk-scope JSON/CSV subset, registering
  through the kernel API and the `FileIo` provider seam. Nothing returns to
  the kernel; network I/O stays permanently out of it; streams/ports and
  binary I/O deferred (stateful handles interact with snapshot/reset/cancel).
- [x] `rum-host` downstream extension hook DONE (3cf45e6, brief-153):
  `with_builtin_installer` — ordered installers, reset()/clone survival,
  capability dispatch enforced on downstream builtins.
- [ ] Python binding (Python embeds rum): a **separate downstream
  repository** on top of `rum-host` via pyo3, started after publication
  mechanics settle ([docs/python-adapter-plan.md](./docs/python-adapter-plan.md)).
  rum-side obligations only: keep `rum-host` sufficient and publish the
  embedding-cost numbers (item above).
- [ ] Downstream embedder (separate repo, own roadmap): `dun`, a
  ratatui-based terminal editor, plans `rum` as its optional heavy-compute
  plugin engine — plugins as pure computations (input snapshots in, output
  intents out) under per-plugin resource limits, cancel/timeout, clean reset,
  and the `Pure` capability sandbox. Light paths (config, themes) stay
  native; `rum-host` is pulled in only when needed.
- [x] Embedding adoption cost DONE (d30fce2, brief-154): guide §Adoption
  Cost — session 0.09 ms steady / eval 4.5 us / 6.5 MiB stripped delta /
  23 external crates.
- [x] `vm_rhs_template/` fate — RESOLVED: DELETED (brief-244, 2026-07-17;
  see ACTIVE PLAN item 2 for the ruling evidence). The
  profiling/debug_tools fate is settled: they stay in the kernel,
  runtime-gated, hidden from restricted hosts via the Diagnostic
  capability.

## Standing Engineering Debt

- [ ] Matcher: three known gaps in `Flat`+`Orderless`+`Optional` combinations
  (binding order for flat-wrapped singletons; `ReplaceList` under-collection
  of grouped flat matches; their compound). Low urgency, exotic corner. The
  `#[ignore]`'d tests in `tests/pattern_matching_orderless_optional.rs` are
  the target spec; a previous localized fix regressed neighboring behavior
  and was reverted under stop-loss.
- [ ] Keep the suspension-safe helper boundary from regressing; continue
  moving helper payloads onto typed continuation values when a flow cannot be
  a `Result<Expr, EvalError>` plus one ordinary suspension frame. Monitor
  display refresh and `ValueQ` rollback stay best-effort unless their
  contract is promoted.
- [ ] Binary snapshot format (deferred, low priority): session persistence is
  textual today, which is fine at current scale; a binary path (direct
  deserialize, no re-parse/re-eval) becomes worthwhile for much larger
  session images.
- [ ] Symbolic `Plus`/`Times` like-term detection is O(n²) per expression
  (bounded upside at small term counts; accepted tradeoff). Revisit only with
  profiler evidence from a real symbolic-algebra workload.
- [ ] Deep `Expr` trees built directly through the Rust API (not through the
  parser, which is depth-guarded) can overflow the stack on drop; iterative
  `Drop` designs all regressed the hot path 20-50% and were rejected.
  Documented host limitation in
  [docs/host-embedding-and-hardening.md](./docs/host-embedding-and-hardening.md).
- [ ] Formatter internals: continue splitting planner/builder/render/test
  modules as the syntax surface grows; decide later whether the string
  subsystem earns a crate boundary (do not create one by default).
- [ ] REPL/CLI polish: decide whether trace viewing belongs in the REPL;
  snapshot-style regression coverage for help/inspection/message rendering.
