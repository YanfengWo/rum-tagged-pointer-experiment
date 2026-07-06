# Codec Family Plan — config-format conversion on one value model

Status: ACTIVE (started 2026-07-07). Continues the codec family founded by
`FromJson`/`ToJson` (brief-138). Target application: config-format
conversion in scripts and pipelines — one shared value model makes
conversion compositional (`ToToml[FromJson[s]]`), and rum-cli's filter mode
makes it a shell one-liner.

## Family conventions (set by brief-138 — every codec follows them)

- `From*`/`To*` named pairs; pure String↔expression; `Pure` capability.
- **Fail-closed**: reject loudly (message with offset + `$Failed`) anything
  outside the documented subset — never misparse silently. Tolerant
  behavior only where explicitly documented.
- Exact numbers only where the format's spec is decimal text; no floats.
- Contract lives in `docs/guide/structured-data.md` sections + reference
  limits notes; drift test enforced.

## Steps

| Brief | Content | Size |
|---|---|---|
| 145 | `FromCsv`/`ToCsv` (RFC 4180 quoting, LF/CRLF, optional `"Delimiter"` for TSV, optional `"Headers"` → Associations; values are Strings — no type inference) + `FromIni`/`ToIni` (ONE documented dialect: `[section]`, `key = value`, `;`/`#` line comments, trimmed, no nesting/multiline; values verbatim strings; duplicates fail-closed) | S+S |
| 146 | `FromToml`/`ToToml` — folk subset over the TOML spec: tables, inline tables, arrays, arrays-of-tables, strings (basic+literal), integers, booleans; TOML floats fail closed unless the decimal is exact; datetimes decode to the brief-141 exact-timestamp values (offset folded) with a documented local-date/local-time story | M |
| 147 | `FromXml`/`ToXml` — symbolic tree `XmlElement[tag, attrs-Association, children-List]` + text nodes as Strings; elements/attributes/text/entities/CDATA/comments(dropped, documented); no DTD, no namespaces resolution (prefixes kept verbatim) | M |
| 148 | `FromYaml`/`ToYaml` — the sane block subset ONLY: block mappings/sequences, flow scalars, quoted strings, comments; NO anchors/aliases/tags/multi-doc/flow-collections; scalar typing rules pinned (unquoted true/false/null/numbers; everything else String); aggressive fail-closed — full YAML is a permanent non-goal | M/L |
| M1 | Line closure: conversion matrix test (each pair round-trips through the shared value model where representable), guide consolidation, archive briefs, bundle | — |

Gates per brief: fmt, workspace tests, reference drift, fail-closed
negatives pasted verbatim, plus one cross-codec conversion probe.

## Out of scope for the line

Streaming/incremental parsing; schema validation; comments preservation
(codecs are value-oriented; the tolerant/span-oriented counterpart is
`TokenizeWith`); plist/netrc-style also-rans.
