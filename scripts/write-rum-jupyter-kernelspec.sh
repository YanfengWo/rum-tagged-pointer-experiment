#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/write-rum-jupyter-kernelspec.sh <kernelspec-root> [kernel-bin]

Writes a development-only kernelspec at:
  <kernelspec-root>/kernels/rum-dev/kernel.json

Examples:
  scripts/write-rum-jupyter-kernelspec.sh /tmp/rum-jupyter-dev
  scripts/write-rum-jupyter-kernelspec.sh /tmp/rum-jupyter-dev /abs/path/to/rum-jupyter-kernel
EOF
}

if [[ $# -lt 1 || $# -gt 2 ]]; then
  usage >&2
  exit 2
fi

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "${script_dir}/.." && pwd)
kernelspec_root=$1
kernel_bin=${2:-"${repo_root}/target/debug/rum-jupyter-kernel"}
kernel_dir="${kernelspec_root}/kernels/rum-dev"

mkdir -p "${kernel_dir}"

cat > "${kernel_dir}/kernel.json" <<EOF
{
  "argv": ["${kernel_bin}", "{connection_file}"],
  "display_name": "rum (dev)",
  "language": "rum",
  "interrupt_mode": "message"
}
EOF

printf '%s\n' "${kernel_dir}/kernel.json"
