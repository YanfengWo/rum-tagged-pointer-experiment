#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/rum-jupyter-home.sh <command> [args...]

Commands:
  init              Create ./jupyter_home, install Jupyter and Playwright.
  env               Print shell exports for the isolated environment.
  kernelspec        Build rum-jupyter-kernel and write the dev kernelspec.
  smoke [expr]      Run the protocol smoke through the isolated environment.
  lifecycle-smoke   Run a real jupyter_client lifecycle smoke.
  stdin-smoke       Run a real jupyter_client stdin suspend/resume smoke.
  kernel-smoke      Run lifecycle-smoke and stdin-smoke.
  browser-smoke     Run a headless JupyterLab browser smoke with Playwright.
  lab [args...]     Start JupyterLab with all state under ./jupyter_home.
  notebook [args...] Start Notebook with all state under ./jupyter_home.
  console [args...] Start jupyter-console with the rum dev kernel.
  paths             Print the important isolated paths.

Environment overrides:
  RUM_JUPYTER_HOME              Defaults to <repo>/jupyter_home.
  RUM_JUPYTER_BOOTSTRAP_PYTHON  Defaults to python3.
EOF
}

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "${script_dir}/.." && pwd)
home_dir=${RUM_JUPYTER_HOME:-"${repo_root}/jupyter_home"}
venv_dir="${home_dir}/venv"
workspace_dir="${home_dir}/workspace"
bootstrap_python=${RUM_JUPYTER_BOOTSTRAP_PYTHON:-python3}

ensure_dirs() {
  mkdir -p \
    "${home_dir}/jupyter/config" \
    "${home_dir}/jupyter/data" \
    "${home_dir}/jupyter/runtime" \
    "${home_dir}/jupyter/kernelspec" \
    "${home_dir}/xdg/cache" \
    "${home_dir}/xdg/config" \
    "${home_dir}/xdg/data" \
    "${home_dir}/pip/cache" \
    "${home_dir}/python/pycache" \
    "${home_dir}/python/userbase" \
    "${home_dir}/playwright/browsers" \
    "${home_dir}/playwright/user-data" \
    "${home_dir}/playwright/artifacts" \
    "${home_dir}/tmp" \
    "${home_dir}/logs" \
    "${workspace_dir}"
}

export_env() {
  export RUM_JUPYTER_HOME="${home_dir}"
  export VIRTUAL_ENV="${venv_dir}"
  export PATH="${venv_dir}/bin:${PATH}"
  export PYTHONNOUSERSITE=1
  export PYTHONDONTWRITEBYTECODE=1
  export PYTHONUSERBASE="${home_dir}/python/userbase"
  export PYTHONPYCACHEPREFIX="${home_dir}/python/pycache"
  export PIP_CACHE_DIR="${home_dir}/pip/cache"
  export JUPYTER_CONFIG_DIR="${home_dir}/jupyter/config"
  export JUPYTER_DATA_DIR="${home_dir}/jupyter/data"
  export JUPYTER_RUNTIME_DIR="${home_dir}/jupyter/runtime"
  export JUPYTER_PATH="${home_dir}/jupyter/kernelspec"
  export IPYTHONDIR="${home_dir}/ipython"
  export MPLCONFIGDIR="${home_dir}/matplotlib"
  export XDG_CACHE_HOME="${home_dir}/xdg/cache"
  export XDG_CONFIG_HOME="${home_dir}/xdg/config"
  export XDG_DATA_HOME="${home_dir}/xdg/data"
  export PLAYWRIGHT_BROWSERS_PATH="${home_dir}/playwright/browsers"
  export RUM_JUPYTER_PYTHON="${venv_dir}/bin/python"
  export RUM_JUPYTER_TMP_ROOT="${home_dir}/tmp"
  export TMPDIR="${home_dir}/tmp"
}

print_export() {
  local name=$1
  local value=$2
  printf 'export %s=%q\n' "${name}" "${value}"
}

print_env() {
  ensure_dirs
  print_export RUM_JUPYTER_HOME "${home_dir}"
  print_export VIRTUAL_ENV "${venv_dir}"
  print_export PATH "${venv_dir}/bin:${PATH}"
  print_export PYTHONNOUSERSITE "1"
  print_export PYTHONDONTWRITEBYTECODE "1"
  print_export PYTHONUSERBASE "${home_dir}/python/userbase"
  print_export PYTHONPYCACHEPREFIX "${home_dir}/python/pycache"
  print_export PIP_CACHE_DIR "${home_dir}/pip/cache"
  print_export JUPYTER_CONFIG_DIR "${home_dir}/jupyter/config"
  print_export JUPYTER_DATA_DIR "${home_dir}/jupyter/data"
  print_export JUPYTER_RUNTIME_DIR "${home_dir}/jupyter/runtime"
  print_export JUPYTER_PATH "${home_dir}/jupyter/kernelspec"
  print_export IPYTHONDIR "${home_dir}/ipython"
  print_export MPLCONFIGDIR "${home_dir}/matplotlib"
  print_export XDG_CACHE_HOME "${home_dir}/xdg/cache"
  print_export XDG_CONFIG_HOME "${home_dir}/xdg/config"
  print_export XDG_DATA_HOME "${home_dir}/xdg/data"
  print_export PLAYWRIGHT_BROWSERS_PATH "${home_dir}/playwright/browsers"
  print_export RUM_JUPYTER_PYTHON "${venv_dir}/bin/python"
  print_export RUM_JUPYTER_TMP_ROOT "${home_dir}/tmp"
  print_export TMPDIR "${home_dir}/tmp"
}

