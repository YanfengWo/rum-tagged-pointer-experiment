# Brief 210 — Whole-registry `Protected` sweep (unit-8 fix batch 2)

Implementation brief — the mechanical cross-cutting fix triaged out of the
unit-8 audit. WL protects essentially every `System\`` builtin; rum protects
only 129 of 612 today, so `Plus[a_, b_] := 42` succeeds and `1 + 2` then
returns `42`. This brief adds `Protected` to every installed builtin except a
small, explicit set of deliberately-writable session variables. Read
`AGENTS.md` (Testing Policy) and the `Protected Enforcement Surface` section
of `docs/dev/usability-pass/kernel-a-findings.md` before starting.

Compat note: this is WL-conformant (WL builtins ARE protected) and reversible
(`Unprotect[f]` still works). `Protected` is checked only on the
definition-mutation path, so there is NO evaluation-hot-path cost and NO
microbench A/B is required.

## Goal

Every installed system builtin carries `Protected`, EXCEPT exactly these four
writable session variables, which stay unprotected:

```
$AssertFunction   $ContextPath   $Packages   $Path
```

When you are done, a freshly regenerated `docs/reference/builtins.md` shows
`Protected` on every `### ` entry except those four, the catalog/mirror
lockstep test passes, the reference drift test passes, and the full workspace
suite is green.

## Why exactly those four (do not expand or shrink the set without reporting)

- `$ContextPath`, `$Packages`, `$Path` have dedicated validated assignment
  branches in `finish_definition_assignment`
  (`crates/rum-runtime/src/runtime/definitions/definitions.rs:206-243`) and are
  meant to be user-assignable.
- `$AssertFunction` is assigned through the ordinary own-value path (tests set
  it directly), so protecting it would BLOCK that assignment.

Everything else in the registry is a language head, constant, attribute
symbol, option symbol, or status symbol — none documented as a user override
point (audit classification). If, and only if, protecting some OTHER symbol
turns a workspace test red because internal code assigns it via the definition
path, STOP, name that symbol in your report, and propose adding it to the
exclusion set — do NOT silently work around it. I decide any fifth exclusion.

## Mechanism (precise — follow it; it keeps source == behavior)

1. **Add one const builder to `BuiltinSpec`** (`crates/rum-runtime/src/runtime/metadata.rs`,
   next to the existing constructors):

   ```rust
   pub const fn protected(self) -> Self {
       let mut spec = self;
       spec.attributes = spec.attributes.with(Attribute::Protected);
       spec
   }
   ```

   `AttributeSet::with` is already `const` (metadata.rs:152) and idempotent, so
   `.protected()` is safe to append even to an entry that already has it
   (it will not double-anything). It preserves every existing attribute —
   holds, `Listable`, `ReadProtected`, etc. Do NOT add `ReadProtected`; this
   sweep adds `Protected` only.

2. **Append `.protected()` to every catalog entry** whose name is not one of
   the four exclusions, in both registries:
   - stdlib catalog: `crates/rum-stdlib/src/catalog/*.rs` (the `BuiltinSpec::…`
     entries — `ordinary`, `ordinary_with_hold`, `ordinary_with_attributes`,
     `with_attributes`, `special`, `new`).
   - kernel catalog: `crates/rum-runtime/src/runtime/builtin_catalog/*.rs`
     (`namespace.rs`, `rules.rs`, `debug.rs`, `scoping.rs`, `strings.rs`,
     `functional.rs`, `associations.rs`).

   Entries that ALREADY carry `Protected` (e.g. the ones built with
   `ATTR_PROTECTED` constants) may be left as-is OR get a harmless `.protected()`
   — your choice, but prefer leaving already-protected entries untouched to
   keep the diff to the ~479 that actually change. The four exclusions get NO
   `.protected()`.

3. **Keep the stdlib mirror in lockstep.** For every stdlib catalog entry you
   protect, the corresponding entry in
   `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs` must
   also gain `Protected` (switch it to the matching `_PROTECTED` attribute
   constant, or add `.with(Attribute::Protected)`). The test
   `catalog_attributes_match_kernel_vocabulary` (`crates/rum-stdlib/src/tests.rs:178`)
   enforces catalog == mirror and will fail loudly on any miss — use it as your
   mechanical checklist, iterating until it passes.

4. **Regenerate the reference** (it is generated from the catalog, so it must
   be regenerated and committed):

   ```
   cargo run -p rum-stdlib --bin generate-builtin-reference
   ```

   The drift test `crates/rum-stdlib/src/tests/builtin_reference_docgen.rs`
   compares the committed file against a fresh generation; it passes only after
   you regenerate.

