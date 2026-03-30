# Research: Gemini CLI Headless/Non-Interactive Behavior

Date: 2026-03-30
Scope: Verify exact Gemini CLI behavior for unattended programmatic execution across seven specific questions.

---

## 1. Auth Probing

**Question:** Is there a `gemini auth status` or equivalent command?

**Status: VERIFIED — No such command exists.**

There is no `gemini auth status` or any standalone auth-check subcommand. The interactive REPL exposes a `/auth` slash command, but that is not usable from a script.

The documented and observed behavior in headless mode:
- If credentials are cached from a prior interactive login, the CLI uses them silently.
- If no suitable credentials are found, the CLI exits with a non-zero error in non-interactive mode.
- Exit code **41** maps to `FatalAuthenticationError`. This is the only programmatic signal available.

**Best practice for programmatic auth readiness check:**
Run a minimal no-op probe and check the exit code:

```sh
gemini -p "ping" --output-format json 2>&1; echo "exit:$?"
```

Exit 0 = credentials valid and usable. Exit 41 = auth failure. Any other non-zero = different fatal error.

For API-key-based auth: check that `GEMINI_API_KEY` is set. The CLI will print a clear error and exit non-zero if the variable is missing or invalid.

**Source:** https://google-gemini.github.io/gemini-cli/docs/troubleshooting.html (exit code table)
**Source:** https://google-gemini.github.io/gemini-cli/docs/get-started/authentication.html

---

## 2. `GEMINI_SYSTEM_MD` Environment Variable

**Question:** Does it work in headless `-p` mode? What is the exact behavior?

**Status: VERIFIED — Works in headless mode. Behavior is consistent across interactive and headless.**

Accepted values:
| Value | Behavior |
|-------|----------|
| `true` or `1` | Loads `./.gemini/system.md` relative to CWD |
| Absolute or relative file path | Loads that specific file |
| `~/path/to/file.md` | Tilde expansion is supported |
| `false`, `0`, or unset | Disables override; built-in prompt is used |

Key behavior:
- This is a **full replacement** of the built-in system prompt. Nothing from the default prompt is preserved unless manually included in the custom file.
- Missing file triggers error: `"missing system prompt file '<path>'"` — exits non-zero.
- The override mechanism is the same in headless (`-p`) and interactive mode. No special behavior in headless.
- Active override displays `|⌐■_■|` in interactive UI (irrelevant in headless).
- Dynamic variables `${AgentSkills}`, `${AvailableTools}`, and per-tool `${toolName}_ToolName` can be used in the custom file.
- To export the default prompt for reference before customizing: set `GEMINI_WRITE_SYSTEM_MD=1`.

**Source:** https://geminicli.com/docs/cli/system-prompt/ (canonical geminicli.com mirror of official docs)

**Note:** The official `google-gemini.github.io` headless doc does not list `GEMINI_SYSTEM_MD` directly. It is documented on geminicli.com and confirmed via web search. Treat as high-confidence but worth a live spike before building the Gemini adapter.

---

## 3. `--include-directories` Bug (Issue #13669)

**Question:** Is this still open/broken?

**Status: VERIFIED — Still open as of 2026-03-30.**

- Issue filed: 2025-11-22
- Labels: `area/core`, `priority/p1`, `status/need-triage`
- On 2026-01-28, it was briefly labeled as a duplicate of issue #16417 (`--include-directories flag does not work in sandbox mode on Linux/WSL`), then the duplicate label was removed — indicating the maintainers consider these distinct problems.
- The core problem: the agent behaves as if access is restricted solely to CWD, ignoring additional paths provided via `--include-directories`.

**Implication for adapter design:**
Do not rely on `--include-directories` for context delivery. Use CWD-relative context injection instead (write context to a temp directory under CWD, or use the app-managed session root as CWD for the invocation).

**Source:** https://github.com/google-gemini/gemini-cli/issues/13669

---

## 4. JSON Output Format

**Question:** What does `--output-format json` actually return? What is the schema?

**Status: VERIFIED — Schema documented.**

Flag: `--output-format json` (alias: `-o json`)
Also available: `--output-format stream-json` (streaming variant, see below).

**Standard JSON output schema (single object):**

```json
{
  "response": "<string — AI-generated answer>",
  "stats": {
    "models": {
      "<model-id>": {
        "requests": "<number>",
        "errors": "<number>",
        "latency": "<number>",
        "tokens": {
          "prompt": "<number>",
          "candidates": "<number>",
          "cached": "<number>",
          "thoughts": "<number>",
          "tool": "<number>"
        }
      }
    },
    "tools": {
      "totalCalls": "<number>",
      "success": "<number>",
      "fail": "<number>",
      "decisions": {
        "accept": "<number>",
        "reject": "<number>",
        "modify": "<number>",
        "auto_accept": "<number>"
      }
    },
    "files": {
      "additions": "<number>",
      "removals": "<number>"
    }
  },
  "error": {
    "type": "<string>",
    "message": "<string>",
    "code": "<string — optional>"
  }
}
```