venv_python() {
  printf '%s\n' "${venv_dir}/bin/python"
}

require_venv() {
  if [[ ! -x "$(venv_python)" ]]; then
    echo "error: isolated venv is missing; run: scripts/rum-jupyter-home.sh init" >&2
    exit 1
  fi
}

cmd_init() {
  ensure_dirs
  if [[ ! -x "$(venv_python)" ]]; then
    "${bootstrap_python}" -m venv "${venv_dir}"
  fi
  export_env
  "$(venv_python)" -m pip install -U pip
  "$(venv_python)" -m pip install -U \
    jupyterlab \
    notebook \
    jupyter-console \
    jupyter-client \
    playwright
  "$(venv_python)" -m playwright install chromium
  cmd_kernelspec
}

cmd_kernelspec() {
  ensure_dirs
  export_env
  (
    cd "${repo_root}"
    cargo build -p rum-jupyter --bin rum-jupyter-kernel >/dev/null
  )
  "${script_dir}/write-rum-jupyter-kernelspec.sh" \
    "${JUPYTER_PATH}" \
    "${repo_root}/target/debug/rum-jupyter-kernel" >/dev/null
  printf 'kernelspec=%s\n' "${JUPYTER_PATH}/kernels/rum-dev/kernel.json"
}

cmd_smoke() {
  require_venv
  ensure_dirs
  export_env
  cmd_kernelspec >/dev/null
  "${script_dir}/smoke-rum-jupyter.sh" "${1:-1+1}"
}

cmd_kernel_smoke() {
  require_venv
  ensure_dirs
  export_env
  cmd_kernelspec >/dev/null
  "$(venv_python)" "${script_dir}/rum-jupyter-kernel-smoke.py" "$@"
}

cmd_browser_smoke() {
  require_venv
  ensure_dirs
  export_env
  cmd_kernelspec >/dev/null
  "$(venv_python)" "${script_dir}/rum-jupyter-browser-smoke.py" "$@"
}

cmd_lab() {
  require_venv
  ensure_dirs
  export_env
  cmd_kernelspec >/dev/null
  exec "${venv_dir}/bin/jupyter" lab \
    --no-browser \
    --ServerApp.open_browser=False \
    --ServerApp.root_dir="${workspace_dir}" \
    --LabApp.extension_manager=readonly \
    "$@"
}

cmd_notebook() {
  require_venv
  ensure_dirs
  export_env
  cmd_kernelspec >/dev/null
  exec "${venv_dir}/bin/jupyter" notebook \
    --no-browser \
    --ServerApp.open_browser=False \
    --ServerApp.root_dir="${workspace_dir}" \
    "$@"
}

cmd_console() {
  require_venv
  ensure_dirs
  export_env
  cmd_kernelspec >/dev/null
  exec "${venv_dir}/bin/jupyter-console" --kernel rum-dev "$@"
}

cmd_paths() {
  ensure_dirs
  cat <<EOF
home=${home_dir}
venv=${venv_dir}
workspace=${workspace_dir}
jupyter_config=${home_dir}/jupyter/config
jupyter_data=${home_dir}/jupyter/data
jupyter_runtime=${home_dir}/jupyter/runtime
jupyter_path=${home_dir}/jupyter/kernelspec
pip_cache=${home_dir}/pip/cache
playwright_browsers=${home_dir}/playwright/browsers
playwright_user_data=${home_dir}/playwright/user-data
playwright_artifacts=${home_dir}/playwright/artifacts
tmp=${home_dir}/tmp
logs=${home_dir}/logs
EOF
}

command=${1:-}
if [[ -z "${command}" || "${command}" == "-h" || "${command}" == "--help" ]]; then
  usage
  exit 0
fi
shift

case "${command}" in
  init) cmd_init "$@" ;;
  env) print_env "$@" ;;
  kernelspec) cmd_kernelspec "$@" ;;
  smoke) cmd_smoke "$@" ;;
  lifecycle-smoke) cmd_kernel_smoke lifecycle "$@" ;;
  stdin-smoke) cmd_kernel_smoke stdin "$@" ;;
  kernel-smoke) cmd_kernel_smoke all "$@" ;;
  browser-smoke) cmd_browser_smoke "$@" ;;
  lab) cmd_lab "$@" ;;
  notebook) cmd_notebook "$@" ;;
  console) cmd_console "$@" ;;
  paths) cmd_paths "$@" ;;
  *)
    usage >&2
    exit 2
    ;;
esac
