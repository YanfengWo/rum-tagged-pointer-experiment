# Embedding API Line (concern a) — Scenario Design & Consumer Survey

**Status: LINE COMPLETE (2026-07-16, briefs 236-242 — scenario design,
consumer survey, and all five queue items landed and gated in one
day).** a1 contract honesty (exit status, requires_reset, b1-field
adoption); a2 dogfood arc (contract surface + rum-cli + rum-repl ports;
in-tree hosts are pure rum-host consumers — cli/repl carry no
rum-runtime/rum-stdlib dependency at all); a3 typed bridge out
(Bool/Association/Rule + jupyter application/json consumer); a4 typed
bridge in (expr_from_typed + injection-safe HostSession::call; kernel
seam eval_expr_request); a5 named profiles (hermetic/scripting;
all-8-capability pins). Parked tail (background-eval helper,
per-request rollback API, lighter checkpoint) remains parked pending
scenario pull. fs/os/net stay deferred until after publication (user
2026-07-15).

Original opening status: scenario-design pass landed 2026-07-16
(Claude, own work — not a Codex brief). This opened concern (a) of the
host-embedding line per the sequencing ruling (user 2026-07-14: (a)
starts only after (b) completes, and MUST open with scenario design and
boundary drawing — no feature bricklaying on instinct). Concern (b)
closed at b4 on 2026-07-15 (`host-embedding-line.md`).

Inputs: three abandoned downstream concepts supplied by the user
(2026-07-16) as abstract scenario material, plus a fresh survey of the four
in-tree consumers (`rum-repl`, `rum-jupyter`, `rum-terminal`, `rum-cli`) —
what they actually bind to and what they had to work around. All file:line
evidence cited below was read this session.

## Ground rules (carried over + drawn here)

