# rum-jupyter

`rum-jupyter` is the Jupyter protocol adapter for `rum`.

For `v0.1`, it should route kernel-facing request handling through `rum-host`
and provide a basic Notebook tech preview. Jupyter protocol, transport,
kernelspec, IOPub, and MIME concerns remain owned by this crate.

The current local smoke path still uses `jupyter-console` because it is quick
and does not require a long-running notebook server. A documented real-Notebook
deploy path is part of the migration work tracked in
[Jupyter Host Migration Audit](../../docs/jupyter-host-migration-audit.md).

## One-Command Smoke Test

```bash
scripts/smoke-rum-jupyter.sh
```

Optional custom expression:

```bash
scripts/smoke-rum-jupyter.sh "Expand[(x + 1)^2]"
```

By default the script uses `~/.venv-jupyter/bin/python`, builds the local
`rum-jupyter-kernel` launcher, writes a temporary kernelspec under `/tmp`, runs
one `jupyter_client` execute round-trip, and then cleans up the temporary
directories.

For a fuller hand-driven `jupyter-console` walkthrough and a curated set of
manual expressions, see
[Jupyter Console Manual Smoke](../../docs/jupyter-console-manual.md).

## Minimal Local Smoke Test

1. Build the development kernel launcher:

```bash
cargo build -p rum-jupyter --bin rum-jupyter-kernel
```

2. Generate a temporary kernelspec root:

```bash
scripts/write-rum-jupyter-kernelspec.sh /tmp/rum-jupyter-dev
```

This writes:

```text
/tmp/rum-jupyter-dev/kernels/rum-dev/kernel.json
```

3. Start a temporary Jupyter console against that kernelspec:

```bash
JUPYTER_PATH=/tmp/rum-jupyter-dev \
JUPYTER_RUNTIME_DIR=/tmp/rum-jupyter-runtime \
~/.venv-jupyter/bin/jupyter-console --kernel rum-dev --simple-prompt
```

Nothing is installed globally. Removing the temporary directories is enough to clean up.