Do NOT change the constructor DEFAULTS (`ordinary` etc. must keep setting
`AttributeSet::empty()`): downstream/plugin builtins register through the same
public constructors and must keep choosing their own attributes. The
`.protected()` opt-in per entry is the whole point.

## The completeness gate (this is how correctness is proven)

After the sweep + regeneration, the count of installed builtins WITHOUT
`Protected` must be exactly four, and exactly the named four. Paste this
verbatim:

```
python3 - <<'PY'
import re
from pathlib import Path
t = Path("docs/reference/builtins.md").read_text()
blocks = re.split(r'\n### ', t)[1:]
unprot = []
for b in blocks:
    name = b.split("\n", 1)[0].strip()
    m = re.search(r'- Attributes: (.*)', b)
    if not m or "Protected" not in m.group(1):
        unprot.append(name)
print("total:", len(blocks))
print("unprotected:", sorted(unprot))
expected = ["$AssertFunction", "$ContextPath", "$Packages", "$Path"]
print("SWEEP-OK" if sorted(unprot) == sorted(expected) else f"SWEEP-FAIL extra/missing={set(unprot) ^ set(expected)}")
PY
```

Must print `SWEEP-OK`. (This parser reads the whole `Attributes:` line, not
just the first backticked token — the instrument-calibration trap from
brief-208. Do not swap it for a first-token regex.)

## Verification (MANDATORY — run all, paste verbatim)

```
cargo run -p rum-stdlib --bin generate-builtin-reference   # regenerate first
# the SWEEP-OK completeness check above
cargo test -p rum-stdlib catalog_attributes_match_kernel_vocabulary
cargo test -p rum-stdlib builtin_reference                 # drift test
cargo fmt --check
cargo test --workspace                                     # full tree green
```

Paste: `SWEEP-OK`, the mirror-match test result, the drift test result, fmt
clean, and the full workspace pass/fail totals.

Sanity behavior probes (paste output) — protection now enforced on a
previously-open head, and an excluded var still writable:

```
printf '' | cargo run -q -p rum-cli -- --eval 'Plus[a_, b_] := 42; Print[1 + 2]'
# expect: Plus::wrsym message, and 3 (not 42)
printf '' | cargo run -q -p rum-cli -- --eval '$ContextPath = {"System`", "Global`"}; Print[$ContextPath]'
# expect: the assignment succeeds (excluded var stays writable)
```

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/metadata.rs` (the one new `protected()`
    builder).
  - `crates/rum-stdlib/src/catalog/*.rs` (the stdlib sweep).
  - `crates/rum-runtime/src/runtime/builtin_catalog/*.rs` (the kernel sweep +
    the `stdlib_attributes.rs` mirror).
  - `docs/reference/builtins.md` (regenerated — do NOT hand-edit; only the
    generator writes it).
- MUST NOT touch: any `eval_*` / evaluation logic, the four excluded entries,
  `TODO.md`, the tracker, the findings file, `Cargo.*`, git, or any test file
  other than what the generator/tests already cover (you add no new test in
  this brief — the completeness gate + existing drift/mirror tests are the
  proof).

## Hard rules

- No `git commit`, branch, push, or git config. Leave changes in the working
  tree for the gate.
- No new `unsafe`; `rum-core`/`rum-runtime` stay zero-`unsafe`.
- Do NOT change constructor defaults or add `Protected` to the four excluded
  symbols. Do NOT add `ReadProtected` anywhere.
- This is not a hot path (attributes are read on the mutation path, not per
  eval step); no microbench A/B. But do not add any per-call cost beyond the
  attribute bit itself.
- Minimal, uniform diff: `.protected()` appends + mirror updates + regenerated
  reference. No drive-by reformatting, no renames, no doc prose changes.
- If protecting a non-excluded symbol breaks a workspace test because internal
  code assigns it, STOP and report that symbol as a candidate fifth exclusion —
  do not work around it.
- Paste real verbatim output. If the workspace suite is not green or SWEEP-OK
  does not print, say so; never claim a total you did not see.

## Report format (final message)

1. What changed — the `protected()` builder, the count of entries swept per
   file (stdlib + kernel + mirror), the regenerated reference.
2. Verification — `SWEEP-OK`, mirror-match result, drift result, fmt, the full
   workspace totals, and the two behavior probes, all verbatim.
3. Any symbol you could NOT protect without a test failure (empty if none —
   and if non-empty, STOP before working around it).
4. Stop-loss / open questions (empty if none).
