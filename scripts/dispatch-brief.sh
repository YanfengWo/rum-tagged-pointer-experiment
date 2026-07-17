#!/usr/bin/env bash
# Dispatch a Codex brief for execution.
#
#   scripts/dispatch-brief.sh <brief-id>
#
# <brief-id> may be a number (081), "brief-081", or a slug fragment; it resolves to
# docs/dev/codex/brief-<id>*.md. Runs `cdx exec` non-interactively (stdin from /dev/null)
# and writes the full Codex report to /tmp/cdx_brief_<num>.log.
#
# This does NOT touch git — commit the brief separately FIRST (keep commit and dispatch as
# two steps). Invoke this via the Bash tool with run_in_background:true so the run is
# harness-tracked; never append `&` to a cdx dispatch.
#
# Sandbox level defaults to workspace-write (user ruling 2026-07-17: least privilege;
# writes confined to the repo + tmp). Override per dispatch ONLY for briefs that truly
# need process attach (the macOS `sample` profiler — the original 2026-06-21 reason for
# full access): CDX_SANDBOX=danger-full-access scripts/dispatch-brief.sh <id>
#
# DRY_RUN=1 prints the resolved brief/log and exits WITHOUT dispatching (for testing).
set -euo pipefail

usage() { echo "usage: scripts/dispatch-brief.sh <brief-id>   (e.g. 081, brief-081)" >&2; exit 2; }
[[ $# -eq 1 ]] || usage

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
id="${1#brief-}"   # strip a leading "brief-" if present

shopt -s nullglob
matches=("$repo_root"/docs/dev/codex/brief-"$id"*.md)
[[ ${#matches[@]} -eq 0 ]] && matches=("$repo_root"/docs/dev/codex/*"$id"*.md)
if [[ ${#matches[@]} -eq 0 ]]; then
  echo "error: no brief matching '$1' under docs/dev/codex/" >&2; exit 1
fi
if [[ ${#matches[@]} -gt 1 ]]; then
  echo "error: ambiguous '$1' matches:" >&2; printf '  %s\n' "${matches[@]}" >&2; exit 1
fi

brief="${matches[0]}"
rel="docs/dev/codex/$(basename "$brief")"
num=$(basename "$brief" | sed -E 's/^brief-([0-9]+).*/\1/')
log="/tmp/cdx_brief_${num}.log"
sandbox="${CDX_SANDBOX:-workspace-write}"

echo "brief:   $rel"
echo "log:     $log"
echo "sandbox: $sandbox"

if [[ -n "$(git -C "$repo_root" status --porcelain)" ]]; then
  echo "WARNING: working tree is not clean — Codex's changes will mix with existing ones" >&2
fi

if [[ "${DRY_RUN:-0}" == "1" ]]; then
  echo "DRY_RUN: not dispatching"
  exit 0
fi

# `cdx` is a zsh function from ~/.zshrc (`with_proxy codex`), so it is NOT visible to a
# bash subshell — run it through an interactive zsh, which sources ~/.zshrc (giving both the
# cdx function and the proxy env). Redirections stay on the outer call so codex inherits them.
set +e
zsh -ic "cdx exec -C '$repo_root' -s $sandbox 'Read $rel and execute it exactly. Follow its Hard rules.'" \
  < /dev/null > "$log" 2>&1
rc=$?
set -e
echo "cdx exit=$rc ; report in $log"
exit $rc
