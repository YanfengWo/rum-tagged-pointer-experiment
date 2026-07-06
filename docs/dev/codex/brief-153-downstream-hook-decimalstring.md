# Brief 153 — Downstream builtin-family hook + DecimalString + guide touch-ups

Implementation brief. The quick-wins batch (A-group): the extension seam
that unblocks the future downstream family (rum-fs/rum-os/dun plugins), one
small report-formatting builtin, and two promised guide sentences.

## 1. Downstream builtin-family hook (`rum-host`)

A downstream crate must be able to register its own builtin family on a
host session through the PUBLIC boundary (today only `install_core` runs).

- Design: `HostSessionBuilder::with_builtin_installer(installer)` taking a
  callable over the kernel API (shape it to what the registry actually
  needs — likely `Arc<dyn Fn(&mut Session) + Send + Sync>` or an equivalent
  trait object; pick the minimal signature that a separate crate depending
  on `rum-runtime`'s kernel API could satisfy). Multiple installers allowed,
  applied IN ORDER after core install (and after the no-core path if
  `with_core_builtins(false)`).
- **`reset()` MUST re-apply installers** (reset rebuilds from config — a
  downstream family must survive it). Same for sessions built from
  `HostConfig` clones. State this in rustdoc and test it.
- Capability interplay is the selling point — test it: an installer
  registers (inside the test) a tiny demo family via the kernel API — one
  `Pure` builtin (`DemoDouble[n]` → `2 n`) and one classified `FileRead`.
  Assert: trusted session evaluates both (FileRead one may still need a
  provider — pick a do-nothing read or just assert dispatch reaches it);
  `sandboxed()` session evaluates the Pure one and DENIES the FileRead one
  with the standard sandbox message; after `reset()`, both still
  registered; snapshot/restore of a session with installer-added builtins
  works (definitions round-trip; the builtins themselves come from the
  installer, not the snapshot — document that).
- rustdoc (missing_docs is on) + one short paragraph in
  `docs/guide/embedding.md` (§ near Building A Session): downstream
  families install through the builder; capabilities still gate at
  dispatch.

## 2. `DecimalString[x, n]` (stdlib, Pure)

Renders an exact number as a plain decimal string with EXACTLY `n`
fractional digits (`n` a non-negative integer): `DecimalString[157/50, 2]`
→ `"3.14"`, `DecimalString[1/3, 4]` → `"0.3333"`, `DecimalString[2, 3]` →
`"2.000"`, negatives with a leading `-`. Rounding: half-away-from-zero,
documented. Non-number first argument or invalid `n` → family-style
invalid-call inert. Exact big-rational inputs must work (no floats
anywhere). Catalog row, vocabulary registration (standing lesson), tests
across the axes (exactness pins incl. carry cases like
`DecimalString[999/1000, 2]` → `"1.00"`), reference regen (expect 596).

## 3. Guide touch-ups (exact edits)

- `docs/guide/evaluation.md` (or the definitions section of
  `patterns-and-rules.md` — wherever definitions are taught): one short
  paragraph documenting the memoization idiom `f[x_] := f[x] = …` (works
  today; show a real transcript, e.g. a Fibonacci definition evaluated
  once).
- `docs/guide/text-utilities.md`: add `DecimalString` to the formatting
  section with a transcript (pairs with `AlignColumns` for numeric report
  columns).

## Scope

- MAY modify: `crates/rum-host/src/**` (builder/config/session + tests),
  `crates/rum-runtime/src/**` ONLY if the kernel registration API needs a
  minimal public re-export for installers (report what), new stdlib
  builtin + catalog + vocabulary + tests, the two guide files +
  `docs/reference/` via generator.
- MUST NOT touch: evaluator/dispatch, other docs, `Cargo.*`.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
cargo run -q -p rum-stdlib --bin generate-builtin-reference && cargo test -p rum-stdlib tests::builtin_reference_docgen::builtin_reference_is_in_sync -- --exact
printf '' | cargo run -q -p rum-cli -- --eval '{DecimalString[157/50, 2], DecimalString[1/3, 4], DecimalString[999/1000, 2], DecimalString[-7/8, 3]}'
cargo test -p rum-host installer -- --nocapture 2>&1 | grep -E '^test |^test result' | head -8
git status --short
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network, no
  new dependencies, no new `unsafe`.
- The installer seam must not add ANY cost to sessions that do not use it.
- Guide examples are real transcripts (the standing rule).
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. The installer API as landed (signature, reset/snapshot semantics) + any
   kernel re-export added.
2. What changed — per file.
3. Verification — all commands verbatim.
4. Stop-loss / open questions (empty if none).
