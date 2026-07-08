# Brief 203 — Usability audit: character_classes + random (16 builtins; light re-check)

Audit brief — sweep unit 7 of the Usability Pass (the `character_classes`
constant family + the `random` family). Read `docs/dev/usability-pass.md`
FIRST: the three axes, the compat rule, the coverage discipline, and the
closed-unit ledgers (units 1-6) — they show the judgment standard, the
corrections method, and the operator-form disambiguation rule. This is an
AUDIT: you change NO behavior and NO code; the only file you create is the
findings document.

This is a LIGHT re-check unit, and the framing matters: the
`RandomPassword` arc (briefs 157-159 — the worked example of this whole
pass) and the stateful↔pure separation line (briefs 200-202) are BOTH
FRESH and settled. Your job on those is to CONFIRM they held and are
labeled correctly, NOT to re-litigate them. The real sweep effort goes to
the rest of the `random` family and the 5 character-class constants.

## Objective

Sweep every builtin in the two catalog families of this unit against the
three usability axes and write
`docs/dev/usability-pass/character-classes-random-findings.md`.

## Count reconciliation (do this first)

The tracker lists this unit as "15" — that count predates brief-201,
which split `SeedRandom` and ADDED `ReseedRandom`. The authoritative
count today is **16**: `character_classes` = 5, `random` = 11. Your
findings table has AT LEAST one row per catalog entry; expect **16 rows**,
and state 16/16 in the coverage check (note the 15→16 reconciliation in
one line so I can see you caught it).

## The scope guardrail (READ TWICE — this governs the whole unit)

This unit is the randomness / entropy / character-class-constant surface.
Six design boundaries are INTENDED, not gaps. A row that would require
crossing one is an `OK` boundary row with a one-line "intended" note, NOT
a gap:

- **Deterministic-by-default is the CONTRACT (FS-purity line).**
  `SeedRandom` / `RandomInteger` / `RandomChoice` / `RandomSample` /
  `RandomString` / `BlockRandom` / `RandomUuid` all draw from the
  REPRODUCIBLE session PRNG — same seed, same stream. Only `RandomBytes`
  / `ReseedRandom` / `RandomPassword` touch host entropy, and only behind
  `Capability::Random`. This split is the intended safety design.
  Confirming the capability LABELS are correct IS a finding; proposing
  "make RandomInteger/Choice/etc. use entropy by default" is NOT — that
  would break determinism.
