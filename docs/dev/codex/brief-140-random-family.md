# Brief 140 — Random family: deterministic session PRNG + entropy capability

Implementation brief. The design was settled in advance (`TODO.md`
§Stdlib Expansion): MOST of randomness is Pure — a seeded PRNG whose state
lives in session state satisfies the audit rule (results depend only on
session state + arguments), and a FIXED default seed preserves
deterministic-by-default. Only the ENTROPY TAP is ambient, and it follows
the FileIo seam pattern: a provider trait in the kernel API, the real
source in `rum-host`, gated by a new `Capability::Random`.

## Goal

- Session-state PRNG with a documented algorithm and fixed default seed;
  `SeedRandom[n]`, `RandomInteger`, `RandomChoice`, `RandomSample`,
  `BlockRandom`, `RandomUuid`, `UuidQ` — all `Pure`, all deterministic,
  byte-reproducible across sessions and runs.
- Entropy: `Capability::Random` gates `SeedRandom[]` (no-args entropy
  reseed) and `RandomBytes[n]`; an `EntropySource` provider trait in the
  kernel API; `StdEntropySource` in `rum-host` (trusted default installs
  it; `sandboxed()` does not). Permissive policy without a provider →
  `General::noentropy` + `$Failed` (mirror `General::noio`).
- Reserve `Capability::Process` in the enum (no builtins; doc comment says
  reserved for the future downstream rum-os).
- Reference/vocabulary/guide updated; workspace green.

## PRNG contract (this is a public reproducibility commitment)

- Algorithm: **xoshiro256\*\*** with **splitmix64** seed expansion —
  implement from the public-domain reference (a few dozen lines of safe
  Rust; zero dependencies). Document in the guide: the algorithm and
  seeding are part of rum's contract; the same seed yields the same
  sequence on every platform and rum version (changing it later would be a
  documented breaking change).
- Default seed: 0 (splitmix64-expanded) at session creation and after
  `reset()`. Snapshot does NOT capture PRNG state (add to the existing
  not-captured list in the guide/rustdoc).
- `SeedRandom[n]` for an exact integer `n` (any size; fold via splitmix64
  over the magnitude bytes + sign) → reseeds, returns `Null`. Pure.
- NOT cryptographic — say so plainly in the guide and reference limit
  notes. Password-grade material goes through `RandomBytes` (entropy).

## Builtin semantics

- `RandomInteger[]` → 0 or 1. `RandomInteger[n]` → uniform 0..n (n ≥ 0
  exact integer; arbitrary precision via rejection sampling — no modulo
  bias). `RandomInteger[{a, b}]` → uniform a..b (a ≤ b). Second argument
  `k` (non-negative integer) → a `List` of `k` draws. No `RandomReal` —
  rationals-only; document the boundary (a script wanting a rational in
  [0,1) can draw `RandomInteger[{0, d - 1}]/d` explicitly).
- `RandomChoice[list]` → one element; `RandomChoice[list, k]` → k
  independent draws (with replacement). Empty list → message + inert
  (family style).
- `RandomSample[list, k]` → k distinct positions without replacement
  (0 ≤ k ≤ Length), order random; `RandomSample[list]` → full shuffle
  (Fisher-Yates over the PRNG).
- `BlockRandom[body]` (`HoldAll`): saves the PRNG state, evaluates the
  body, restores the state — the real backend for the shell removed in
  brief-134; register it fresh (do not recycle the archived stub).
- `RandomUuid[]` → a version-4-format UUID string drawn from the session
  PRNG (Pure; document: reproducible under the default seed — session-local
  identifiers; use an entropy reseed first when global uniqueness matters).
  `UuidQ[str]` → format predicate (8-4-4-4-12 hex, version/variant bits
  checked). Both Pure.
- `RandomBytes[n]` → `ByteArray` of n bytes drawn DIRECTLY from the
  `EntropySource` (NOT the session PRNG); `Capability::Random`.
  `SeedRandom[]` (no args) → entropy reseed of the session PRNG;
  `Capability::Random`.

## EntropySource seam

