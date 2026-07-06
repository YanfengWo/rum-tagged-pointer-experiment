# TODO

This file is for active or near-term work only. Completed history and
operational lessons live in [PROGRESS.md](./PROGRESS.md) (append-only log).

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

## Stdlib Expansion For Scripting (ACTIVE)

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
- [ ] Codec family after JSON (config-format conversion is a target
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
- [ ] brief-144 (in flight): Tokenizer mechanism for highlighting and text tooling (S/M):
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
- [ ] Candidates parked: raw multiline string literals (parser-level, real
  friction in every task, bigger blast radius — decide separately);
  schema/validation helpers; `ByteArray` depth audit.
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
- [ ] **Coverage rebuild** (the integration-depth gap left by removing the
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
- [ ] `rum-host`: verify (or add) an extension hook for downstream builtin
  families before `rum-fs` starts — a downstream crate must be able to
  register its family on a host session through the public boundary.
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
- [ ] Document the embedding adoption cost for light hosts: `rum-host`
  dependency footprint, binary-size delta, and cold `Session` +
  `install_core` startup time.
- [ ] `vm_rhs_template/` fate (experiment, not instrumentation —
  gate/delete/keep). The profiling/debug_tools fate is settled: they stay in
  the kernel, runtime-gated, hidden from restricted hosts via the Diagnostic
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
