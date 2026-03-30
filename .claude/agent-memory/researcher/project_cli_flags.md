---
name: verified CLI flags for provider adapters
description: Exact verified flag names and syntax for Claude, Codex, and Gemini CLIs as of 2026-03-30
type: project
---

Verified 2026-03-30 against live docs.

## Claude Code (v2.1.81)
- Non-interactive: `claude -p "prompt" --output-format json --permission-mode dontAsk --max-turns 1`
- `--permission-mode dontAsk` — auto-denies everything not pre-allowed, fully non-interactive
- `--system-prompt "..."` — replaces entire system prompt (strips tool schema ~11.6k tokens)
- `--append-system-prompt "..."` — appends to default (preserves tool schema)
- `--allowedTools "Read" "Bash(git log *)"` — space-separated tool patterns
- `--bare` — requires API key, incompatible with subscription auth. Do not use.
- `--permission-mode auto` — can abort non-interactive sessions at classifier block threshold. Do not use.

## Codex CLI (v0.117.0)
- Non-interactive: `codex exec -a never -s read-only "prompt"`
- `--ask-for-approval` / `-a never` — THIS IS THE CORRECT FLAG (not `-c approval_policy="never"`)
- `-c developer_instructions="..."` — injects as developer-role message (confirmed in config reference)
- `--output-schema path` — validates tool output against JSON Schema file
- `--sandbox` / `-s read-only` — values: read-only, workspace-write, danger-full-access

## Gemini CLI (v0.35.3)
- Non-interactive: `gemini -p "prompt" --output-format json`
- `-p` / `--prompt` — both trigger headless mode. No `--non-interactive` flag exists.
- `--output-format` / `-o` — values: `text` (default), `json`, `stream-json`. `stream-json` now listed in official CLI reference (PR #17504) but verify with live spike before building streaming.
- `--approval-mode` — values: `default`, `auto_edit`, `yolo`, `plan`. `plan` is experimental read-only mode; in headless contexts it auto-transitions to yolo on execution phase — do not assume it stays read-only.
- `--sandbox` / `-s` — enables sandboxing. `GEMINI_SANDBOX` env var: `true`, `docker`, `podman`, `sandbox-exec`, `runsc`, `lxc`. macOS uses Seatbelt by default.
- `--include-directories` — still broken (issue #13669, open as of 2026-01-28, priority/p1). Use CWD instead.
- Auth probing: no `gemini auth status` command. Best probe: `gemini -p "ok" --output-format json 2>/dev/null; echo $?`. Exit 41 = FatalAuthenticationError.
- `GEMINI_SYSTEM_MD` — CONFIRMED: works in headless mode. Values: `true/1` (loads .gemini/system.md), file path, or `false/0` (disabled). Full replacement semantics — no default prompt preserved. Missing file = non-zero exit with error message. Verified via geminicli.com/docs/cli/system-prompt/.
- No `--ephemeral` or `--no-session-persistence` flag. Sessions always written to `~/.gemini/tmp/<project_hash>/chats/`. Vendor-owned; cannot suppress from CLI.
- `--allowed-mcp-server-names` — restrict MCP server access (comma-separated or multi-flag).

**Why:** These flag details were verified directly and correct a stale claim in docs/v1_research.md (Codex approval flag). Accurate flags are critical for adapter implementation.
**How to apply:** Use these exact flags when building provider adapters. Re-verify Codex flags at Phase 1 (actively developed, version already at 0.117.0).
