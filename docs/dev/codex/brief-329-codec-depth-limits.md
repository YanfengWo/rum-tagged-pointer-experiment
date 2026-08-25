# brief-329 — give the YAML and TOML decoders a depth limit

## Goal

`FromJson` refuses over-deep documents with a message. `FromYaml` and `FromToml`
recurse without any depth budget and exhaust the native stack instead. Give both
the same kind of limit, reported the same way.

## Measured today (release CLI, verified by the director — do not re-derive)

```
FromYaml[StringJoin[Table[StringRepeat[" ", i] <> "k:\n", {i, 0, 8000}]]]
  → thread 'main' has overflowed its stack, exit -6

FromToml["a = " <> StringRepeat["[", 15000] <> StringRepeat["]", 15000]]
  → exit -6

FromJson[StringRepeat["[", 300] <> StringRepeat["]", 300]]
  → $Failed
    FromJson::depth: JSON nesting exceeds the fixed depth limit of 256 at byte offset 256.

FromJson[StringRepeat["[", 200] <> StringRepeat["]", 200]]  → List   (under the cap)
```

Each of these is a one-line script. Reported thresholds from the survey are
around 7 100 nested levels for YAML and 14 100 for TOML, but the exact number is
not the point — there is no limit, so some input always reaches the stack.

## The mechanism

- **YAML**: `parse_nested_or_null` (`crates/rum-stdlib/src/yaml/decode.rs:192`)
  calls `parse_block` (`:199`) for each deeper indentation level, and
  `parse_block` calls back into it. One mutual recursion per indent level, no
  budget.
- **TOML**: `parse_array` (`crates/rum-stdlib/src/toml/decode.rs:157`) calls
  `parse_value`, which calls `parse_array` for a nested array. Same shape.
- **JSON** already tracks a `depth` parameter and checks it against
  `MAX_JSON_DEPTH` (`crates/rum-stdlib/src/json/decode.rs:6,114,120`) at each
  point it descends, reporting through `FromJson::depth` (`:48`).

## The shape to build

Follow the JSON decoder. Thread a depth through the recursive descent, check it
where the parser descends, and report a message naming the limit and the offset.

Three things to decide and state, rather than assume:

1. **The limit value.** JSON uses 256. Whether the same number suits YAML's
   indentation nesting and TOML's array nesting is a judgement — say what you
   chose and why. Prefer consistency across the three unless there is a reason
   not to, and if you deviate, say what the reason is.
2. **The message tag.** `FromJson::depth` is the precedent; the natural
   counterparts are `FromYaml::depth` and `FromToml::depth`. Check the message
   registry conventions before adding tags — this repository registers message
   tag/name pairs, and an unregistered tag may fail a vocabulary test.
3. **Every descent point, not just the obvious one.** YAML nests through block
   mappings, block sequences and flow collections; TOML through arrays, inline
   tables and dotted keys. A limit on one path while another recurses freely is
   not a fix. Enumerate the descent points you found and say how you know the
   list is complete.

## What must not change

- Documents under the limit must decode exactly as they do now, including key
  order and duplicate-key rejection.
- Existing error messages and their byte offsets for all other malformed input.
- `FromJson` is already correct — do not change its limit or its behavior.

Capture goldens from the pre-change build for a representative corpus and run
them green against the unmodified decoders first, as briefs 310 and 312 did.

## Scope

Files you MAY modify:

- `crates/rum-stdlib/src/yaml/decode.rs`
- `crates/rum-stdlib/src/toml/decode.rs`
- the message-tag registration sites the new tags require
- the `rum-stdlib` test modules for YAML and TOML

Out of scope: everything else, including `json/decode.rs`, the TOML encoder, and
the decoders' own non-depth behavior.

Do not add unconditional cost to the default evaluation path.

## Deliverable

- Both decoders bounded, reporting a message rather than exhausting the stack.
- The descent-point inventory, with your completeness argument.
- Goldens for under-limit documents, captured pre-change.
- Tests at the limit, one past it, and far past it (the sizes above).

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then, with a fresh `cargo build --release -p rum-cli` — feed stdin from
`/dev/null`, the CLI waits for input otherwise:

```
cat > /tmp/b329-1.rum <<'EOF'
Print[FromYaml[StringJoin[Table[StringRepeat[" ", i] <> "k:\n", {i, 0, 8000}]]]];
EOF
cat > /tmp/b329-2.rum <<'EOF'
Print[FromToml["a = " <> StringRepeat["[", 15000] <> StringRepeat["]", 15000]]];
EOF
cat > /tmp/b329-3.rum <<'EOF'
Print[Head[FromYaml["a:\n  b:\n    c: 1"]]];
Print[Head[FromToml["a = [[1, 2], [3]]"]]];
Print[Head[FromJson["[[1,[2]]]"]]];
EOF
for i in 1 2 3; do echo "--- $i ---"; ./target/release/rum-cli /tmp/b329-$i.rum < /dev/null; echo "exit=$?"; done
```

The first two must print `$Failed` with a depth message and exit non-zero.
**An exit code of -6 or 134 is a failure of this brief.** The third must be
unchanged.

No microbench A/B: these are codec builtins, not the evaluation hot path. Say so
if you believe otherwise.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director gates and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network. Commands: file edits in Scope,
  `cargo`, the release `rum-cli`, and `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses.
- **No new `unsafe`.**
- **Leave no `TODO` or `FIXME` markers.** This repository has none by policy.
- Minimal diff; no drive-by reformatting.
- **Stop-loss is real.** If a descent point cannot be bounded without changing
  under-limit results, STOP and report it.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per decoder, with line ranges, and the descent points you
   bounded.
2. The limit value and message tags you chose, and why.
3. Verification — each command with verbatim output and exit codes, plus the
   golden corpus size.
4. Stop-loss / open questions.