- Kernel API trait (mirror `FileIo`'s shape): `fn fill(&self, buf: &mut
  [u8]) -> Result<(), EntropyError>`; session holds an optional boxed
  source; kernel crates stay free of any entropy syscall (the hygiene
  test's spirit — `StdEntropySource` lives in `rum-host`, reading
  `/dev/urandom` via `std::fs` there; document the unix-only scope).
- `HostConfig`: trusted default installs `StdEntropySource`; `sandboxed()`
  installs none. `rum-cli`/REPL install it like they install `StdFileIo`.

## Scope

- MAY modify: `crates/rum-runtime/src/**` (PRNG state in session +
  kernel_api accessor, `EntropySource` trait, `Capability::Random` +
  reserved `Process` variant + exhaustive-match arms, vocabulary
  registrations), NEW stdlib family `crates/rum-stdlib/src/random.rs` +
  `catalog/random.rs` + tests, `crates/rum-host/src/**` (`StdEntropySource`,
  config wiring, rustdoc, policy tests), `crates/rum-cli/src/main.rs`
  (install the source), `docs/reference/` via generator,
  `docs/guide/resource-limits-and-sandbox.md` (capability list + one
  paragraph), `docs/guide/design-boundaries.md` (determinism section gains
  the Random story), guide README link if a new section file is added.
- MUST NOT touch: evaluator/dispatch hot paths, the eval-loop poll
  machinery, matcher, string engine, other docs. No new dependencies.

## Tests

- Determinism pins: fresh session default-seed draws equal a LITERAL
  expected sequence (e.g. first five `RandomInteger[100]` values written
  into the test) — this pins the algorithm contract; same for a
  `SeedRandom[42]` sequence and for two consecutive CLI invocations
  producing identical output.
- Range/validity: bounds respected on huge ranges (≥ 2^80 spans);
  `RandomSample` output is a permutation (sorted == sorted input);
  `RandomChoice` empty-list message.
- `BlockRandom` restores the stream (draw, BlockRandom-draw, draw ==
  draw, draw with the middle absent).
- `RandomUuid` format + `UuidQ` positives/negatives.
- Capability: `{Pure}` policy denies `SeedRandom[]`-no-args and
  `RandomBytes` (sandbox message), allows all PRNG builtins; permissive
  policy without provider → `General::noentropy` + `$Failed`; trusted
  default `RandomBytes[8]` returns 8 bytes and two draws differ.
- Vocabulary/catalog registrations (brief-136 lesson) + reference drift.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
cargo run -q -p rum-stdlib --bin generate-builtin-reference && cargo test -p rum-stdlib builtin_reference_is_in_sync -- --exact
cargo run -q -p rum-cli -- --eval '{RandomInteger[100], RandomInteger[100], RandomInteger[{-5, 5}, 4]}'
cargo run -q -p rum-cli -- --eval '{RandomInteger[100], RandomInteger[100], RandomInteger[{-5, 5}, 4]}'   # identical to the previous line (deterministic default)
cargo run -q -p rum-cli -- --eval 'SeedRandom[42]; {RandomChoice[{a,b,c}, 5], RandomSample[Range[10]]}'
cargo run -q -p rum-cli -- --eval '{x = RandomInteger[1000], BlockRandom[RandomInteger[1000]], RandomInteger[1000]}'
cargo run -q -p rum-cli -- --eval '{RandomUuid[], UuidQ[RandomUuid[]]}'
cargo run -q -p rum-cli -- --eval 'RandomBytes[8]'
cargo run -q -p rum-cli -- --eval 'RandomBytes[8]'    # differs from previous (entropy)
git status --short
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network.
- Zero new dependencies; PRNG from the public-domain reference in safe
  Rust; kernel crates gain no `std::fs`/entropy syscalls (hygiene test must
  stay green — the entropy source lives in rum-host).
- No new `unsafe`. Not an eval-hot-path change; if dispatch/eval code needs
  touching beyond registrations, STOP and report.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. The PRNG contract as landed (algorithm, seeding, default-seed literals).
2. What changed — per file; capability arms listed.
3. Verification — all commands verbatim (incl. the identical-twice and
   differs-twice pairs).
4. Stop-loss / open questions (empty if none).
