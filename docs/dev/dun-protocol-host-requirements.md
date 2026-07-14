# dun Protocol Host Requirements (supersedes the embedding framing)

Status: proposal from the downstream `dun` editor project, 2026-07-13.
Read together with `dun-embedding-plugin-requirements.md` and
`dun-general-editor-plugin-requirements.md`; this document records what
changed after `dun` shipped its plugin system, which of the earlier asks
still stand, and which are new.

## What changed on the dun side

The earlier two documents assumed `rum` would be **embedded** in the editor
process (`session.call(...)`, `HostConfig::sandboxed()`, `HostTyped`). That
architecture is dead. `dun` decided (2026-07-10) and has since shipped a
**protocol-first** plugin system: plugins run in **external host
processes** that speak the Dun Plugin Protocol over framed stdio, and the
editor never links a language runtime.

Shipped and in production inside dun today:

- Protocol v0: `u32` little-endian length-prefixed frames carrying UTF-8
  JSON envelopes (`v`, `kind`, `request_id`, `plugin_id`, optional `role`,
  optional buffer `revision`, `payload`).
- Message kinds: `hello`/`hello-ack` (handshake declares protocol version
  and **trust class**), role `request`/`response`, `diagnostic`,
  `cancel-request`, `error`, `shutdown`. There is deliberately **no
  LoadPlugin**: one host process serves one plugin (the plugin source is
  given to the host at spawn, e.g. via argv), so host lifecycle is plugin
  lifecycle.
- Policy caps enforced by the editor per plugin: max frame bytes (default
  256 KiB, checked before allocation), bounded input snapshots, bounded
  span counts, bounded stderr capture, bounded diagnostics, and a
  wall-clock timeout (default 2 s) covering the whole request.
- Failure model: **crash-only**. On timeout the editor sends a best-effort
  `cancel-request` and then kills the host; on any protocol violation
  (malformed frame, oversized frame, wrong revision, unknown fields where
  they matter, span flood) it kills the host outright and relaunches
  lazily after a cooldown. A host must tolerate SIGKILL at any instant.
- First role end to end: `syntax-highlight`. Input snapshot is
  `{buffer_revision, language, first_line, lines[]}` (a bounded visible
  window, not the whole file). Output is
  `{spans: [{line, start_col, end_col, style}]}` with **character (code
  point) columns** and a fixed style vocabulary of semantic classes —
  currently `keyword`, `comment`, `string`, `number`, `emphasis` — mapped
  to themes by the editor, exactly as the embedding doc proposed. Stale
  `revision` responses are discarded.
- Normative spec: `dun`'s `docs/plugin-protocol.md`. Conformance is
  mechanical: `hosts/check-host.py` in the dun repo is a language-agnostic
  checker, and three reference hosts exist (Rust/syntect, Python/Pygments,
  dependency-free Lua). A `rum` host can be developed against the checker
  without touching dun itself.

Trust is config-declared: `plugin.<id>.trust = pure-sandbox |
user-trusted-external`. Speaking the protocol is **not** a security claim;
ordinary interpreters must be configured `user-trusted-external`. The
`pure-sandbox` class is reserved for runtimes that are separately known to
deny all ambient authority — that class is `rum`'s slot, and it is the
reason dun still wants `rum`: **shipping untrusted third-party plugins as
a default experience is gated on a pure-sandbox host existing.** Python
and Lua hosts cannot clear that bar.

## What this makes obsolete

As dun-facing API surface, these earlier asks are gone:

- the host function-call API (`session.call("DunHighlight", ...)`);
- `HostTyped` expansion *as dun's result surface*;
- any in-process embedding profile aimed at dun.

They do not disappear — they become **internal requirements of the
`rum-host` shim** (the small executable that speaks the protocol on one
side and drives the kernel on the other). The shim still wants a typed,
injection-free way to call a plugin function and structured extraction of
the result good enough to serialize into the protocol's JSON payloads. But
that interface is now `rum`'s own to design; dun only sees the wire.

Also obsolete: any sizing constraint tied to dun's binary. `rum-host` is a
separate artifact and does not count against dun's 1 MiB budget. (Small
still helps — the deployment story is single-user SSH boxes — but it is
no longer a hard gate.)

## What survives, upgraded to the gate for pure-sandbox

The kernel-level sandbox semantics from the embedding doc survive
unchanged and are now *load-bearing for the trust class*:

1. **Pure-only evaluation profile**: no filesystem, process, network,
   clock, entropy, or terminal authority reachable from plugin code. This
   is the definition of `pure-sandbox`; without it a rum host is just
   another `user-trusted-external` interpreter.
2. **No interactive suspension** (`Input`/`InputString` must be unable to
   block a request).
3. **Panic/error isolation per request**, with the session either
   recovered or the host process exiting cleanly (dun will relaunch).
4. **Request isolation**: state created during a call must not leak into
   the next request by default. Explicit state is still possible — the
   log-filter design passes state as *data* through the host (see below).
5. **Resource-limit outcome classification**: the shim must distinguish
   normal results from timeout / cancellation / element-and-work limits to
   map them onto protocol `error` payloads versus ordinary diagnostics.
   (`TerminatedEvaluation[...]`-style in-band successes are hard to map.)

