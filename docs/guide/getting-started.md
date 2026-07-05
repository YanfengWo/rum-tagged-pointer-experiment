# Getting Started

This chapter gets from a checkout to running source code.

All commands are run from the repository root.

The workspace uses the Rust toolchain declared by the repository. The default
developer checks are:

- `cargo fmt --check`
- `cargo test -p rum-runtime`
- `cargo test --workspace`

For guide examples, the important binary is `rum-cli`.

The CLI can run a single expression, run a file, print syntax information, or
start the REPL.

## One Expression

Use `--eval` when you want one expression evaluated in a fresh session.

```text
$ cargo run -q -p rum-cli -- --eval 'Total[{1, 2, 1/2}]'
7/2
```

The output is the final expression after evaluation.

`Total` is a builtin. Its attributes and capability class are listed in the
[`Total`](../reference/builtins.md#total) reference entry.

## REPL

Running `cargo run -p rum-cli` with no arguments starts the prompt.

The REPL keeps one session alive, so definitions entered on one line are
available on later lines.

```text
$ printf 'square[x_] := x * x\nsquare[6]\n' | cargo run -q -p rum-cli
rum> Null
rum> 36
rum>
```

The first line stores a delayed downvalue for `square`.

The second line calls it.

`SetDelayed` returns `Null` when it stores the definition.

## Script Files

Use `--file` to evaluate a source file.

The file is evaluated in one session, and the CLI prints the final result.

```text
$ cat > /tmp/rum-guide-start.wl <<'EOF'
square[x_] := x * x
square[6]
EOF
$ cargo run -q -p rum-cli -- --file /tmp/rum-guide-start.wl
36
```

A positional path works too:

`cargo run -q -p rum-cli -- /tmp/rum-guide-start.wl`

The guide examples use `--file` because it makes the command form explicit.

## First Definitions

`Set` stores the evaluated right-hand side immediately.

`SetDelayed` stores the right-hand side unevaluated and evaluates it each time
the definition fires.

Use `=` for stable values and `:=` when the right-hand side should be evaluated
at call time.

Rules have a similar split:

- `lhs -> rhs` creates `Rule[lhs, rhs]`
- `lhs :> rhs` creates `RuleDelayed[lhs, rhs]`

Definitions are session state. A new `--eval` command starts a new CLI session,
so definitions from a previous command are not present.

Use the REPL or a file when you want multiple inputs to share definitions.

## Getting Help

The generated reference is the compact builtin catalog:

- one entry per builtin
- the builtin's summary
- its attributes
- its capability class
- implementation limit notes where the catalog declares one

For example, start with:

- [`Set`](../reference/builtins.md#set)
- [`SetDelayed`](../reference/builtins.md#setdelayed)
- [`Rule`](../reference/builtins.md#rule)
- [`RuleDelayed`](../reference/builtins.md#ruledelayed)
- [`Function`](../reference/builtins.md#function)
- [`List`](../reference/builtins.md#list)

The reference is not a tutorial. Use it to check exact names and metadata while
you use this guide for the working model.

## Parse And Format Tools

The CLI also exposes parser and formatter helpers.

Use `--parse` when you need to see the parsed syntax tree and tokens.

Use `--format` when you need the formatter's rendering for one expression.

These helpers are diagnostics for source text. They do not replace evaluation
examples in this guide.

## Fresh Sessions

Each `cargo run -q -p rum-cli -- --eval ...` command constructs a new CLI
session, installs the core builtins, evaluates the expression, prints the
result, and exits.

That makes `--eval` examples deterministic and isolated.

It also means stateful examples should be written as one compound expression,
as multiple REPL inputs, or as a script file.

## Messages

Runtime messages are printed after the value in CLI transcripts.

Do not assume that a message means the top-level evaluation failed. Many
message-producing expressions still return a value.

The evaluation chapter shows `Message`, `Quiet`, and `Check`.

## File Extensions

The CLI accepts ordinary text files.

The examples use `.wl` because it is a common source-file suffix in this
repository, not because the runtime requires that suffix.

`Get`, `Needs`, `Save`, and `DumpSave` also operate on text paths subject to the
host capability policy.

The scoping and resource chapters explain those boundaries.
