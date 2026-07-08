# dun Embedding Plugin Requirements

Status: proposal from the downstream `dun` editor project.

`dun` is a Rust terminal editor intended for SSH-oriented operations work:
open and save text files, inspect server logs, and eventually run sandboxed
plugins written in `rum`. The editor will not depend on `rum` until the
release-facing host API is stable enough to target deliberately, but its plugin
boundary is already designed around `rum` as a pure embedded evaluator.

This document records concrete usability requirements that would make `rum`
work well as `dun`'s embedded plugin language. The two motivating plugin roles
are syntax highlighting and custom log filtering.

## Host Boundary Required By dun

`dun` owns authority. A `rum` plugin is a pure computation that receives a
bounded input snapshot and returns structured data. It must not read files,
write files, run processes, access the network, mutate editor state directly,
or write terminal output directly.

Required host behavior:

- run untrusted plugin source with `HostConfig::sandboxed()` or an equivalent
  pure-only profile;
- deny filesystem, process, network, clock, entropy, diagnostic, and terminal
  authority unless a future `dun` role explicitly models and validates it;
- apply resource limits, wall-clock timeout, cancellation, and panic isolation
  to every plugin request;
- treat plugin output as data or intent, not authority;
- let the host validate every returned value against the plugin role and policy;
- keep one plugin failure from poisoning editor state or corrupting buffers.

The current `rum-host` design is close to this boundary: it has pure capability
policies, resource limits, timeout/cancel, panic poisoning, reset, snapshots,
and host-facing typed values. The gaps below are about making that boundary
ergonomic and robust for editor-style embedding.

## Syntax Highlighting Role

The highlighter role should take text plus lightweight context and return spans.
The plugin should never return concrete colors or terminal styles. It should
return semantic classes such as `comment`, `string`, `keyword`, `number`,
`key`, `value`, `section`, `operator`, `error`, or `text`; `dun` maps those
classes through its own theme and terminal-capability layer.

Existing useful foundation:

- `TokenizeWith[rules, text]` is a strong first version for simple languages and
  config files. It returns total, ascending, non-overlapping `{class, {start,
  end}}` spans using 1-based inclusive code-point positions.
- The shared string-pattern language, regular-expression wrapper, character
  classes, `StringPosition`, `StringCases`, `StringReplace`, and list/rewrite
  operations are all useful for lightweight tokenizers.
- `RumTokens[text]` is useful for highlighting `rum` source itself, but `dun`
  needs a general mechanism for arbitrary file types.

Suggested enhancements for syntax highlighting:

1. **Host function-call API.** Add a safe call entrypoint so a host can invoke a
   named plugin function with typed arguments without constructing a source
   string:

   ```rust
   session.call("DunHighlight", &[HostArg::Str(text), HostArg::Str(language)])
   ```

   This avoids source-string escaping bugs, injection hazards, and avoidable
   allocation when passing large text snapshots.

2. **Richer `HostTyped`.** Expand host-facing typed values beyond `List` so
   hosts can parse `Association`, `Rule`, booleans, null-like symbols, and a
   bounded generic call tree. `dun` can start with list tuples, but associations
   make future plugin results much clearer.

3. **Plugin sandbox profile.** Add a host-level profile or options for
   non-interactive plugin evaluation:

   - pure-only capability policy;
   - no `Input` or `InputString` suspension;
   - bounded message count and byte length;
   - bounded print/output count and byte length;
   - option to treat unexpected output as a plugin failure.

4. **Request isolation.** Pure code can still create definitions and session
   state. Highlighter calls should ideally run against a loaded plugin image but
   roll back per-request state after each call. A `call_isolated` or
   `eval_isolated` helper would prevent order-dependent highlighting, state
   leaks, and slow unbounded session growth.

5. **Span-unit options.** `TokenizeWith` currently reports code-point spans.
   `dun` uses UTF-8 byte columns internally and terminal display cells at the UI
   edge. Supporting span options would reduce host-side conversion hazards:

   - `"SpanUnit" -> "CodePoint"` as the current default;
   - `"SpanUnit" -> "Utf8Byte"`;
   - possibly `"SpanUnit" -> "LineColumn"` for line-oriented tools.

6. **Stateful or incremental tokenization.** Config files are often easy to
   tokenize with a single pass, but language files need multiline comments,
   heredocs, nested modes, or multiline strings. A future tokenizer API could
   accept an initial lexer state and return a final state so an editor can
   tokenize a viewport plus context instead of a whole file.

7. **Capture spans.** Highlighters often match a larger syntactic unit but style
   subparts differently, such as `key = value` or `name: type`. Named captures
   or nested span output would reduce fragile rule duplication.

8. **Compiled or cached tokenizer rules.** Editors call highlighters often. A
   way to cache parsed token rules in a session, or to represent tokenizer specs
   as reusable pure data, would improve repeated viewport highlighting.

9. **Structured parser-token helpers for codecs.** `rum` already has JSON, TOML,
   YAML, INI, XML, CSV, and schema helpers. Span-preserving token or diagnostic
   helpers for those formats would make config-file highlighting more accurate
   than regular expressions alone.