## New requirements born from the protocol

These did not exist in the embedding framing:

1. **Stdout is the protocol channel.** The single most important new
   constraint: plugin code must have **no path to the host process's
   stdout**. Scoping, to prevent a misreading: "stdout" here is the
   private pipe dun creates when it spawns *this one* host process
   (`Stdio::piped()`), not a terminal and not anything shared — every
   enabled plugin is its own process with its own pipe pair, so plugins
   can never contend with each other. The ownership question is entirely
   *within* one host process: the shim (host framework code) writes
   protocol frames to that pipe, and plugin code must not be able to — a
   stray `Print` reaching it corrupts framing and gets that one host
   killed. The kernel must let the embedder own stdout entirely and
   capture/redirect all plugin-visible output (to be forwarded, if at
   all, as bounded `diagnostic` messages or stderr). The old "bounded
   print/output" ask is now channel-integrity, not just tidiness.
2. **Crash-only lifecycle.** The host is SIGKILLed on timeout or protocol
   violation and relaunched cold. The kernel must need no destructors, no
   temp files, no persistent state, and must leave nothing behind when
   killed mid-evaluation. (A pure kernel should get this for free — it
   still deserves a test.)
3. **Cold-start latency.** dun launches hosts lazily on the first request
   and the default 2 s timeout covers handshake plus the request. Kernel
   init plus loading a small plugin image should sit comfortably in the
   low hundreds of milliseconds on modest hardware; a few ms is the ideal.
   Worth measuring and pinning in rum's CI.
4. **Kernel-side memory ceiling.** dun deliberately has **no** RSS
   watchdog (a deferred design that a kernel allocation cap would make
   unnecessary). The timeout catches runaway *time*; nothing today catches
   a plugin ballooning memory *within* the timeout. A per-evaluation or
   per-session allocation cap with a classified "memory limit" outcome is
   the single most valuable hardening item rum could add for this
   embedding.
5. **JSON at the boundary.** The wire payloads are JSON. Whether the shim
   uses rum's existing JSON codecs or its own serializer is internal, but
   result values must be extractable without trusting plugin-controlled
   serialization.

## Role roadmap alignment

dun's next plugin stage ("Distinctive Plugins") grows the protocol
role-by-role, driven by real hosts, in this expected order: `log-filter`
(successor of the removed command-output family), `document-structure`
(outline returns as a plugin), `text-transform` (sort/dedup/table-align
edits), `config-helper` (validation/diagnostics). These map directly onto
the roles in `dun-general-editor-plugin-requirements.md` — diagnostics,
formatters, outline/symbols, config validators — so that document remains
the right catalogue of *language-level* helpers; nothing in it is
invalidated by the transport change.

Two notes for the log-filter role specifically:

- The chunked design (`FilterInit`/`FilterChunk`/`FilterFinish`) survives,
  reconciled with the wire model as bounded request/response: dun sends
  `chunk + prior state` in each request payload and the plugin returns
  `matches + new state`, with state as pure data. Kernel-side, that is the
  same ask as before: cheap construction and consumption of pure state
  values; no hidden session authority.
- The bounded-results/truncation-marker conventions from the earlier docs
  will be enforced by dun's validator exactly as span validation is today
  (counts, ranges, depth, structured `truncated` marker), so result-size
  helpers on the rum side keep their value.

The language-level asks from the embedding doc that stand verbatim:
tokenizer capture spans, stateful/incremental tokenization, cached rule
representations, span-preserving codec token helpers, record/field parsing
with source offsets, regex named captures with spans, deterministic
timestamp parsing under `Pure`, adversarial-text robustness (dun's
sanitizer hardening this week — bidi overrides, zero-width characters —
confirmed how real that class of input is). One simplification: the
protocol settled on **code-point columns**, which is `TokenizeWith`'s
existing default, so the `"SpanUnit" -> "Utf8Byte"` option lost its
urgency; code-point output is now exactly right.

## Re-prioritized ask list

1. Pure-only, non-interactive plugin profile with **stdout fully owned by
   the embedder** and all plugin output captured (gate for
   `pure-sandbox`).
2. Kernel-side memory ceiling with a classified outcome (dun has no other
   defense against memory balloons).
3. Structured outcome classification (success / timeout / cancel /
   element-limit / work-limit / memory-limit) exposed to the embedder.
4. Per-request isolation with explicit pure-data state passing.
5. Typed plugin-function invocation + structured result extraction in the
   shim-facing API (the old #1 and #2, now internal to `rum-host`).
6. Cold-start latency measured and kept low.
7. Capture spans / incremental tokenization / codec token helpers / record
   parsing helpers (unchanged language-level items, in the earlier docs'
   own priority order).
8. A `rum-host` prototype validated against dun's `hosts/check-host.py`;
   dun's protocol doc, checker, and three reference hosts are the
   development kit, and the syntax-highlight role is the natural first
   target (bounded input, fixed vocabulary, easiest conformance).

Everything above keeps the earlier documents' closing principle: this must
not push `rum` toward becoming an editor framework. The protocol boundary
made the separation physical — dun asks only for a small, safe, pleasant
pure kernel behind a thin conformant shim.