- **Rationals-only, no floats (BY DESIGN).** `RandomReal`,
  `RandomComplex`, `RandomVariate`, and WL's distribution zoo are
  DELIBERATELY ABSENT (docgen note, `docgen.rs:117`: "There is no
  RandomReal in the current exact-number boundary"). rum's random surface
  is folk uniform integer / choice / sample / string only. Do NOT propose
  floating-point randoms or distribution families — out of scope by
  design; filing them wastes the sweep.
- **Stateful↔pure separation is SETTLED (briefs 200-202, FRESH — a
  closed USER ruling).** `SeedRandom[seed]` = `Capability::Pure`,
  deterministic-only, works inside `sandboxed()`; `ReseedRandom[]` =
  `Capability::Random`, the entropy reseed, denied in `sandboxed()`.
  Standing rule: stateful reads are single-purpose symbols gated by a
  SPEC-level capability, never crammed into a pure symbol, never gated by
  a runtime check. Do NOT propose merging `SeedRandom`/`ReseedRandom`,
  re-gating either, adding a stateful form to `SeedRandom`, or a runtime
  capability check. CONFIRMING the split is clean and correctly labeled
  IS the finding here.
- **The RandomPassword arc is FRESH (briefs 157-159).** The full policy
  surface — string-keyed `"Require" -> {group,...}`,
  `"NoAdjacentRepeats" -> True|False`, `"Exclude" -> {substr,...}`,
  `"IgnoreCase" -> True|False`; entropy-backed rejection sampling; fail
  closed after 10000 attempts — is the pass's worked example. LIGHT
  re-check only: confirm it holds (policy options honored? fail-closed on
  an unsatisfiable policy? entropy-failure path returns `$Failed` +
  message? not affected by `SeedRandom`/`BlockRandom`?). Do NOT re-open
  the design.
- **ASCII / English-only character classes are folk BY DESIGN.** The 5
  constants are ASCII sets (`UppercaseLetters` A-Z, `LowercaseLetters`
  a-z, `DigitCharacters` 0-9, `PunctuationCharacters` the ASCII
  punctuation set, `Alphanumeric` the 62-char A-Za-z0-9). Unicode /
  non-English alphabets / a WL-style `CharacterRange` / `Alphabet[lang]`
  defer to the FUTURE unicode line (see the unit-1/3 close-out
  deferrals). Do NOT propose full-unicode character machinery here.
  Whether the ASCII set is COMPLETE and USEFUL, and whether an obvious
  folk-cheap sibling constant is missing, IS a finding.
- **Folk scope, not a CAS/stats library.** No distribution families, no
  seeded-generator objects, no streaming RNG. Uniform-only folk surface.

What IS a legitimate finding here (the three axes, tuned to this unit):

- **Safety — attribute hygiene (the likely main mechanical fix this
  unit):** all 11 `random` heads are registered via `BuiltinSpec::
  ordinary(...)`, which sets `AttributeSet::empty()` — so every one
  carries `Attributes: None` and LACKS `Protected` (verify against the
  reference: `RandomInteger`/`RandomSample`/`RandomString`/… all print
  `Attributes: None`). The 5 character-class heads ALREADY have
  `Protected` (`catalog/character_classes.rs` uses
  `with_attributes(ATTR_PROTECTED)`). A `Protected` sweep over the 11
  random heads is the same class as brief-162/196. Note per-head; flag
  that `BlockRandom` carries `HoldAll` (`ordinary_with_hold`) which any
  fix MUST preserve.
- **Safety — capability / policy labeling:** confirm the Pure-vs-Random
  split is exactly right (the seven deterministic heads = `Pure`; the
  three entropy heads = `Random`), and that `sandboxed()` denies the
  three entropy heads while keeping the deterministic ones. Cite existing
  `rum-host` sandbox tests for the entropy-denial rather than standing up
  a sandbox at runtime, if a live probe is not cheap. Confirm the 200-202
  separation is clean (no residual dual-form, no runtime capability
  check in the random path).
- **Safety — fail-closed:** entropy-failure paths return `$Failed` +
  message (`RandomBytes`/`RandomPassword`/`ReseedRandom` — `random.rs`
  `push_entropy_error`), and malformed-input paths fail closed rather
  than returning a silent-wrong answer (`RandomChoice[{}]` empty message,
  `RandomSample` count>length `ksamp` message, `RandomInteger`/`String`
  malformed spec → inert). Probe a representative set; a SILENT wrong /
  partial / biased result would be a real safety finding.
- **Flexibility — WL-documented, folk-cheap forms only:** the tensor
  `{dims}` output forms are the prime candidate — today the count arg is
  a scalar → 1D list (`parse_output_count` in `random.rs`), so
  `RandomInteger[spec, {n1,n2,...}]`, `RandomChoice[list, {dims}]`,
  `RandomString[spec, {dims}]` (nested arrays) are WL-documented forms
  rum lacks. Also `RandomChoice[weights -> elements]` (WL weighted
  choice) and any zero-arg convenience forms WL documents
  (`RandomInteger[]` → 0|1 is already present; check `RandomString[]`).
  NOTE only the folk-cheap ones; mark clearly which would need a nested
  generator loop vs. which fold trivially. Apply the operator-form
  disambiguation rule — most random verbs are data forms and will NOT
  qualify; say so where it applies.
- **Convenience:** are `RandomPassword`'s policy option keys
  discoverable via `Options[...]` (the unit-6 discovery-registration
  pattern), or only documented in prose? Named-alphabet defaults are
  already wired (`DEFAULT_RANDOM_ALPHABET = ALPHANUMERIC`) — confirm the
  character-class constants compose cleanly as `RandomString`/`Password`
  alphabets. Sibling-name / discoverability ergonomics.

Origin note: `SeedRandom`/`RandomInteger`/`RandomChoice`/`RandomSample`/
`RandomString`/`BlockRandom`/`RandomReal`(absent) are WL-inherited names;
`ReseedRandom` is a rum-native split (brief-201, a deliberate WL
divergence — WL overloads `SeedRandom[]`); `RandomUuid`/`UuidQ`/
`RandomBytes`/`RandomPassword` are rum-native; the 5 character-class
constants are rum-native folk names (WL uses `CharacterRange`/`Alphabet`).
Mark Origin honestly; for rum-native heads judge on the three axes
intrinsically, not on WL parity.

## Inputs

- Catalog name lists (authoritative, 16 total — findings table has AT
  LEAST one row per entry, grouped by family, catalog order):
  - `crates/rum-stdlib/src/catalog/character_classes.rs` (5:
    Alphanumeric, DigitCharacters, LowercaseLetters,
    PunctuationCharacters, UppercaseLetters).
  - `crates/rum-stdlib/src/catalog/random.rs` (11: SeedRandom,
    RandomInteger, RandomChoice, RandomSample, RandomString, BlockRandom,
    RandomUuid, UuidQ, RandomBytes, ReseedRandom, RandomPassword).
- Implementations: `crates/rum-stdlib/src/character_classes.rs` (the
  constants + `install`); `crates/rum-stdlib/src/random.rs` (the
  dispatchers + `parse_output_count`, `random_usize_below`,
  `entropy_usize_below`, `push_entropy_error`);
  `crates/rum-stdlib/src/random/password_policy.rs` (the RandomPassword
  policy engine). Session-side entropy/PRNG seams live in `rum-runtime`
  (`seed_random_with_integer`, `seed_random_from_entropy`,
  `fill_entropy`, `evaluate_with_random_state_restored`) — read but do
  NOT modify.
- Contracts: entries in `docs/reference/builtins.md` (all 16 have
  entries); `docs/guide/design-boundaries.md` (rationals-only +
  determinism-by-default — the authority for what is intentionally out of
  scope). The FS-purity line's `Capability::Random` gating.
- Sandbox behavior: check `crates/rum-host/src/` tests for the
  entropy-capability denial (the unit-6 method — cite the test rather
  than re-probing sandbox at runtime if not cheap).
- WL comparison: documented WL surfaces from your knowledge (extra
  argument forms like `{dims}`, weighted `RandomChoice`, attributes).
  Design reference ONLY — never run an external engine. For rum-native
  heads there is no WL parity target; judge intrinsically.

## Findings file format

Same as units 1-6 (the six existing findings files are the template): the
seven-column table (`Builtin | Origin | Verdict | Axis | Finding |
Proposal | Size`), `OK` rows REQUIRED (the all-rows table IS the coverage
proof), one row per axis-gap; then an **Evidence appendix** (verbatim
probe transcripts, stdin fed: `printf '' | cargo run -q -p rum-cli --
--eval '…'`; batch related checks; `file:line` citations acceptable for
code-visible gaps like the missing `Protected`) and a **Triage sketch**
(2-5 candidate fix batches ranked by script-user value, noting which touch
call planning / attribute-driven planning and therefore need the A/B
gate).

For every row, mark clearly whether an apparent gap is a REAL usability
finding or an intentional design boundary (determinism / rationals-only /
settled-separation / fresh-arc / ASCII-folk — an `OK` row with a one-line
"intended" note, NOT a gap). Getting this classification right is the
point of the unit — and this unit's twist is that two whole batches
(157-159 and 200-202) are already-landed work you are CONFIRMING, not
proposing.

## Probe budget

Cap ~20 probe evals total (16 builtins, a light unit); anything needing
more becomes an open question. Expected wall time 15-25 min. Batch related
checks into one `--eval` list where possible (e.g. all the malformed-input
fail-closed probes together; a determinism check = the same
`SeedRandom[k]; RandomInteger[…]` twice). Feed stdin (`printf '' | …`) so
the CLI does not block. `Attributes[...]` on each of the 16 heads is a
cheap way to prove the `Protected` gap in one batch.

## Verification (MANDATORY — run all, paste verbatim)

```
grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/character_classes.rs
grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/random.rs
# one-off shell check: every catalog name from the TWO files appears as a
# table row; must print COVERAGE-OK 16/16 (paste the check script + output)
git status --short          # ONLY the new findings file
cargo fmt --check           # clean — you touched no code
```

## Scope

- MAY create:
  `docs/dev/usability-pass/character-classes-random-findings.md` — the
  ONLY file in the diff.
- MUST NOT touch: any crate code, tests, guide, reference, `TODO.md`, the
  tracker, the other findings files, or anything else.

## Hard rules

- Do NOT `git commit`, branch, or push. No network, no new dependencies,
  no code changes.
- Every `gap` row is evidence-backed: a probe transcript or a `file:line`
  citation. Never claim a behavior you did not run.
- WL comparison from documented knowledge only; no external engines.
- The design boundaries are hard lines: do NOT propose entropy-by-default,
  floating-point / distribution randoms, re-merging the SeedRandom/
  ReseedRandom split, or full-unicode character machinery. A row that
  would require any of those is an `OK` boundary row, not a gap.
- Stop-loss: if the probe surface misbehaves, STOP after two attempts on
  the same probe and report it.

## Report format

1. Coverage statement: rows vs names, the check output verbatim (16/16),
   with the 15→16 reconciliation noted.
2. Top gaps by script-user value, one line each (mark the safety ones —
   the `Protected` hygiene gap, any not-fail-closed path, any capability/
   labeling or determinism hole — FIRST). Confirm explicitly that the
   157-159 arc and the 200-202 separation held (or flag precisely if
   not).
3. The triage sketch (note which batches touch call planning /
   attribute-driven planning and need the A/B gate — the `Protected`
   sweep does NOT, per the brief-196 precedent).
4. Verification outputs verbatim.
5. Stop-loss / open questions (empty if none).
