# Jupyter Console Manual Smoke

This page records the manual `jupyter-console` workflow for `rum-jupyter`
through the isolated `jupyter_home/` harness.

It is meant for local developer validation when we want to watch a real Jupyter
frontend talk to the kernel, but do not want to install a global kernelspec,
write Jupyter state outside the checkout, or keep a long-running notebook
server around.

This is not the full `v0.1` Notebook acceptance path. The `v0.1` target is a
`rum-host`-backed basic Notebook tech preview; see
[Jupyter Host Migration Audit](./jupyter-host-migration-audit.md). This
console smoke remains the fastest adapter sanity check before running the full
Notebook frontend path.

## Prerequisites

- an initialized local harness:

```bash
scripts/rum-jupyter-home.sh init
```

- no global Jupyter installation changes are required
- all Jupyter config, runtime state, caches, logs, notebooks, and temporary
  files stay under `jupyter_home/`

## Fastest Path

If you only want the acceptance smoke checks, run:

```bash
scripts/run-bounded.py --timeout 180 -- scripts/rum-jupyter-home.sh kernel-smoke
scripts/run-bounded.py --timeout 180 -- scripts/rum-jupyter-home.sh browser-smoke
```

Those commands:

- builds `rum-jupyter-kernel`
- writes the development kernelspec under `jupyter_home/`
- runs real `jupyter_client` lifecycle, stdin, protocol, and
  unsupported-message keepalive probes
- starts JupyterLab and drives a notebook with headless Chromium

In Codex-style sandboxes these commands need permission to bind local loopback
ports. Filesystem state still stays under `jupyter_home/`.

## Manual Console Flow

1. Ensure the isolated harness exists:

```bash
scripts/rum-jupyter-home.sh init
```

2. Start a console against the development kernelspec:

```bash
scripts/run-bounded.py --timeout 3600 -- scripts/rum-jupyter-home.sh console --simple-prompt
```

3. Exit the console when you are done. Nothing is installed globally.

For a Notebook frontend instead of a console, start JupyterLab with:

```bash
scripts/run-bounded.py --timeout 3600 -- scripts/rum-jupyter-home.sh lab
```

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

The isolated harness keeps its state by design. The main locations are:

```bash
scripts/rum-jupyter-home.sh paths
```

`browser-smoke` also writes its notebook, server log, and failure artifacts
under `jupyter_home/workspace`, `jupyter_home/logs`, and
`jupyter_home/playwright/artifacts`.

## Common Checks

If the console fails to start, verify:

- `jupyter_home/venv/bin/jupyter-console` exists
- `jupyter_home/venv/bin/python -c "import jupyter_client"` works
- `target/debug/rum-jupyter-kernel` exists
- `scripts/rum-jupyter-home.sh kernelspec` rewrites the development kernelspec

If the frontend starts but stalls during ready-handshake, the first thing to
re-check is whether `kernel_info_request` still emits the expected IOPub
`busy/idle` status pair.