`error` is only present on failure.

**Streaming JSON (`stream-json`) event types:**
`init`, `message`, `tool_use`, `tool_result`, `error`, `result`

Note: `stream-json` was in development as of earlier research (PR #10883). The CLI reference doc now lists it as a valid `--output-format` value, suggesting it is shipped, but behavior should be validated with a live spike before streaming is built into the adapter.

**Sources:**
- https://google-gemini.github.io/gemini-cli/docs/cli/headless.html
- https://geminicli.com/docs/cli/headless/
- https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/cli-reference.md

---

## 5. Session Persistence

**Question:** Is there a `--no-session-persistence` or `--ephemeral` equivalent?

**Status: VERIFIED — No such flag exists.**

Gemini CLI automatically saves all sessions to:
```
~/.gemini/tmp/<project_hash>/chats/
```

Where `<project_hash>` is derived from the project root directory. Sessions are project-scoped.

**No opt-out mechanism is documented or present in the CLI reference.** There is no `--no-session-persistence`, `--ephemeral`, `--no-save`, or equivalent flag.

Session management flags that do exist:
| Flag | Purpose |
|------|---------|
| `--resume` / `-r` | Resume a previous session by index, UUID, or "latest" |
| `--list-sessions` | List all sessions for current project and exit |
| `--delete-session <id>` | Delete a session by index or UUID |

Related settings in `settings.json` (not a flag, but configurable):
- `general.sessionRetention.enabled` (default: `true`) — auto-cleanup
- `general.sessionRetention.maxAge` (default: `"30d"`) — retention window
- `model.maxSessionTurns` (default: `-1`, unlimited)

**Implication for the adapter:**
Gemini will always write session history to `~/.gemini/tmp/`. The app cannot suppress this. The product must document this as vendor-owned state that is outside app cleanup scope.

**Sources:**
- https://geminicli.com/docs/cli/session-management/
- https://geminicli.com/docs/reference/configuration/
- https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/cli-reference.md

---

## 6. Sandbox and Approval Modes

**Question:** What does `--sandbox` or `--approval-mode` look like for read-only use?

**Status: VERIFIED.**

### `--sandbox` / `-s`

Enables sandboxed execution. Environment variable override: `GEMINI_SANDBOX`.

`GEMINI_SANDBOX` accepted values: `true`, `false`, `docker`, `podman`, `sandbox-exec`, `runsc`, `lxc`

Platform behavior:
| Platform | Default sandbox method |
|----------|----------------------|
| macOS | Seatbelt (`sandbox-exec`) |
| Linux | Docker, Podman, gVisor, or LXC |
| Windows | Native Sandbox via `icacls` |

macOS `SEATBELT_PROFILE` values: `permissive-open` (default — restricts writes outside project dir, allows most other ops), `restrictive-open`, `strict-open`, `strict-proxied`, or custom.

Additional sandbox configuration:
- `tools.sandboxAllowedPaths` — additional directories accessible inside sandbox
- `tools.sandboxNetworkAccess` (default: `false`) — network access toggle inside sandbox

### `--approval-mode`

Accepted values:
| Value | Behavior |
|-------|----------|
| `default` | Prompt for approval on each tool call |
| `auto_edit` | Auto-approve edit tools (write_file, replace); prompt for others |
| `yolo` | Auto-approve all tool calls |
| `plan` | Read-only mode (experimental — see below) |

`--yolo` / `-y` flag is deprecated; use `--approval-mode=yolo` instead.

### `--approval-mode=plan` for read-only use

This is the most relevant mode for unattended read-only execution.

What it does:
- Constrains the agent to read-only operations: file reads, directory listing, pattern matching, web search, internal docs.
- Blocks all write operations to production code.
- Allows writes only to `.md` files in designated plan directories.

**Critical headless caveat:** When exiting plan mode in a headless context, the CLI automatically transitions to YOLO mode (auto-approve all) to allow execution without hanging. This means a plan-mode session that proceeds to an execution phase will run entirely unattended with full write access. For a read-only adapter, either:
1. Use `--approval-mode=plan` and ensure the prompt never triggers an execution phase, or
2. Use `--approval-mode=default` with `--sandbox` for defense-in-depth.

Security settings in `settings.json`:
- `security.disableYoloMode` — blocks YOLO even if flagged at CLI
- `security.disableAlwaysAllow` — removes "allow always" option
- `security.toolSandboxing` (default: `false`) — experimental tool-level sandboxing

**Sources:**
- https://geminicli.com/docs/cli/sandbox/
- https://geminicli.com/docs/cli/plan-mode/
- https://geminicli.com/docs/reference/configuration/
- https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/cli-reference.md

---

## 7. Other Headless Flags Relevant to Unattended Execution

**Status: VERIFIED from CLI reference (PR #17504, merged).**

Complete headless-relevant flag surface:

| Flag | Alias | Default | Notes |
|------|-------|---------|-------|
| `--prompt` | `-p` | — | Triggers headless/non-interactive mode. Can be combined with stdin. |
| `--output-format` | `-o` | `text` | Values: `text`, `json`, `stream-json` |
| `--model` | `-m` | `auto` | Model aliases: `auto`, `pro`, `flash`, `flash-lite` |
| `--debug` | `-d` | `false` | Verbose logging to stderr; use to extract auth URL in headless SSH flows |
| `--all-files` | `-a` | — | Include all files in CWD as context |
| `--include-directories` | — | — | BROKEN — see item 3 |
| `--sandbox` | `-s` | `false` | Sandboxed execution |
| `--approval-mode` | — | `default` | Values: `default`, `auto_edit`, `yolo`, `plan` |
| `--allowed-mcp-server-names` | — | — | Comma-separated or multi-flag; restricts MCP server access |
| `--extensions` | `-e` | all | Load specific extensions only |
| `--version` | `-v` | — | Print version and exit (useful for availability probe) |

**Flags that do NOT exist (confirmed absent from reference):**
- `--non-interactive` — not a flag; non-interactive is inferred from `-p` or non-TTY
- `--no-session-persistence` — does not exist
- `--ephemeral` — does not exist
- `gemini auth status` — not a subcommand

**Auth readiness probe pattern (best available):**

```sh
# Lightweight: just check version resolves and is non-zero exit
gemini --version

# Stronger: actually test an API call
gemini -p "ok" --output-format json 2>/dev/null
# exit code 0 = auth OK; exit code 41 = FatalAuthenticationError
```

For API-key auth: simply check `[ -n "$GEMINI_API_KEY" ]` before invoking.

**Sources:**
- https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/cli-reference.md
- https://google-gemini.github.io/gemini-cli/docs/cli/headless.html
- https://google-gemini.github.io/gemini-cli/docs/get-started/authentication.html
- https://google-gemini.github.io/gemini-cli/docs/troubleshooting.html

---

## Summary Table

| Item | Finding | Confidence |
|------|---------|------------|
| `gemini auth status` | Does not exist. Best probe: run `-p "ok"`, check exit code 41 | High |
| `GEMINI_SYSTEM_MD` in headless | Works identically in headless; full replacement semantics | High (not in official headless doc; on geminicli.com mirror) |
| `--include-directories` bug #13669 | Still open, `p1`, unresolved as of 2026-01-28 | High |
| JSON output schema | `{response, stats{models,tools,files}, error?}` | High |
| Session persistence flag | No `--ephemeral` or equivalent; sessions always written to `~/.gemini/tmp/` | High |
| `--sandbox` values | `true/false/docker/podman/sandbox-exec/runsc/lxc` via `GEMINI_SANDBOX` | High |
| `--approval-mode` values | `default`, `auto_edit`, `yolo`, `plan` (plan is experimental + headless transitions to yolo) | High |
| Other headless flags | See full table above; no `--non-interactive` flag exists | High |

---

## Sources of Truth

| Area | Canonical Source | Drift Risk |
|------|-----------------|------------|
| Flag reference | https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/cli-reference.md | Medium — added via PR #17504, actively maintained |
| Headless mode | https://google-gemini.github.io/gemini-cli/docs/cli/headless.html | Medium — lags behind source |
| Auth | https://google-gemini.github.io/gemini-cli/docs/get-started/authentication.html | Low — stable |
| Exit codes | https://google-gemini.github.io/gemini-cli/docs/troubleshooting.html | Low |
| GEMINI_SYSTEM_MD | https://geminicli.com/docs/cli/system-prompt/ | Medium — community mirror, not official; verify |
| Sandbox | https://geminicli.com/docs/cli/sandbox/ | Medium |
| Plan mode | https://geminicli.com/docs/cli/plan-mode/ | High — labeled experimental |
| Issue tracker | https://github.com/google-gemini/gemini-cli/issues | High — active project, issue resolution pace is variable |

**Verification method:** Run `gemini --version` to confirm installed version, then run `gemini --help` to diff the flag surface against this doc. The CLI is actively developed; re-verify before building the Gemini adapter.
