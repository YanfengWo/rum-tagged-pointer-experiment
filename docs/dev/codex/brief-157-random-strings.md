# Brief 157 — Random strings: `RandomString` (PRNG) + `RandomPassword` (entropy)

Small implementation brief closing the random-family gap — and a real
footgun: composing "passwords" from the session PRNG yields the SAME
string in every fresh session (fixed default seed). The password path must
be entropy-backed and bias-free, as a builtin.

## Builtins

- `RandomString[n]` / `RandomString[alphabet_String, n]` — n characters
  drawn independently from the alphabet's characters (default alphabet:
  `A-Za-z0-9`, 62 chars, documented) using the SESSION PRNG. `Pure`,
  deterministic by default — for test data, identifiers, simulations.
  Empty alphabet or invalid n → family-style invalid-call inert.
  Alphabet = code points of the string (duplicates allowed and weight the
  draw — document).
- `RandomPassword[n]` / `RandomPassword[alphabet_String, n]` — same
  shapes, but every character drawn DIRECTLY from the `EntropySource`
  with REJECTION SAMPLING over the alphabet size (no modulo bias — state
  the method in the reference limits note). `Capability::Random`; no
  provider under a permissive policy → `General::noentropy` + `$Failed`
  (the existing contract); denied under `sandboxed()`. Not affected by
  `SeedRandom`/`BlockRandom` (it never touches the PRNG) — test that.
- Guide (the Randomness section): one warning paragraph — never generate
  secrets with the session PRNG; `RandomPassword` is the entropy path.

## Tests

- `RandomString`: pinned sequence under the default seed (deterministic
  contract); alphabet weighting; unicode alphabets (draw from
  `"日本語"`); `SeedRandom[42]` reproducibility; `BlockRandom` isolation.
- `RandomPassword`: length/alphabet membership; two draws differ (trusted
  default); unaffected by `SeedRandom[7]` before/after (streams differ
  from `RandomString`'s); capability denial + noentropy contracts
  (rum-host policy tests, brief-140 shape); a fixed-bytes test
  `EntropySource` yields the exact expected string (bias-free mapping pin
  incl. a rejection case: alphabet size 62 with byte values ≥ 248 forcing
  rejection — craft the byte sequence).
- Vocabulary/catalog registrations + reference drift (expect 600).

## Scope

- MAY modify: stdlib random family files + catalog + vocabulary + tests,
  rum-host policy tests, `docs/guide/design-boundaries.md` or the
  randomness guide section (wherever brief-140 documented it) +
  `docs/reference/` via generator.
- MUST NOT touch: the PRNG core, EntropySource seam, evaluator/dispatch,
  `Cargo.*`.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
cargo run -q -p rum-stdlib --bin generate-builtin-reference && cargo test -p rum-stdlib tests::builtin_reference_docgen::builtin_reference_is_in_sync -- --exact
rg -c '^### ' docs/reference/builtins.md      # expect 600
printf '' | cargo run -q -p rum-cli -- --eval '{RandomString[8], RandomString["abc", 5]}'
printf '' | cargo run -q -p rum-cli -- --eval '{RandomString[8], RandomString["abc", 5]}'   # identical to previous (PRNG determinism)
printf '' | cargo run -q -p rum-cli -- --eval 'RandomPassword[16]'
printf '' | cargo run -q -p rum-cli -- --eval 'RandomPassword[16]'                          # differs (entropy)
printf '' | cargo run -q -p rum-cli -- --eval 'StringLength[RandomPassword["abcdefghjkmnpqrstuvwxyz23456789", 20]]'
git status --short
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network, no
  new dependencies, no new `unsafe`.
- Rejection sampling for BOTH paths' alphabet mapping (the PRNG path too —
  keep the uniformity story uniform).
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. Contracts as landed (+ the default-alphabet and weighting decisions).
2. What changed — per file.
3. Verification — all commands verbatim (incl. the identical/differs pairs).
4. Stop-loss / open questions (empty if none).
