# Provider-Owned Persistence Boundaries

This document describes what each provider CLI does with session history outside of the app's control, and what the app can and cannot clean up.

## App-Owned Artifacts

The app stores all session data under its own session root:
- macOS: `~/Library/Application Support/multi-model-synthesizer/sessions/<session-id>/`

**The app owns and can safely delete/archive everything under this path.** This includes raw stdout/stderr, normalized JSON, evidence matrices, briefs, event logs, and session metadata.

## Provider-Owned Persistence

Each provider CLI may independently write data outside the app session directory. The app **does not** delete, modify, or manage this data.

### Claude Code

- **Flag used:** `--no-session-persistence`
- **Effect:** When this flag is set, Claude Code does not save the session to its internal history.
- **Residual state:** With `--no-session-persistence`, Claude should not persist session data. Some internal caches (model downloads, auth tokens) remain under `~/.claude/` but are not session-specific.
- **App behavior:** The app passes `--no-session-persistence` on every run to minimize provider-owned side effects.

### Codex CLI

- **Flag used:** `--ephemeral`
- **Effect:** When this flag is set, Codex CLI does not save the session to its local history.
- **Residual state:** With `--ephemeral`, Codex should not persist conversation history. Auth tokens and configuration remain under `~/.codex/` but are not session-specific.
- **App behavior:** The app passes `--ephemeral` on every run to minimize provider-owned side effects.

### Gemini CLI

- **Flag available:** None. There is no `--ephemeral`, `--no-session-persistence`, or equivalent flag.
- **Residual state:** Gemini CLI **always** writes session history to:
  ```
  ~/.gemini/tmp/<project_hash>/chats/
  ```
  Where `<project_hash>` is derived from the working directory. This behavior cannot be suppressed.
- **Auto-cleanup:** Gemini has a built-in retention policy (default: 30 days) configurable via `settings.json`:
  - `general.sessionRetention.enabled` (default: `true`)
  - `general.sessionRetention.maxAge` (default: `"30d"`)
- **Manual cleanup:** Users can delete specific sessions with `gemini --delete-session <id>` or clear all sessions for a project manually.
- **App behavior:** The app does not attempt to clean up Gemini's session history. Users should be aware that Gemini retains its own copy of conversation history under `~/.gemini/`.

## Summary

| Provider | Side-effect reduction | Residual persistence | App cleans up? |
|----------|----------------------|---------------------|----------------|
| Claude | `--no-session-persistence` | Minimal (auth/cache only) | No — not needed |
| Codex | `--ephemeral` | Minimal (auth/config only) | No — not needed |
| Gemini | None available | Full session history in `~/.gemini/tmp/` | No — out of scope |

## Design Principle

The app only deletes data it created, under its own session root. Provider-owned state is the provider's responsibility. The UI and documentation make this boundary clear rather than promising cleanup the app cannot deliver.
