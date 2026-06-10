# Jupyter Console Manual Smoke

This page records the manual `jupyter-console` workflow for `rum-jupyter`.

It is meant for local developer validation when we want to watch a real Jupyter
frontend talk to the kernel, but do not want to install a global kernelspec or
keep a long-running notebook server around.

This is not the full `v0.1` Notebook acceptance path. The `v0.1` target is a
`rum-host`-backed basic Notebook tech preview; see
[Jupyter Host Migration Audit](./jupyter-host-migration-audit.md). Until that
path lands, this console smoke remains the fastest adapter sanity check.

## Prerequisites

- a working local Python environment with `jupyter-console` and
  `jupyter-client`
- a built `rum-jupyter-kernel` launcher
- no global Jupyter installation changes are required

In our current WSL setup, the default Python path is:

```text
~/.venv-jupyter/bin/python
```

The matching console is:

```text
~/.venv-jupyter/bin/jupyter-console
```

## Fastest Path

If you only want a quick end-to-end check, run:

```bash
scripts/smoke-rum-jupyter.sh
```

That one command:

- builds `rum-jupyter-kernel`
- writes a temporary kernelspec under `/tmp`
- starts a real Jupyter client round-trip
- executes one expression
- cleans up temporary files afterward

## Manual Console Flow

1. Build the development launcher:

```bash
cargo build -p rum-jupyter --bin rum-jupyter-kernel
```

2. Generate a temporary kernelspec root:

```bash
scripts/write-rum-jupyter-kernelspec.sh /tmp/rum-jupyter-dev
```

3. Start a temporary console:

```bash
JUPYTER_PATH=/tmp/rum-jupyter-dev \
JUPYTER_RUNTIME_DIR=/tmp/rum-jupyter-runtime \
~/.venv-jupyter/bin/jupyter-console --kernel rum-dev --simple-prompt
```

4. Exit the console when you are done. Nothing is installed globally.

## Suggested Manual Cases

Paste these one at a time in `jupyter-console`.

### Arithmetic

```text
1 + 1
Expand[(x + 1)^2]
```

Expected shape:

- a normal execute round-trip
- `In` / `Out` counters advance
- results appear as ordinary execute results

### Strings

```text
StringCases["abracadabra", Repeated["a", {2, Infinity}]]
TemplateApply[StringTemplate["Hello, `name`!"], <|"name" -> "Ada"|>]
```

Expected shape:

- string-pattern matching round-trips through the frontend
- association-driven template filling works in notebook-style execution

### Associations

```text
assoc = <|"a" -> 1, "b" -> 2|>
Lookup[assoc, "b"]
Keys[assoc]
```

Expected shape:

- association syntax parses through the Jupyter request path
- scalar keyed lookup returns `2`
- key-list rendering is stable

### Control Flow

```text
sum = 0; Do[sum = sum + i, {i, 3}]; sum
Catch[Throw[1, tag], tag]
```

Expected shape:

- compound expressions and loops execute as one request
- `Catch` / `Throw` survive the adapter boundary cleanly

### Stdin

```text
InputString["Name: "]
```

Expected shape:

- the frontend prompts for input
- submitting a reply resumes the suspended evaluation

## Keeping Temporary Files For Debugging

For the one-command smoke script, keep the temporary directories with:

```bash
RUM_JUPYTER_KEEP_TMP=1 scripts/smoke-rum-jupyter.sh
```

That is useful when we want to inspect:

- the generated kernelspec
- the Jupyter runtime directory
- the exact launcher path used in the smoke run

## Common Checks

If the console fails to start, verify:

- `~/.venv-jupyter/bin/jupyter-console` exists
- `~/.venv-jupyter/bin/python -c "import jupyter_client"` works
- `target/debug/rum-jupyter-kernel` exists
- the temporary kernelspec points at the expected launcher path

If the frontend starts but stalls during ready-handshake, the first thing to
re-check is whether `kernel_info_request` still emits the expected IOPub
`busy/idle` status pair.
