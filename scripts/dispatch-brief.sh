#!/usr/bin/env bash
# Dispatch a brief for execution by a coding-agent worker.
#
#   scripts/dispatch-brief.sh <brief-id>
#
# <brief-id> may be a number (081), "brief-081", or a slug fragment; it resolves to
# docs/dev/codex/brief-<id>*.md. Runs the worker non-interactively (stdin from /dev/null)
# and writes the full report to /tmp/<worker>_brief_<num>.log.
#
# This does NOT touch git — commit the brief separately FIRST (keep commit and dispatch as
# two steps). Invoke this via the Bash tool with run_in_background:true so the run is
# harness-tracked; never append `&` to a dispatch.
#
# WORKER SELECTION (env BRIEF_WORKER, default codex)
#
#   BRIEF_WORKER=codex   `cdx exec` — Codex. The default worker.
#   BRIEF_WORKER=cgl     `cgl -p`   — claude-code on a GLM backend. EMERGENCY FALLBACK for
#                                     when Codex has no quota. Not the normal path.
#
# Both are zsh functions from ~/.zshrc (`with_proxy codex` / `with_proxy claude`), so neither
# is visible to a bash subshell — each runs through an interactive zsh, which sources ~/.zshrc
# (giving both the function and the proxy env). Redirections stay on the outer call so the
# worker inherits them.
#
# PRIVILEGE — the two workers fence differently, and they are NOT equivalent:
#
#   codex  Sandbox defaults to workspace-write (user ruling 2026-07-17: least privilege;
#          writes confined to the repo + tmp) and is an OS-level fence on WRITES, while
#          commands themselves are unrestricted. Override ONLY for briefs that truly need
#          process attach (the macOS `sample` profiler — the original 2026-06-21 reason for
#          full access):  CDX_SANDBOX=danger-full-access scripts/dispatch-brief.sh <id>
#
#   cgl    claude-code has NO OS-level write fence. The nearest equivalent is a tool-level
#          permission mode, so this script uses acceptEdits (file edits auto-approved) plus
#          an explicit --allowedTools list, which keeps the deny list in
#          ~/.claude-glm/settings.json (e.g. `Bash(rm:*)`) in force. That is a WEAKER fence
#          than codex's: writes are not confined to the repo by the OS. Escalate only when a
#          brief genuinely cannot proceed:
#            CGL_PERMISSION_MODE=bypassPermissions scripts/dispatch-brief.sh <id>
#          The brief's own Scope + Hard rules remain the inner fence for both workers.
#
# DRY_RUN=1 prints the resolved brief/worker/log and exits WITHOUT dispatching (for testing).
set -euo pipefail

usage() {
  echo "usage: scripts/dispatch-brief.sh <brief-id>   (e.g. 081, brief-081)" >&2
  echo "       BRIEF_WORKER=codex|cgl   (default codex)" >&2
  exit 2
}
[[ $# -eq 1 ]] || usage

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
id="${1#brief-}"   # strip a leading "brief-" if present

worker="${BRIEF_WORKER:-codex}"
case "$worker" in
  codex|cdx) worker=codex; log_tag=cdx ;;
  cgl|glm)   worker=cgl;   log_tag=cgl ;;
  *) echo "error: unknown BRIEF_WORKER '$worker' (expected codex or cgl)" >&2; exit 2 ;;
esac

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
log="/tmp/${log_tag}_brief_${num}.log"
task="Read $rel and execute it exactly. Follow its Hard rules."

echo "brief:   $rel"
echo "worker:  $worker"
echo "log:     $log"

if [[ -n "$(git -C "$repo_root" status --porcelain)" ]]; then
  echo "WARNING: working tree is not clean — the worker's changes will mix with existing ones" >&2
fi

if [[ "$worker" == codex ]]; then
  sandbox="${CDX_SANDBOX:-workspace-write}"
  echo "sandbox: $sandbox"
  cmd="cdx exec -C '$repo_root' -s $sandbox '$task'"
else
  permission_mode="${CGL_PERMISSION_MODE:-acceptEdits}"
  echo "permission-mode: $permission_mode (no OS-level write fence — see header)"
  # Keep the worker from adopting CLAUDE.md, which is the director's file, not the worker's.
  # NOTE: keep this text free of single quotes — it is embedded in a single-quoted zsh -ic
  # command below, and an apostrophe would terminate that string early.
  role="You are the implementation worker for this brief, not the project director. \
CLAUDE.md belongs to the director and is NOT your instruction file. \
Follow AGENTS.md and the brief. Do not commit, branch, or push; leave changes in the \
working tree. Report with verbatim evidence."
  # A single response is capped at CLAUDE_CODE_MAX_OUTPUT_TOKENS; exceeding it
  # kills the run and leaves nothing behind (brief-307 died that way at the
  # 32000 default). The cap belongs to cgl, not to this repo, so it lives in
  # ~/.claude-glm/settings.json; set CGL_MAX_OUTPUT_TOKENS only to override a
  # single dispatch.
  cgl_env=""
  if [[ -n "${CGL_MAX_OUTPUT_TOKENS:-}" ]]; then
    cgl_env="CLAUDE_CODE_MAX_OUTPUT_TOKENS=$CGL_MAX_OUTPUT_TOKENS "
    echo "max-output-tokens: $CGL_MAX_OUTPUT_TOKENS (overriding cgl default)"
  fi
  cmd="cd '$repo_root' && ${cgl_env}cgl -p '$task' \
--permission-mode $permission_mode \
--allowedTools Bash Edit Write Read Grep Glob \
--append-system-prompt '$role'"
fi

if [[ "${DRY_RUN:-0}" == "1" ]]; then
  echo "DRY_RUN: not dispatching"
  echo "would run: $cmd"
  exit 0
fi

set +e
zsh -ic "$cmd" < /dev/null > "$log" 2>&1
rc=$?
set -e
echo "$worker exit=$rc ; report in $log"
exit $rc
