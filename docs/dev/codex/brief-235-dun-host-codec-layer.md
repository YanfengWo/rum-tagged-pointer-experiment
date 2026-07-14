# Brief 235 — rum-dun-host part 1: the protocol codec layer

Implementation brief — host-embedding line, item b5 (see
`docs/dev/host-embedding-line.md`), **part 1 of 2**. b5 delivers a `rum`
host that speaks dun's Plugin Protocol for the `syntax-highlight` role,
validated against dun's `hosts/check-host.py`. It is split because
building it is several things (a new crate, wire framing, JSON, protocol
logic, a sandboxed `rum` highlight engine, integration tests). This
brief builds ONLY the pure, fully-unit-testable codec bottom layer — no
main loop, no `rum` integration. brief-236 adds the binary, the main
loop, the sandboxed `rum` highlighting, and the conformance tests on top
of what this brief delivers, with zero rework of this layer.

The protocol is normatively defined by dun's read-only dev kit:
`/Users/fft/Developments/dun/docs/plugin-protocol.md` (spec),
`/Users/fft/Developments/dun/hosts/check-host.py` (machine checker), and
three reference hosts (`hosts/lua/dun-lua-host.lua` is the smallest full
example). You MAY READ those files to understand the wire format; you
MUST NOT modify anything under `/Users/fft/Developments/dun` (it is a
separate, read-only repository).

## Wire format (from the dev kit — implement exactly)

- **Frame**: `u32` little-endian payload length, then that many bytes of
  UTF-8 JSON. `check-host.py` caps an inbound frame at `1 << 20` bytes
  before allocating and treats a short read as end-of-stream.
- **Envelope** (JSON object): fields `v` (int, always 0), `kind`
  (string), `request_id` (int), `plugin_id` (string), optional `role`
  (string), optional `revision` (int), `payload` (object / null). The
  Lua host emits them in the order
  `v, kind, request_id, plugin_id, role, revision, payload`.
- **Messages this host handles** (see `check-host.py` lines 64-150 for
  the exact shapes):
  - inbound `hello` → outbound `hello-ack` with
    `payload: {host_id: <string>, trust: <"pure-sandbox"|
    "user-trusted-external">}`;
  - inbound `request` (role `syntax-highlight`, `revision`,
    `payload: {language, first_line, lines: [..]}`) → outbound
    `response` echoing `request_id` and `revision`, role
    `syntax-highlight`, `payload: {spans: [{line, start_col, end_col,
    style}]}`;
  - inbound `shutdown` → clean exit (handled in 236);
  - inbound `cancel-request` → no reply (handled in 236);
  - anything else → outbound `error` with `payload: {message: <string>}`.

This brief builds the TYPES and CODEC for all of the above. The main
loop that dispatches them is brief-236.

## Required change

### 1. New crate

