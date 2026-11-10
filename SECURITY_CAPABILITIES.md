# Tauri capability review

## Current frontend permissions

The `default` capability applies only to the local `main` window.

| Permission | Frontend access | Justification |
|---|---|---|
| `core:default` | Tauri core functionality | Required for normal Tauri IPC and window operation. |
| `dialog:allow-open` | User-mediated file/folder selection | Import and restore flows use the native picker. |
| `dialog:allow-save` | User-mediated destination selection | Export and backup flows use the native picker. |
| `shell:allow-open` | Open URLs or local paths using the OS handler | `openExternal.ts` validates HTTP(S) URLs and local paths; `email.ts` constructs a `mailto:` URI from the recipient. This does not grant shell command execution. |
| `notification:default` | Desktop notifications | Used for application notifications. |

The direct filesystem plugin permission and its broad APPDATA/Documents/Desktop/Downloads
scope were removed. Data import, export, and backup continue through named Rust commands,
which perform path validation in `apps/desktop/src-tauri/src/commands/path_guard.rs`.

## IPC boundary

Only commands registered in `src-tauri/src/lib.rs`'s `generate_handler!` are invokable.
Capabilities do not define a per-command allowlist; every registered command is part of
the frontend's reachable IPC surface. Each command must therefore validate its own
arguments and constrain filesystem/database operations. A complete audit of every
registered command was not performed as part of this focused permission change.

## Restrictions and follow-up

- No direct frontend shell execution permission is granted.
- No direct frontend filesystem read/write scope is granted.
- Native file dialogs remain enabled, and their returned paths still cross the IPC
  boundary; command-level validation remains mandatory.
- The import/export IPC commands can still read/write paths supplied through IPC.
  Their current guard rejects traversal and selected sensitive locations, but it does
  not constrain access to files chosen through a native dialog. A full command audit
  and a dialog-to-Rust redesign are still needed to prevent a compromised webview from
  invoking those commands with other user-readable paths.
- `shell:allow-open` is retained because the application uses it for external links.
  Keep URL validation at every call site and do not pass arbitrary user-controlled
  paths or commands to it.
- Review the registered IPC handlers whenever new commands are added. In particular,
  verify input-size limits, authorization assumptions, SQL parameterization, and path
  handling for all import/export/backup commands.
