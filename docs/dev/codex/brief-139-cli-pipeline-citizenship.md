# Brief 139 — rum-cli pipeline citizenship

Implementation brief. Host-side only: `rum-cli` (and, if banner logic lives
there, `rum-repl`) — ZERO kernel-crate changes. Makes rum a well-behaved
Unix filter so shell pipelines (`|`, redirection) work naturally.

## Goal

1. **Filter mode**: when stdin is not a TTY (`std::io::IsTerminal`, stable
   well before 1.85):
   - `rum-cli` with no program argument reads ALL of stdin as the program,
     evaluates it in one session, prints the final result — no banner, no
     `rum>` prompts;
   - a positional `-` means the same explicitly.
   Interactive TTY behavior is unchanged.
2. **Piped data**: when a program IS given (`--eval`/`--file`/positional
   path) and stdin is not a TTY, the CLI pre-reads stdin and binds it,
   VERBATIM (no re-parsing, no escaping round-trip — construct the string
   value programmatically), as `` Global`$ScriptInput `` before evaluation.
   TTY stdin → `$ScriptInput` stays undefined.
3. **Script arguments**: everything after a `--` separator becomes
   `` Global`$ScriptArgs `` as a `List` of strings (in all program modes;
   defined as `{}` when `--` is present with nothing after; undefined when
   no `--`).
4. **Stream discipline, pinned by tests**: evaluation results and `Print`
   output go to stdout; runtime messages/diagnostics go to stderr; exit
   code 0 on success (a successful outcome whose value is
   `TerminatedEvaluation[...]` counts as success), nonzero on
   syntax/load/eval failures. Document the actual code set in `--help` or a
   short doc comment — whatever the CLI already uses for docs.
5. **EPIPE**: when stdout closes downstream (e.g. `| head -1`), the CLI
   exits quietly with code 0 — no panic, no error spew.

## Scope

- MAY modify: `crates/rum-cli/src/**`, `crates/rum-cli/tests/**`,
  `crates/rum-repl/src/**` ONLY if prompt/banner emission lives there,
  `docs/guide/getting-started.md` (a new "Pipelines And Filter Mode"
  section with REAL transcripts, per the guide rule).
- MUST NOT touch: `rum-core`/`rum-runtime`/`rum-stdlib`/`rum-host` source
  (binding `$ScriptInput` uses existing public session/eval APIs), other
  docs, scripts.

## Tests (rum-cli integration tests; spawning the built binary with piped
stdin from test code is fine — the kernel hygiene rule does not cover
rum-cli tests)

- `echo 'Total[{1,2,3}]' | rum-cli` → `6`, no prompts/banner in stdout.
- `printf 'x=1\nx+1\n' | rum-cli` (filter mode, multi-expression program) →
  final result only, per existing --file semantics.
- data mode: `printf 'a\nb\n' | rum-cli --eval 'StringLines[$ScriptInput]'`
  → `List["a", "b"]`; verbatim check with quotes/backslashes in the data.
- `rum-cli --eval '$ScriptArgs' -- one "two three"` → `List["one", "two three"]`.
- stderr separation: an expression that emits a message — assert the `!`
  line is on stderr and the value on stdout.
- exit codes: success=0; a syntax error → nonzero; `TerminatedEvaluation`
  value → 0.
- EPIPE: pipe a long `Print` loop into a closed/early-exiting reader and
  assert quiet exit (platform-reasonable approximation is acceptable;
  document what the test actually asserts).

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
echo 'Total[{1,2,3}]' | cargo run -q -p rum-cli
printf 'a\nb\nc\n' | cargo run -q -p rum-cli -- --eval 'Length[StringLines[$ScriptInput]]'
printf '{"k": [1, 2.5]}' | cargo run -q -p rum-cli -- --eval 'FromJson[$ScriptInput]'
cargo run -q -p rum-cli -- --eval '$ScriptArgs' -- alpha beta
bash -c 'yes "Print[1]" 2>/dev/null | head -200 | cargo run -q -p rum-cli - >/dev/null; echo pipe-exit=$?' 
git status --short
```

(The last line's shape may be adapted — show SOME real closed-pipe probe
and its exit code.)

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network.
- Zero kernel-crate changes; `$ScriptInput`/`$ScriptArgs` bound through
  existing public APIs, data passed verbatim.
- Interactive TTY behavior byte-identical (REPL tests unchanged).
- No new dependencies (`IsTerminal` is std).
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. Mode matrix as landed (tty/non-tty × program-source × data/args) — a
   small table.
2. What changed — per file.
3. Verification — all commands verbatim.
4. Stop-loss / open questions (empty if none).