- **Products of this line land in the `rum-host` contract layer only**
  (plus any kernel seam a contract feature genuinely needs). No showcase
  code for any scenario — the scenarios are downstream repositories by
  definition, exactly like the b5 ruling ("building a dun plugin is
  downstream work"). `docs/python-adapter-plan.md` already freezes the
  pyo3 half of scenario S1 as a non-goal for this repo.
- **Judging criterion = general embeddable-language merits** (the
  elisp/guile/lua/python family), not any single scenario's current needs
  (user reframe 2026-07-14).
- JSON serialization stays shim-side (concern (b) ruling); a typed value
  bridge makes shim-side JSON trivial, which is the correct division.
- Standing hard lines apply: no new `unsafe`, no default-path cost for
  off-by-default features, testing policy (fixtures test rum's own hosts
  only), honest measurement for anything perf-adjacent.

## Scenario families (abstract; user-supplied 2026-07-16)

Three abandoned downstream concepts, deliberately different in character.
They are examined for what they would demand of the kernel/contract — not
built.

### S1 — Python adapter (AI-infra development-time / compile-time steps)

rum as a rules/transformation/validation engine inside Python pipelines.
Batch, non-interactive; compile-time steps demand hermetic evaluation
(same input → same output). Data arrives from the Python side (configs,
model metadata, policy rules) and results must return as native Python
types.

Demands: **bidirectional typed value bridge** (dict↔Association,
list↔List, bool↔True/False — both directions; the input direction is the
sharper pain: today the only channel is source-string concatenation =
escaping + injection surface + double serialization); a **named hermetic
profile** (mechanism complete — fixed PRNG seed, no ambient clock,
`sandboxed()` has no providers — but no citable named preset/contract);
machine-readable failure classification (largely done: b1 discriminator +
`HostFailureCode`); batch throughput on a long-lived session with a rule
base installed once.

### S2 — Rust adapter (wasm-language tooling)

rum embedded in a Rust toolchain as a symbolic rewriting engine over IR /
term structures. In-process, high-frequency, **large term trees**.

Demands: **parse-free typed entry** (the host already holds structured
IR; serializing it to rum source text and parsing it back is double waste
plus a quoting minefield — wants direct value construction + a typed
`call(head, args)`); **deep-tree construction safety** (collides head-on
with the documented limitation: deep `Expr` trees built through the Rust
API — which lacks the parser's depth guard — can overflow the stack on
drop; iterative `Drop` designs were rejected for 20-50% hot-path
regressions, so a builder API must build the guard in, not reopen
iterative Drop); fixed-rule-base × many-inputs lifecycle (reset-between /
snapshot-restore exist; whether a lighter checkpoint earns API surface is
this line's call); the typed path must not be slower than the parse path
(A/B discipline applies).

### S3 — Tauri typst IDE (dun's ancestor; in-process embedding)

GUI application embedding rum as its scripting engine: interactive,
long-lived session, user scripts extending document processing. This is
the in-process tight embedding that dun's protocol pivot abandoned — the
kind of application the 2026-07-14 reframe explicitly keeps on the table.

Demands: **named embedding profiles with multiple trust levels
coexisting** (user-authored scripts: scripting/interactive profile with
`Input`/`Dialog` via `Capability::Interactive`; third-party plugins:
sandboxed — today `HostConfig::sandboxed()` is the only preset and the
trusted side is hand-wired); **streaming output contract** (a GUI panel
wants `Print`/messages as they happen, not batched at completion; the
kernel mechanism exists as `Session::set_print_sink` but `rum-host`
exposes nothing and the semantics — ordering, threading, interaction with
cancel — are undefined); UI-thread cooperation (cancel handle, `Send`,
timeout: already solid); editor tooling surface (`complete`/`inspect`/
`definitions`: already present); typed results for UI rendering (the
bridge again).

## In-tree consumer survey (what they actually did)

| Consumer | Binds to | Notes |
|---|---|---|
| `rum-jupyter` | **`rum-host` only** | The one real contract consumer (~8.8k lines non-test). |
| `rum-cli` | `rum-runtime::Session` directly (+ `rum-host` only for `StdFileIo`/`StdEntropySource`) | Bypasses the contract for evaluation entirely. |
| `rum-repl` | `rum-runtime::Session` directly | Same. |
| `rum-terminal` | `rum-runtime` types | Rendering helpers bound to runtime types — unusable by a contract-level host. |

### Findings (each = a named contract gap or adoption debt)

- **F1 — Typed input does not exist; the CLI smuggles host data through
  the input-suspension machinery.** To bind stdin/argv the CLI queues raw
  strings via `queue_input_response` and then evaluates
  `` Global`$ScriptArgs = {InputString[], InputString[], …} ``
  (`rum-cli/src/main.rs:240-264`) — using the *interactive suspension*
  channel as a value-injection side door because source-string escaping is
  the only alternative. This is the typed-injection gap demonstrated
  in-tree.
- **F2 — `HostTyped` has ZERO consumers.** Workspace grep: no use outside
  `rum-host` itself. Even `rum-jupyter` renders from `value.text` only
  (`adapter/render.rs:69`). The typed surface shipped as dun-facing, dun
  pivoted, and it has never been exercised by a real consumer — widening
  it (Bool/Association/Rule currently collapse to `Other`,
  `model/outcome.rs:148-173`) should be paired with giving it a real
  consumer, or it stays speculative.
- **F3 — The b1 termination discriminator has zero adoption;
  `rum-jupyter` still string-matches.** `adapter/worker.rs:130-132`:
  `response.value.text == "TerminatedEvaluation[\"Cancelled\"]"`. b1
  landed `HostEvalSuccess.termination` precisely to kill this; the
  in-tree consumer was never migrated (grep: no non-test `termination`
  use in `rum-jupyter`).
- **F4 — `HostFailureCode::Exit` discards the exit status.**
  `model/outcome.rs:493`: `SessionRequestError::Exit { .. } =>
  HostFailureCode::Exit` — the code is dropped at the boundary. A
  contract-level host cannot implement `Exit[7]` → process exit 7; the
  CLI gets it from `SessionRequestError::Exit { code }` directly
  (`rum-cli/src/main.rs:351-356`), which is one of the reasons it cannot
  ride the contract.
- **F5 — Streaming prints exist only below the contract.**
  `Session::set_print_sink` is what makes the CLI a good pipeline citizen
  (streaming `Print` + EPIPE → interrupt, `rum-cli/src/main.rs:285-304`);
  `rum-host` exposes only the batched `prints` vector on the outcome. Any
  interactive/GUI host (S3) is forced below the contract for this today.
- **F6 — Script/file evaluation semantics exist only below the
  contract.** `eval_script_request` / `eval_file_request`
  (`rum-cli/src/main.rs:201,207`) have no `rum-host` counterpart —
  a contract host can only single-request `eval`.
- **F7 — Contract knowledge is hand-rolled in the consumer.**
  `rum-jupyter` decides for itself which failures poison the session
  (`adapter/worker.rs:145-151`: `Panic | Poisoned` ⇒ reset) — this
  belongs to the contract (a method or documented invariant), not to each
  host's folklore. Likewise the whole background-execution scaffold
  (`ExecuteWorker`: spawn thread, move session in, poll `is_finished`,
  join session back out, cancel handle, `catch_unwind` —
  `adapter/worker.rs:26-115,169-188`) is boilerplate every interactive
  host must reinvent; S3 would write it again verbatim.
- **F8 — (context, not a gap)** `rum-jupyter` hand-rolls JSON
  (`json_support.rs`, 435 lines) per the dependency policy. A typed
  bridge does not remove that, but it removes the need to re-parse
  rendered expression text — the shim serializes from `HostTyped`
  directly, which is the intended division (JSON stays shim-side).

### The stance question the survey forces — RULED (user, 2026-07-16)

`docs/python-adapter-plan.md` declares the standing rule: downstream
binds to `rum-host`, never to `rum-runtime` internals — and obligates rum
to keep `rum-host` *sufficient*. The survey shows it is not yet
sufficient even for our own trusted hosts (F4-F6 are each individually
disqualifying for `rum-cli`).

**USER RULING: dogfood (stance A).** In-tree trusted hosts are ported
onto `rum-host` once the missing surface lands. Rationale (user's words):
our own crates already supply excellent requirements — we can learn what
qualities a good host API needs without any external showcase. With
publication deferred there are no external consumers, so dogfooding is
the only standing mechanism that keeps the contract honest (the survey's
F2/F3 show what zero-consumer surface rots into).

Execution shape (director's plan under the ruling):
- Port `rum-cli` first (purest batch/pipeline host, isomorphic to S1;
  best sufficiency proof). HARD GATE: `rum-cli` is the microbench
  vehicle — the port must A/B the timing path and show no regression.
- Port `rum-repl` second, judging each debug command individually:
  contract-worthy surface enters the contract (e.g. message-catalog
  inspection — S3 editor tooling wants it too); deliberately-raw surface
  stays on `Session` with a documented WHY. The boundary becomes
  explicit instead of accidental.
- `rum-terminal` renderers gain contract-type twins (downstream
  embedders need rendering helpers too).
- The raw `Session` API is not deleted or hidden; the ruling governs
  what our hosts *exemplify*.

## Cross-cutting requirements (ranked by scenario hits × in-tree evidence)

| # | Requirement | S1 | S2 | S3 | In-tree evidence | Status today |
|---|---|---|---|---|---|---|
| 1 | Typed value bridge, **in** (value construction + typed `call`) | ✓✓ | ✓✓ | ✓ | F1 ($ScriptArgs smuggling) | Missing entirely; `HostEvalRequest` = `source: String` only (`model/requests.rs:42`) |
| 2 | Typed value bridge, **out** (`HostTyped` widening: Bool/Association/Rule/…) | ✓✓ | ✓ | ✓ | F2 (zero consumers — needs one) | 7 variants; collapses to `Other` (`model/outcome.rs:148`) |
| 3 | Contract sufficiency for trusted hosts (streaming print sink, script/file eval, exit status, queued input) | ✓ | — | ✓✓ | F4, F5, F6 (why cli/repl bypass) | All below the contract only |
| 4 | Named embedding profiles (hermetic / scripting / interactive / sandboxed) | ✓ | — | ✓✓ | CLI hand-wires providers | Only `sandboxed()` exists |
| 5 | Structured-outcome adoption + contract-knowledge API (b1 migration; `requires_reset`) | ✓ | — | ✓ | F3, F7 | Field exists, unadopted |
| 6 | Deep-tree construction safety (builder depth guard) | — | ✓✓ | — | Standing-debt entry (drop overflow) | Documented limitation |
| 7 | Background-eval helper (managed worker pattern) | — | — | ✓✓ | F7 (`ExecuteWorker` scaffold) | Every host reinvents |
| 8 | Per-request rollback as first-class API | ✓ | ✓ | ~ | b4 landed the pattern | Pattern only, no API |

Headline: **the typed value bridge is the line's center of gravity, and
the input direction is the sharper half** — the concern-(b) diagnosis
ranked `HostTyped` widening first because dun's needs were output-shaped;
on general embeddable-language merits, injection-safe typed input is at
least its equal (all three scenarios + the bluntest in-tree workaround).
Rows 3/5 are the "make the contract honest" cluster: cheap, evidence-rich,
and they de-risk everything else by giving the contract real consumers.

## Proposed brief queue (reordered after the stance-A ruling; order
pending final sign-off)

Order favors: honesty of the existing contract first (small, closes known
debt), then the dogfood port (creates permanent in-tree contract
consumers — after it, every later feature lands with real hosts
exercising it), then the two bridge halves (the line's core), then
profiles, then the parked tail.

1. **a1 — contract honesty batch.** `HostFailureCode::Exit` carries the
   status; `HostEvalSuccess.termination` adopted in `rum-jupyter`
   (kill the F3 string-match); poison/reset contract exposed
   (`requires_reset` or equivalent) and `rum-jupyter` ported onto it.
   Small, mechanical, all evidence-backed.
2. **a2 — trusted-host surface + dogfood port.** Streaming print sink,
   script/file eval, queued input on the contract (each judged on
   general merits — all three are scenario-backed: S3 GUI streaming,
   S1 batch-host script semantics); then port `rum-cli` (microbench
   A/B hard gate), then `rum-repl` (per-command contract-vs-raw split,
   documented), `rum-terminal` contract-type render twins.
3. **a3 — typed value bridge, out.** Widen `HostTyped`
   (Bool/Association/Rule at minimum; decide the recursion/depth-cap
   story) + give it a real consumer so it stops being speculative
   (candidate: `rum-jupyter` MIME rendering from typed).
4. **a4 — typed value bridge, in.** Host-side value construction +
   typed `call(head, args)` entry on `HostSession`; injection-safe by
   construction; answers the deep-tree drop guard (S2) at the builder;
   A/B: the typed path is not required to *win*, but must not regress
   the parse path.
5. **a5 — named profiles.** `hermetic()`/`scripting()`/`interactive()`
   presets with documented, tested semantics (sandboxed() already
   exists; hermetic is mostly a naming + contract-doc exercise).
6. **Parked pending scenario pull:** background-eval helper (row 7 —
   real but S3-only today; jupyter's scaffold works), per-request
   rollback API (row 8 — b4's pattern suffices until a consumer
   demonstrates the need), lighter checkpoint for S2's
   rule-base × many-inputs shape.

Foreseeable user rulings downstream of this queue (flagged now, decided
at brief time): a3's jupyter-MIME change is wire-visible (new MIME types
in execute results); a2's REPL per-command split (which debug commands
enter the contract); a5's preset semantics (which capabilities each
named profile grants).

## Evidence ledger (session 2026-07-16)

- `rum-cli/src/main.rs:11` — direct `rum_runtime::Session` import;
  `:240-264` input smuggling; `:285-304` print sink + EPIPE;
  `:351-356` exit-code extraction below the contract.
- `rum-repl/src/lib.rs:6` — direct runtime import; `:280-285`
  incompleteness via `parse_request` (contract equivalent exists:
  `is_complete`).
- `rum-jupyter/src/adapter/worker.rs:130-132` cancel string-match;
  `:145-151` hand-rolled reset policy; `:26-115,169-188` worker scaffold.
- `rum-jupyter/src/adapter/render.rs:69` — text-only rendering.
- `rum-host/src/model/outcome.rs:132,493` — `Exit` drops the status;
  `:148-173` — `HostTyped` variants.
- `rum-host/src/model/requests.rs:42` — `HostEvalRequest` is
  source-string-only.
- Workspace grep — `HostTyped` consumers outside `rum-host`: none;
  `termination` in `rum-jupyter` non-test code: none.