## Custom Log Filtering Role

The log-filter role is different from syntax highlighting. It should let an
operator quickly write a small `rum` expression that extracts useful records
from custom logs during an incident. The core workflow is:

1. `dun` streams or windows log text from files it has opened itself.
2. The user writes a `rum` filter for that site's custom format.
3. `dun` feeds bounded records, chunks, or windows to the plugin.
4. The plugin returns structured matches, extracted fields, severity labels, or
   summary rows.
5. `dun` displays the result and performs any file/editor action itself.

This role asks more from `rum` than highlighting because logs can be large,
messy, adversarial, and only partly structured.

Suggested enhancements for log filtering:

1. **Streaming-friendly host protocol.** Add guidance or APIs for repeated
   bounded calls over chunks/records. A host should not have to pass a whole log
   file to one evaluation. A useful shape is:

   - load plugin definitions once;
   - call `FilterInit[config]` once, if needed;
   - call `FilterChunk[state, chunk, meta]` repeatedly;
   - call `FilterFinish[state]` for final summaries.

   The state must remain pure data returned by the plugin and stored by the
   host, not hidden mutable authority in the runtime session.

2. **Record-oriented helpers.** Provide convenient pure helpers for common log
   tasks:

   - split text into LF/CRLF records while preserving source offsets;
   - parse key-value fields (`a=b`, `a="b c"`, bracketed tags);
   - parse delimited fields with quote/escape policy;
   - parse common timestamp shapes to deterministic values without reading the
     current clock;
   - return field spans alongside field values.

3. **Regex and string-pattern captures.** Log filters need captured groups more
   than highlighting does. APIs should make it easy to return named captures
   with source spans, not only matched strings.

4. **Offset-preserving transforms.** Operators often need to show the original
   matching log line with highlighted extracted fields. Helpers that preserve
   byte/code-point offsets through splitting, trimming, and field parsing would
   avoid host/plugin disagreement.

5. **Bounded result protocols.** A malicious or accidental filter can emit too
   many matches. Host-facing helpers should make it easy to cap result count,
   string length, message count, and nested value depth, and to report that the
   result was truncated.

6. **Resource-limit classification.** Some resource limit trips currently
   surface as successful values such as `TerminatedEvaluation[...]`. For host
   plugins, it would be useful for `HostEvalOutcome` or `HostEvalSuccess` to
   expose a structured flag that distinguishes normal success from timeout,
   cancellation, generated-element limit, iteration-work limit, and other
   recoverable terminations.

7. **Deterministic time parsing, no ambient time.** Log filters often parse
   timestamps, compare durations, and group by buckets. These operations should
   remain pure and deterministic. Current-time readers should stay outside the
   plugin sandbox, but timestamp parsing/conversion with explicit inputs should
   be available under `Pure`.

8. **Library ergonomics for incident scripts.** The language should make short
   one-off scripts pleasant:

   - operator forms for common string, list, association, and rule operations;
   - good failure messages for malformed patterns and codecs;
   - compact association/key lookup forms;
   - predictable behavior on invalid UTF-8 already decoded or escaped by the
     host;
   - examples for syslog-like, nginx-like, journald-export-like, and custom
     application logs.

9. **Plugin diagnostics as data.** Filters should be able to return warnings or
   diagnostics separately from matches. The host can show these in a status or
   diagnostics panel without treating them as terminal output.

10. **Testing harness support.** A host-facing way to run plugin fixtures would
    help both projects. A fixture can include input chunks, expected structured
    matches, expected diagnostics, and expected truncation/limit behavior.

## Suggested Result Shapes

For first adoption, prefer `List` tuple shapes because current `HostTyped`
already exposes lists. Associations can become the nicer shape once
`HostTyped` supports them structurally.

Syntax highlighting v1:

```text
List[List[class_String, List[start_Integer, end_Integer]], ...]
```

Log filtering v1:

```text
List[
  List["match", List[start_Integer, end_Integer], fields_List],
  List["diagnostic", severity_String, message_String],
  List["truncated", reason_String]
]
```

Where `fields_List` can start as:

```text
List[List[name_String, value_String], ...]
```

Later, richer `HostTyped` would allow clearer association-oriented results:

```text
<|
  "Kind" -> "match",
  "Span" -> {start, end},
  "Fields" -> <|"service" -> "api", "status" -> "500"|>
|>
```

## Priorities

The highest-value `rum` changes for `dun` are:

1. host function-call API with typed arguments;
2. richer `HostTyped`, especially associations and rules;
3. non-interactive plugin sandbox profile;
4. isolated per-request plugin calls;
5. capture/span support for string patterns and regex;
6. span-unit options for tokenizer-like APIs;
7. streaming/chunk protocol guidance for log filters;
8. record and field parsing helpers with source offsets.

`dun` can start with `TokenizeWith` and list-tuple results, but the items above
would make `rum` much more comfortable as a general embedded plugin language
for editor workflows.
