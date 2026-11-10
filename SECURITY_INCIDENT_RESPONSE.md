# Git history exposure response

## Status and scope

The repository index contained `node_modules` paths. They are now removed from the
current index and excluded by `.gitignore`; the dependency folders were also removed
from the working tree. There are 8,395 staged path removals, while the current `HEAD`
still contains those paths. This does **not** remove old commits from Git history.
A filename scan did not establish that all historical blobs are free of secrets.
The redacted content scanner and history rewrite tools are not installed in this
environment. No history rewrite or remote push has been performed.

The cleanup script is `scripts/security/rewrite-history.sh`. It uses a separate
mirror clone, creates a pre-rewrite bundle, and does not rewrite the source checkout.
Its default mode scans only. Use `--rewrite` to produce a cleaned mirror; `--push`
requires a separate explicit confirmation of the exact remote URL.

## Incident checklist

- [ ] Freeze merges and tell repository collaborators that history may be rewritten.
- [ ] Preserve a restricted backup before rewriting; do not publish or attach it.
- [ ] Run `bash scripts/security/rewrite-history.sh <remote-url>` and retain the
      redacted scan report.
- [ ] Review suspicious historical paths and large blobs without printing secret
      contents into tickets, chat, or logs.
- [ ] Rotate every credential that may have appeared in any historical blob; deleting
      the blob does not revoke a credential.
- [ ] Run `bash scripts/security/rewrite-history.sh <remote-url> --rewrite`.
- [ ] Review the rewritten refs and rerun secret scanning before authorizing a push.
- [ ] Coordinate a maintenance window, then use `--rewrite --push` only with owner
      approval and a recovery plan.
- [ ] Verify remote branches/tags and secret scans after the rewrite.
- [ ] Require collaborators to re-clone or follow a coordinated reset procedure;
      do not merge old refs back into the cleaned history.
- [ ] Restrict or destroy the pre-rewrite backup only after the incident owner approves.

## Credential rotation procedure

1. Inventory tokens, API keys, passwords, signing keys, certificates, and deploy keys
   referenced by configuration or historical blobs. Treat any possibly exposed value
   as compromised; do not paste it into the incident record.
2. Revoke each credential at its issuing provider and issue a replacement with the
   narrowest required scope and a short expiry where supported.
3. Update local secret stores and deployment/CI secret stores; never add replacements
   to source control. Verify the application works using the replacement.
4. Rotate dependent credentials if a compromised credential could have granted access
   to them. Record owner, revocation time, and verification status without recording
   secret values.
5. Review provider access logs for suspicious use and preserve relevant evidence.

## Redacted incident record

- Incident owner:
- Detection date/time (UTC):
- Repository and affected refs:
- Suspected artifact categories:
- Secret scanner/version and redacted report location:
- Credential classes revoked (never list values):
- History rewrite operator and maintenance window:
- Backup location/access controls:
- Remote verification and collaborator re-clone status:
- Remaining risks and closure approval:

## Verification commands

Check current index and ignore behavior:

```bash
git ls-files | grep -E '(^|/)node_modules/'
git check-ignore -v --no-index node_modules/.probe apps/desktop/node_modules/.probe
```

List large blobs reachable from all refs (adjust the size threshold to policy):

```bash
git rev-list --objects --all |
  git cat-file --batch-check='%(objecttype) %(objectname) %(objectsize) %(rest)' |
  awk '$1 == "blob" && $3 > 52428800 { print $3, $4 }'
```

The listing identifies paths and sizes only; use a redacting secret scanner for blob
contents. Never print suspected secret values into terminal logs.
