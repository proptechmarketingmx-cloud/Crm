#!/usr/bin/env bash
set -Eeuo pipefail

usage() {
  cat <<'USAGE'
Usage: rewrite-history.sh <git-remote-url> [--rewrite] [--push]

Default: clone the remote to a clean-room directory, back it up, and scan history.
--rewrite: also remove dependency/build directories and likely secret/database files.
--push: force-push rewritten branches and tags after a redacted secret scan passes.
         Requires --rewrite and an interactive confirmation of the exact remote URL.

The source checkout and its working tree are never rewritten.
USAGE
}

if [[ $# -lt 1 ]]; then
  usage >&2
  exit 2
fi

remote_url=$1
shift
rewrite=false
push=false
for option in "$@"; do
  case "$option" in
    --rewrite) rewrite=true ;;
    --push) push=true ;;
    *) usage >&2; exit 2 ;;
  esac
done

if [[ "$push" == true && "$rewrite" != true ]]; then
  printf '%s\n' "--push requires --rewrite." >&2
  exit 2
fi
if ! command -v git-filter-repo >/dev/null 2>&1 && [[ "$rewrite" == true ]]; then
  printf '%s\n' "git-filter-repo is required for --rewrite." >&2
  exit 1
fi
if ! command -v gitleaks >/dev/null 2>&1; then
  printf '%s\n' "gitleaks is required to scan history before and after rewriting." >&2
  exit 1
fi

workspace=$(mktemp -d "${TMPDIR:-/tmp}/900crm-history-clean.XXXXXX")
mirror="$workspace/repository.git"
backup="$workspace/pre-rewrite.bundle"
printf 'Clean-room directory: %s\n' "$workspace"

git clone --mirror "$remote_url" "$mirror"
git -C "$mirror" bundle create "$backup" --all

printf '%s\n' "Scanning original history (findings are redacted):"
original_scan_status=0
gitleaks git "$mirror" --redact --no-banner || original_scan_status=$?
if [[ "$original_scan_status" -gt 1 ]]; then
  printf '%s\n' "Secret scan failed; review the scanner error before proceeding." >&2
  exit "$original_scan_status"
fi

if [[ "$rewrite" != true ]]; then
  if [[ "$original_scan_status" -eq 0 ]]; then
    printf '%s\n' "Audit complete. No refs were rewritten or pushed."
    exit 0
  fi
  printf '%s\n' "Potential secrets found. No refs were rewritten or pushed." >&2
  exit 1
fi

git -C "$mirror" filter-repo --force --invert-paths \
  --path-regex '(^|/)(node_modules|target)(/|$)|(^|/)\.env(\..*)?$|(^|/)[^/]*\.(sqlite|sqlite3|db)(-[^/]*)?$|(^|/)(id_rsa|id_dsa|id_ecdsa|id_ed25519|credentials)(\.[^/]*)?$'

printf '%s\n' "Scanning rewritten history (findings are redacted):"
rewritten_scan_status=0
gitleaks git "$mirror" --redact --no-banner || rewritten_scan_status=$?
if [[ "$rewritten_scan_status" -ne 0 ]]; then
  printf '%s\n' "Rewritten history still contains findings or the scan failed; no push performed." >&2
  exit "$rewritten_scan_status"
fi

printf 'Rewritten clean-room repository: %s\n' "$mirror"
printf 'Pre-rewrite backup bundle: %s\n' "$backup"
printf '%s\n' "Review the rewritten refs and coordinate with all collaborators before any push."

if [[ "$push" != true ]]; then
  printf '%s\n' "No push performed. Re-run with --push only after review and credential rotation."
  exit 0
fi

printf 'Type "PUSH AFTER ROTATION: %s" to confirm force-pushing branches and tags: ' "$remote_url"
IFS= read -r confirmation
if [[ "$confirmation" != "PUSH AFTER ROTATION: $remote_url" ]]; then
  printf '%s\n' "Confirmation did not match; no push performed." >&2
  exit 1
fi

git -C "$mirror" remote add cleaned-origin "$remote_url"
git -C "$mirror" push --force cleaned-origin 'refs/heads/*:refs/heads/*'
git -C "$mirror" push --force cleaned-origin 'refs/tags/*:refs/tags/*'
printf '%s\n' "Force-push completed. Collaborators must re-clone or safely reset their local branches."
