# rum-jupyter

`rum-jupyter` is the Jupyter protocol adapter for `rum`.

For `v0.1`, it should route kernel-facing request handling through `rum-host`
and provide a basic Notebook tech preview. Jupyter protocol, transport,
kernelspec, IOPub, and MIME concerns remain owned by this crate.

The current supported local path is the isolated `jupyter_home/` harness. It
keeps Jupyter config, runtime files, Python caches, Playwright browsers,
notebooks, logs, and temporary files under the repository checkout.

The migration status is tracked in
[Jupyter Host Migration Audit](../../docs/jupyter-host-migration-audit.md).

## Isolated Local Harness

```bash
scripts/rum-jupyter-home.sh init
scripts/run-bounded.py --timeout 180 -- scripts/rum-jupyter-home.sh kernel-smoke
scripts/run-bounded.py --timeout 180 -- scripts/rum-jupyter-home.sh browser-smoke
```

`kernel-smoke` runs real `jupyter_client` lifecycle, stdin, protocol, and
unsupported-message keepalive probes against the development kernel.
`browser-smoke` starts JupyterLab and drives a notebook with headless Chromium.

To start the same isolated JupyterLab environment manually:

```bash
scripts/run-bounded.py --timeout 3600 -- scripts/rum-jupyter-home.sh lab
```

In Codex-style sandboxes, the native ZeroMQ and Jupyter smoke commands need
permission to bind loopback ports. They still keep filesystem state under
`jupyter_home/`.

For a hand-driven console walkthrough and a curated set of manual expressions,
see [Jupyter Console Manual Smoke](../../docs/jupyter-console-manual.md).

## One-Shot Protocol Smoke

```bash
scripts/smoke-rum-jupyter.sh
```

Optional custom expression:

```bash
scripts/smoke-rum-jupyter.sh "Expand[(x + 1)^2]"
```

This older low-level smoke builds the local launcher, writes a temporary
kernelspec under `/tmp`, runs one `jupyter_client` execute round-trip, and
cleans up afterward. Prefer `scripts/rum-jupyter-home.sh kernel-smoke` for the
current acceptance harness.

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
jupyter_home/venv/bin/jupyter-console --kernel rum-dev --simple-prompt
```

Nothing is installed globally. Removing the temporary directories is enough to clean up.