`crates/rum-dun-host/`, added to workspace `members` in the root
`Cargo.toml` but **NOT** to `default-members` (it must not enter the
default `rum-cli` build or size budget — it is a reference host, like
dun's own `rust-syntect` host). `Cargo.toml`: package name
`rum-dun-host`, workspace version/edition/rust-version/license,
`[lib]` only in this brief (the `[[bin]]` comes in 236). **No
dependencies in this brief** — the codec is hand-rolled (dun itself
chose hand-rolled JSON over serde for the same trusted-computing-base
reason; a reference host should show the wire format is speakable
without a serialization framework). Add a crate-level `//!` doc and
`#![warn(missing_docs)]`.

### 2. `frame` module — framed stdio

- `read_frame<R: Read>(reader: &mut R) -> io::Result<Option<Vec<u8>>>`:
  reads a 4-byte little-endian length; a short read (0..4 bytes, i.e.
  EOF) returns `Ok(None)`; a length exceeding a `MAX_FRAME_BYTES: usize
  = 1 << 20` const returns an `io::Error` (`InvalidData`) BEFORE
  allocating; a truncated payload returns `UnexpectedEof`; otherwise
  `Ok(Some(payload_bytes))`.
- `write_frame<W: Write>(writer: &mut W, payload: &[u8]) -> io::Result<()>`:
  writes the `u32` little-endian length then the payload, then flushes.
  Return an error (do not panic) if `payload.len() > u32::MAX as usize`.
- Unit tests: round-trip a payload; EOF (empty reader → `None`); a
  4-byte header whose length exceeds the cap → error with no huge
  allocation; a header promising more bytes than provided → `UnexpectedEof`;
  a payload at exactly the cap boundary.

### 3. `json` module — minimal JSON value + codec

- `enum JsonValue { Null, Bool(bool), Int(i64), Str(String),
  Array(Vec<JsonValue>), Object(Vec<(String, JsonValue)>) }`. Object
  preserves insertion order (a `Vec`, not a map) so encoded output is
  deterministic and field order matches the protocol.
- `fn encode(value: &JsonValue) -> String`: compact (no spaces), strings
  escaped per JSON (`"` → `\"`, `\` → `\\`, `\n`/`\r`/`\t`, and other
  control chars < 0x20 as `\u00XX`). Non-ASCII printable bytes pass
  through as UTF-8.
- `fn decode(text: &str) -> Result<JsonValue, JsonError>`: a small
  recursive-descent parser for the same value set, with a depth limit
  (`MAX_DEPTH: usize = 64`) returning `JsonError` (not a panic) on
  overflow or malformed input. Accept the JSON number forms the protocol
  uses (integers; if a fractional/exponent form appears, either parse to
  `Int` by truncation is WRONG — instead reject with `JsonError` unless
  it is an exact integer, since the protocol only carries integer
  numerics; state your choice in the report). Accept and ignore
  insignificant whitespace.
- Helper accessors used by 236 (add them here, they are codec surface):
  `JsonValue::get(&self, key: &str) -> Option<&JsonValue>` (Object
  lookup), `as_str`, `as_i64`, `as_array`, `as_object`. Return `Option`,
  never panic.
- Unit tests: encode each variant; round-trip
  `decode(encode(v)) == v` for a nested object mirroring a real
  `request` envelope; string escaping incl. a quote, a backslash, a
  newline, and a `\u00XX` control char; a decode of the exact JSON
  `check-host.py` sends for `hello` and for a `request` (copy the
  literal object shape from the checker); depth-limit rejection;
  malformed input rejection (unterminated string, trailing garbage);
  a `\uXXXX` escape decodes to the right char (basic multilingual plane
  is enough; surrogate-pair handling MAY be a documented limitation).

### 4. `protocol` module — typed messages over the codec

- Inbound: `fn parse_envelope(payload: &[u8]) -> Result<Inbound,
  ProtocolError>` decoding bytes → `JsonValue` → a typed `Inbound` enum:
  `Hello`, `Request { request_id, revision, language, first_line, lines }`,
  `Shutdown`, `CancelRequest { request_id }`,
  `Unknown { request_id, kind }`. Missing/mistyped required fields →
  `ProtocolError`, not a panic. `first_line` and `revision` are ints;
  `lines` is an array of strings.
- Outbound builders returning `JsonValue` in the protocol field order:
  `hello_ack(host_id, trust) -> JsonValue`,
  `response(request_id, revision, spans) -> JsonValue` where a span is a
  small `struct Span { line: i64, start_col: i64, end_col: i64, style:
  String }` and the builder emits the `{line,start_col,end_col,style}`
  object; `error(request_id, message) -> JsonValue`. These build the
  FULL envelope (`v:0`, `kind`, `request_id`, `plugin_id`, `role` where
  the protocol includes it, `revision` where relevant, `payload`).
- `enum Trust { PureSandbox, UserTrustedExternal }` with a
  `fn as_wire(self) -> &'static str` → `"pure-sandbox"` /
  `"user-trusted-external"`. (236 decides which this host advertises.)
- Unit tests: parse the literal `hello` and `request` envelopes from
  `check-host.py` into the right `Inbound` variants with the right
  fields; build a `hello-ack` / `response` / `error` and assert the
  encoded string has the required fields and values (e.g. a response
  echoes `request_id` and `revision`, carries role
  `"syntax-highlight"`, and lists the spans); a malformed envelope
  (missing `kind`) → `ProtocolError`.

## Scope

MAY modify / create:
- `Cargo.toml` (root — add the member, NOT to default-members)
- `crates/rum-dun-host/Cargo.toml` (new)
- `crates/rum-dun-host/src/lib.rs` (new — module decls, crate doc)
- `crates/rum-dun-host/src/frame.rs` (new)
- `crates/rum-dun-host/src/json.rs` (new)
- `crates/rum-dun-host/src/protocol.rs` (new)

MUST NOT touch: any existing crate's source, `Cargo.lock` beyond what
adding a dependency-free member does automatically, anything under
`/Users/fft/Developments/dun` (read-only reference), `docs/**`, any
other file. No `[[bin]]`, no main loop, no `rum-host`/`rum` dependency
in this brief (those are 236).

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test -p rum-dun-host
cargo test --workspace          # prove the new member breaks nothing
git status --short              # only MAY-listed files
```

- **No microbench A/B**: a new non-default reference-host crate, not on
  any evaluation path. State it.

## Hard rules

- Do NOT `git commit`, branch, or push; leave changes in the working
  tree. No network. **No dependencies** — the codec is hand-rolled.
  No new `unsafe` (the crate must be `#![forbid(unsafe_code)]` or contain
  none — state which).
- The decoder parses external bytes: it must return `Result`/`Option`
  on every malformed input, never `panic!`/`unwrap`/`expect` on
  attacker-influenceable data. (Tests may `unwrap` on known-good data.)
- Match `check-host.py`'s exact field names, the `1<<20` frame cap, and
  little-endian framing. When in doubt, the checker is the spec.
- Minimal diff, no drive-by changes. You MUST paste real verbatim test
  output.
- Stop-loss: if hand-rolling the JSON decoder cleanly is turning large,
  STOP and report — do not reach for a dependency without escalating
  (that is a design decision for the director).

## Report format

1. What changed — per file, line ranges, one-line why.
2. Codec decisions: the fractional-number policy, surrogate-pair
   handling, and the unsafe posture (`forbid` vs none).
3. Test survey — the frame / json / protocol unit tests, named, with
   what each pins.
4. Verification — commands with verbatim outputs (test totals).
5. Deviations / open questions (empty if none).
