#!/usr/bin/env bash
# One-call rebuild of the Stable-mode mma-hol snapshot checkpoint chain with the
# CURRENT rum source, tagged by the git HEAD short hash so you never hand-track
# "v-numbers". Thin wrapper over scripts/snapshot-checkpoint-chain.sh that hardcodes
# this machine's paths.
#
#   bash scripts/rebuild-snapshot.sh [--force] [extra engine args...]
#
# What it does, zero-arg:
#   1. cargo build --release -p rum-cli  (so the binary == the working tree)
#   2. tag = <git short HEAD>[-dirty]    (dirty = uncommitted changes present)
#   3. build the chain into  $OUT_PARENT/rum-snapshot-checkpoints-<tag>/  (+ .log)
#   4. point  $OUT_PARENT/rum-snapshot-checkpoints-latest  at it (stable path —
#      so "the latest snapshot" is always findable without knowing the hash).
#
# Hardcoded defaults (override via env if ever needed):
#   RUM_MMA_HOL_ROOT     source mma-hol (pristine acceptance clone, never written)
#   RUM_SNAPSHOT_OUT_PARENT  parent dir for the chains (siblings of the repo)
#
# Why a script and not a memory note: a script is INVOKED (deterministic); a memory
# is RECALLED (probabilistic, and was once missed — see the feedback memory). The
# chain itself is built by snapshot-checkpoint-chain.sh on a scratch copy; the
# pristine acceptance clone stays read-only. Per-step timeout / --limit / etc. pass
# straight through to the engine.
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
mma_hol=${RUM_MMA_HOL_ROOT:-/Users/fft/Developments/mma-hol-acceptance}
out_parent=${RUM_SNAPSHOT_OUT_PARENT:-/Users/fft/Developments}
engine="${repo_root}/scripts/snapshot-checkpoint-chain.sh"

force=0
no_build=0
passthrough=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --force)    force=1; shift ;;
    --no-build) no_build=1; shift ;;   # use the existing target/release binary as-is
    *)          passthrough+=("$1"); shift ;;
  esac
done

[[ -x "${engine}" ]] || { echo "error: engine not found/executable: ${engine}" >&2; exit 2; }
[[ -f "${mma_hol}/tests/build_snapshot.wls" ]] || {
  echo "error: ${mma_hol} is not an mma-hol checkout (set RUM_MMA_HOL_ROOT)" >&2; exit 2; }

# (1) binary must match the source being snapshotted.
rum_cli="${repo_root}/target/release/rum-cli"
if [[ "${no_build}" == 0 ]]; then
  echo "[rebuild-snapshot] cargo build --release -p rum-cli ..."
  ( cd "${repo_root}" && cargo build --release -p rum-cli >/dev/null )
fi
[[ -x "${rum_cli}" ]] || { echo "error: rum-cli not built: ${rum_cli}" >&2; exit 2; }

# (2) HEAD-based tag; mark dirty so an uncommitted build is never mistaken for HEAD.
head=$(git -C "${repo_root}" rev-parse --short HEAD)
dirty=""
git -C "${repo_root}" diff --quiet --ignore-submodules HEAD || dirty="-dirty"
tag="${head}${dirty}"

out_dir="${out_parent}/rum-snapshot-checkpoints-${tag}"
log_file="${out_dir}.log"
latest_link="${out_parent}/rum-snapshot-checkpoints-latest"

if [[ -d "${out_dir}" && "${force}" != 1 ]]; then
  echo "[rebuild-snapshot] chain for ${tag} already exists: ${out_dir}"
  echo "[rebuild-snapshot] (use --force to rebuild it; pointing 'latest' at it)"
  ln -sfn "${out_dir}" "${latest_link}"
  exit 0
fi
[[ "${force}" == 1 ]] && rm -rf "${out_dir}"

echo "[rebuild-snapshot] HEAD=${tag}  source=${mma_hol}"
echo "[rebuild-snapshot] output=${out_dir}"
echo "[rebuild-snapshot] log=${log_file}"
[[ -n "${dirty}" ]] && echo "[rebuild-snapshot] WARNING: working tree is dirty — the tag is approximate."

# (3) build; the engine prints per-step '[NN] extend <module> ... ok (Ns, ...)'.
set +e
bash "${engine}" \
  --mma-hol "${mma_hol}" \
  --output-dir "${out_dir}" \
  --rum-cli "${rum_cli}" \
  "${passthrough[@]}" \
  > "${log_file}" 2>&1
rc=$?
set -e

# (4) stable 'latest' pointer regardless of partial/complete (a partial chain is
# still the newest artifact; the log + manifest say how far it got).
ln -sfn "${out_dir}" "${latest_link}"

echo "[rebuild-snapshot] engine exit=${rc} (0=complete, 1=stuck/timeout, 2=usage)"
echo "[rebuild-snapshot] deepest:" "$(tail -3 "${log_file}" | sed -n 's/.*deepest module reached: //p')"
echo "[rebuild-snapshot] chain:  ${out_dir}"
echo "[rebuild-snapshot] latest: ${latest_link} -> ${out_dir}"
exit "${rc}"
