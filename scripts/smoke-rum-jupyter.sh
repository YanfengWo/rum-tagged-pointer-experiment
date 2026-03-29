#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/smoke-rum-jupyter.sh [expr]

Run a one-shot local Jupyter smoke test for rum-jupyter without installing any
global kernelspec. The optional expr defaults to:

  1+1

Environment overrides:
  RUM_JUPYTER_PYTHON      Python interpreter with jupyter_client installed
  RUM_JUPYTER_KERNEL_BIN  Path to rum-jupyter-kernel launcher
  RUM_JUPYTER_KEEP_TMP    Set to 1 to keep the temporary directories
EOF
}

if [[ $# -gt 1 ]]; then
  usage >&2
  exit 2
fi

if [[ ${1:-} == "-h" || ${1:-} == "--help" ]]; then
  usage
  exit 0
fi

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "${script_dir}/.." && pwd)
expr=${1:-"1+1"}
python_bin=${RUM_JUPYTER_PYTHON:-"${HOME}/.venv-jupyter/bin/python"}
kernel_bin=${RUM_JUPYTER_KERNEL_BIN:-"${repo_root}/target/debug/rum-jupyter-kernel"}
keep_tmp=${RUM_JUPYTER_KEEP_TMP:-0}

if [[ ! -x "${python_bin}" ]]; then
  echo "error: Python interpreter not found at ${python_bin}" >&2
  echo "hint: set RUM_JUPYTER_PYTHON or create ~/.venv-jupyter first" >&2
  exit 1
fi

if ! "${python_bin}" -c 'import jupyter_client' >/dev/null 2>&1; then
  echo "error: ${python_bin} cannot import jupyter_client" >&2
  echo "hint: install jupyter-client into the selected Python environment" >&2
  exit 1
fi

tmp_root=$(mktemp -d /tmp/rum-jupyter-smoke.XXXXXX)
kernelspec_root="${tmp_root}/kernelspec"
runtime_dir="${tmp_root}/runtime"

cleanup() {
  if [[ "${keep_tmp}" == "1" ]]; then
    echo "kept temporary files under ${tmp_root}" >&2
  else
    rm -rf "${tmp_root}"
  fi
}
trap cleanup EXIT

mkdir -p "${runtime_dir}"

(
  cd "${repo_root}"
  cargo build -p rum-jupyter --bin rum-jupyter-kernel >/dev/null
)

"${script_dir}/write-rum-jupyter-kernelspec.sh" "${kernelspec_root}" "${kernel_bin}" >/dev/null

export JUPYTER_PATH="${kernelspec_root}"
export JUPYTER_RUNTIME_DIR="${runtime_dir}"
export RUM_JUPYTER_SMOKE_EXPR="${expr}"

"${python_bin}" - <<'PY'
import os
from jupyter_client import KernelManager

expr = os.environ["RUM_JUPYTER_SMOKE_EXPR"]
km = KernelManager(kernel_name="rum-dev")
kc = None

try:
    km.start_kernel(env=os.environ.copy())
    kc = km.client()
    kc.start_channels()
    kc.wait_for_ready(timeout=30)
    kc.execute(expr)

    result = None
    seen = []
    idle_seen = False
    while not idle_seen:
        msg = kc.get_iopub_msg(timeout=30)
        msg_type = msg["header"]["msg_type"]
        seen.append(msg_type)
        if msg_type == "execute_result":
            result = msg["content"]["data"].get("text/plain")
        if msg_type == "status" and msg["content"].get("execution_state") == "idle":
            idle_seen = True

    shell = kc.get_shell_msg(timeout=30)
    print(f"expr={expr}")
    print(f"shell_status={shell['content'].get('status')}")
    print(f"execution_count={shell['content'].get('execution_count')}")
    print(f"iopub={','.join(seen)}")
    print(f"result={result}")
finally:
    if kc is not None:
        kc.stop_channels()
        kc = None
    if km.has_kernel:
        km.shutdown_kernel(now=True)
    km = None
PY
