# CLI Flag Verification Research
**Date:** 2026-03-30
**Purpose:** Verify exact flags and syntax for Claude Code CLI, Codex CLI, Gemini CLI, Tauri v2, and Rust tooling before implementation begins.

---

## 1. Claude Code CLI

### Current State

Verified against live docs at `https://code.claude.com/docs/en/cli-usage` (the canonical redirect from `https://docs.anthropic.com/en/docs/claude-code/cli-usage`).

Locally installed: v2.1.81 at `/Users/seanflanagan/.local/bin/claude` (`docs/v1_research.md:64`).

**Flags confirmed to exist (exact names):**

| Flag | Notes |
|------|-------|
| `-p` / `--print` | Non-interactive / print mode. Required for all headless use. |
| `--output-format` | Valid values: `text`, `json`, `stream-json`. Works only in print mode. |
| `--max-turns` | Print mode only. Accepts integer. Exits with error when limit reached. |
| `--permission-mode` | See permission mode section below. |
| `--system-prompt` | Replaces entire default system prompt. Works in both interactive and non-interactive modes. |
| `--allowedTools` | Tools that execute without permission prompts. Space-separated patterns. |
| `--dangerously-skip-permissions` | Equivalent to `--permission-mode bypassPermissions`. |
| `--tools` | Restricts which built-in tools Claude can use. Use `""` to disable all. |
| `--append-system-prompt` | Appends to default prompt (preserves built-in tool schema). |
| `--bare` | Minimal mode, skips CLAUDE.md etc. **Requires API key — incompatible with subscription auth.** |
| `--json-schema` | Get validated JSON output matching a JSON Schema (print mode only). |
| `--max-budget-usd` | Cost cap in dollars (print mode only). |
| `--no-session-persistence` | Prevents session save to disk (print mode only). |

### Permission Mode — Critical Finding

**`dontAsk` is confirmed valid.** The complete list of valid `--permission-mode` values (verified at `https://code.claude.com/docs/en/permission-modes`):

| Mode value | Behavior |
|------------|----------|
| `default` | Prompts for file edits and commands |
| `acceptEdits` | Prompts for commands only |
| `plan` | Read-only exploration, no edits |
| `auto` | Background classifier (requires Team plan + Sonnet 4.6/Opus 4.6) |
| `dontAsk` | Auto-denies everything not explicitly pre-allowed. Fully non-interactive. |
| `bypassPermissions` | All checks disabled. Containers/VMs only. |

`dontAsk` is the correct mode for locked-down unattended runs where `--allowedTools` pre-approves only the needed surface. It is never in the `Shift+Tab` cycle — it is headless-only.

### Validated Command (from research doc)

```bash
claude -p "prompt" --output-format json --permission-mode dontAsk --max-turns 1
```

This is **confirmed correct**. All four flags exist with those exact names and that syntax.

### Constraints

- `--bare` requires API key. Do not use with subscription auth (`docs/v1_research.md:141`).
- `--permission-mode auto` can abort non-interactive sessions when the classifier hits its block threshold (3 consecutive or 20 total blocks abort in `-p` mode, no user to prompt). Do not use for unattended runs.
- `--system-prompt` replaces the entire default prompt including tool schema (~11.6k tokens). For research-only runs this is intentional. For runs where tool access is needed, use `--append-system-prompt` instead.
- `-p` skips workspace trust dialog — only run in explicitly user-selected directories.

### Sources of Truth

- Canonical CLI reference: `https://code.claude.com/docs/en/cli-usage`
- Permission modes: `https://code.claude.com/docs/en/permission-modes`
- Verification method: Direct `WebFetch` of live docs, 2026-03-30.
- Drift risk: **Medium.** Flag names are stable but new modes may be added; `dontAsk` behavior is well-documented and unlikely to change meaning.

---

## 2. Codex CLI

### Current State

Verified against live docs at `https://developers.openai.com/codex/cli/reference` and `https://developers.openai.com/codex/config-advanced`.

Locally installed: v0.117.0 at `/opt/homebrew/bin/codex` (`docs/v1_research.md:65`).

**Flags confirmed to exist (exact names):**

| Flag | Notes |
|------|-------|
| `codex exec` / `codex e` | Non-interactive run command. Confirmed. |
| `--ask-for-approval` / `-a` | Valid values: `untrusted`, `on-request`, `never`. **This is the correct flag name.** |
| `-c key=value` / `--config key=value` | Override any config key. Repeatable. JSON parsing applied where applicable. |
| `--output-schema path` | Path to JSON Schema file; validates tool output. Confirmed. |
| `--output-last-message` / `-o path` | Write final assistant message to file. |
| `--sandbox` / `-s` | Values: `read-only`, `workspace-write`, `danger-full-access`. |
| `--full-auto` | Shortcut for `workspace-write` sandbox + `on-request` approvals. |
| `--dangerously-bypass-approvals-and-sandbox` / `--yolo` | Bypasses all approvals and sandboxing. |

### Critical Flag Correction

**`-c approval_policy="never"` is incorrect.** The research doc at `docs/v1_research.md:156` states:

> `--ask-for-approval never` does not exist in codex-cli 0.117.0. Use `-c approval_policy="never"` instead.

This is now **reversed**. Current docs confirm `--ask-for-approval never` (or `-a never`) **does exist** and is the correct flag. The `-c approval_policy="never"` form may work as a config override but is not the canonical CLI flag. Use `-a never` for non-interactive runs.

Correct command for fully unattended non-interactive use:

```bash
codex exec -a never -s read-only "prompt"
```

### developer_instructions — Confirmed Valid

`developer_instructions` **is a real config key** (confirmed in config reference at `https://developers.openai.com/codex/config-reference`):

> "Additional developer instructions injected into the session (optional)."

Usage:

```bash
codex exec -c developer_instructions="You are a skeptical code reviewer..." -a never "prompt"
```

This is the correct mechanism for perspective injection in Codex. It injects as a developer-role message, not a full system prompt replacement.

### Additional Config Keys for System Prompt Control

| Key | Notes |
|-----|-------|
| `developer_instructions` | Injects as developer-role message. Confirmed. |
| `model_instructions_file` | Path to file replacing built-in instructions instead of AGENTS.md. |
| `experimental_instructions_file` | Full replacement of main system prompt from file. Undocumented for CLI but functional per community reports. |

`--output-schema` was confirmed in the reference. Path should point to a JSON Schema file.

### Constraints

- `--full-auto` is not safe for read-oriented research — use explicit `-s read-only -a never` instead.
- `--skip-git-repo-check` is required for non-repo directories (still valid per v0.117.0 local install).
- Config hierarchy: CLI flags > profile > project `.codex/config.toml` > user `~/.codex/config.toml` > system > defaults.

### Sources of Truth

- CLI reference: `https://developers.openai.com/codex/cli/reference`
- Config reference: `https://developers.openai.com/codex/config-reference`
- Config advanced: `https://developers.openai.com/codex/config-advanced`
- Verification method: Direct `WebFetch` of live docs + web search, 2026-03-30.
- Drift risk: **High.** Codex CLI is actively developed (v0.117.0 already). Flags have changed before — `--ask-for-approval` was apparently added or made canonical after v1_research was written. Re-verify adapter before each major milestone.

---

## 3. Gemini CLI

### Current State

Verified against `https://google-gemini.github.io/gemini-cli/docs/cli/headless.html` and `https://geminicli.com/docs/cli/headless/`.

Locally installed: v0.35.3 at `/opt/homebrew/bin/gemini` (`docs/v1_research.md:66`). Not installed at time of original research but is now present.

**Flags confirmed to exist (exact names):**

| Flag | Notes |
|------|-------|
| `-p` / `--prompt` | Triggers headless mode. Both forms valid. |
| `--output-format` | Valid values: `text` (default), `json`. Confirmed. |
| `--yolo` / `-y` | Auto-approves all actions. |
| `--model` / `-m` | Specify model variant. |
| `--all-files` / `-a` | Include all files in context. |
| `--include-directories` | Add directories to context. **Still broken — issue #13669 silently ignores this.** |

**Does NOT exist:**
- `--non-interactive` flag — not present per current docs.

### Output Format

`--output-format json` is confirmed. The JSON output includes the model response, token usage stats, and API latency metrics. For JSONL streaming, a `--output-format stream-json` flag exists (per PR #10883 in `google-gemini/gemini-cli`), though it may be under active development.

Correct headless command:

```bash
gemini -p "prompt" --output-format json
```

### Auth for Headless Mode

The docs confirm headless mode uses credentials cached from a prior interactive Google-account login. No standalone `gemini auth status` CLI command exists — auth is checked inside an interactive session via `/auth`. This means:

- Auth state cannot be probed programmatically without an interactive session.
- The orchestrator must treat Gemini auth as "unknown until first headless run succeeds or fails."
- Auth failure surfaces as a non-zero exit code, not a structured error.

The `GEMINI_SYSTEM_MD` environment variable (for system prompt injection) is confirmed in `docs/v1_research.md:240`. This is the correct mechanism for perspective injection in Gemini (no `--system-prompt` flag exists).

### Constraints

- `--include-directories` is still broken (silently ignored). Use CWD or pipe content instead. (`docs/v1_research.md:169`)
- No `--non-interactive` flag. Headless is triggered by `-p` or non-TTY environment.
- Auth cannot be pre-validated via CLI command.
- `stream-json` format may not be stable yet (tracked in PR #10883).

### Sources of Truth

- Headless reference: `https://google-gemini.github.io/gemini-cli/docs/cli/headless.html`
- Mirror: `https://geminicli.com/docs/cli/headless/`
- GitHub repo: `https://github.com/google-gemini/gemini-cli`
- Verification method: Direct `WebFetch` + web search, 2026-03-30.
- Drift risk: **High.** Gemini CLI is actively developed. The `--include-directories` bug and streaming JSON support are moving targets. The `stream-json` format may stabilize and should be re-verified before Phase 1.

---

## 4. Tauri v2

### Current State

Verified against `https://v2.tauri.app/start/create-project/`.

**Project creation commands (all valid):**

```bash
npm create tauri-app@latest     # npm
pnpm create tauri-app           # pnpm
yarn create tauri-app           # yarn
sh <(curl https://create.tauri.app/sh)  # shell
cargo install create-tauri-app --locked && cargo create-tauri-app  # Cargo
```

The interactive wizard asks for: project name, frontend language, package manager, frontend framework.

**Supported frontend templates:**

- `vanilla` (HTML/CSS/JS without framework)
- Vue.js, Svelte, React, SolidJS, Angular, Preact
- Yew, Leptos, Sycamore (Rust-native)

**Official recommendation for simple projects:** The vanilla template is explicitly recommended as the starting point. For TypeScript specifically, the vanilla template with Vite handles TS natively.

**Recommendation for this project:** Vanilla TypeScript + Vite. Rationale:
- The UI is a control surface, not a component-heavy application (`docs/requirements.md:54`).
- No build-time framework overhead.
- Tauri's own docs recommend vanilla to learn fundamentals first.
- SolidJS is the lightest reactive framework option if reactivity becomes needed later — it has a first-class template and zero virtual DOM overhead.

### Constraints

- Tauri v2 is the current stable release. Tauri v1 is legacy — do not use `v1.tauri.app` docs.
- `npm create tauri-app@latest` pins to the latest v2 scaffolding. The `@latest` tag is important.
- Tauri apps launched from Finder/Dock do not inherit shell `PATH` — binary discovery must use `/bin/sh -lc "which ..."` (`docs/v1_research.md:405`).
- The Tauri shell plugin is not needed for subprocess management. Use `tokio::process::Command` directly from Rust backend code.

### Sources of Truth

- Create project: `https://v2.tauri.app/start/create-project/`
- Prerequisites: `https://v2.tauri.app/start/prerequisites/`
- Frontend config: `https://v2.tauri.app/start/frontend/`
- Verification method: Direct `WebFetch` of live v2 docs, 2026-03-30.
- Drift risk: **Low.** Tauri v2 is stable. The `create-tauri-app` utility and template list is unlikely to change significantly.

---

## 5. Rust Tooling

### Current State

All tooling is standard Rust ecosystem — no external dependencies required.

**Linting and formatting:**

| Tool | Status | Usage |
|------|--------|-------|
| `clippy` | Bundled with `rustup` | `cargo clippy -- -D warnings` |
| `rustfmt` | Bundled with `rustup` | `cargo fmt --check` (CI) / `cargo fmt` (apply) |

Both are first-party tools installed via `rustup component add clippy rustfmt`. No additional setup needed.

**Test framework:**

Built-in `cargo test`. No external framework needed for unit/integration tests. Use `#[cfg(test)]` modules for unit tests and `tests/` directory for integration tests.

**Async subprocess management:**

`tokio::process::Command` is the standard approach. Key API points verified against `https://docs.rs/tokio/latest/tokio/process/struct.Command.html`:

- Imitates `std::process::Command` interface but returns futures.
- `stdin`/`stdout`/`stderr` default to inherited from parent. For subprocess capture, use `.stdout(Stdio::piped())` etc. before spawning.
- `.output()` unconditionally configures stdout/stderr as pipes.
- `.spawn()` + `.wait_with_output()` allows streaming while capturing.
- Avoid dropping `Child` handle before awaiting — runtime attempts cleanup on best-effort basis only.

For this project's use case (streaming stdout from long-running CLI invocations), the pattern is:

```rust
let mut child = tokio::process::Command::new("claude")
    .args(["-p", &prompt, "--output-format", "json"])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;
// read from child.stdout as AsyncBufRead
```

Optional: `tokio-process-tools` crate provides real-time output inspection, pattern matching, and graceful termination with signal escalation — worth evaluating for Phase 1 if raw tokio::process proves unwieldy.

### Constraints

- `tokio` must be added as a dependency with the `process` and `rt-multi-thread` features.
- Tauri v2 already includes `tokio` as a transitive dependency — check for version compatibility before adding a direct dep.
- For env sanitization (removing `ANTHROPIC_API_KEY`, `CODEX_API_KEY`, `GEMINI_API_KEY`), use `.env_remove(key)` on the `Command` builder before spawning (`docs/v1_research.md:346`).

### Sources of Truth

- tokio::process docs: `https://docs.rs/tokio/latest/tokio/process/`
- tokio tutorial: `https://tokio.rs/tokio/tutorial`
- clippy: `https://doc.rust-lang.org/clippy/`
- rustfmt: `https://rust-lang.github.io/rustfmt/`
- Verification method: Web search + docs.rs, 2026-03-30.
- Drift risk: **Low.** tokio's process API is stable. clippy and rustfmt are first-party stable tools.

---

## Summary: What Changed from v1_research.md

| Item | v1_research.md claim | Verified current state |
|------|---------------------|----------------------|
| Codex approval flag | `-c approval_policy="never"` (and `--ask-for-approval never` "does not exist") | `--ask-for-approval never` / `-a never` **is the correct canonical flag**. `-c approval_policy=...` may still work as a config override but is not the CLI flag. |
| Gemini `--non-interactive` | Not mentioned | Does not exist. Confirmed. Headless triggered by `-p` or non-TTY. |
| Claude `--permission-mode dontAsk` | Stated as valid | **Confirmed valid.** `dontAsk` is in the official mode table. |
| Claude `--system-prompt` | Stated as valid | **Confirmed valid.** Replaces entire system prompt. |
| Claude `--allowedTools` | Stated as valid | **Confirmed valid.** Space-separated tool patterns. |
| Codex `developer_instructions` | Stated as valid via `-c` | **Confirmed in official config reference** as "Additional developer instructions injected into the session." |
| Gemini `--output-format json` | Stated as valid | **Confirmed.** Valid values: `text`, `json`. `stream-json` in development (PR #10883). |
| Tauri project creation | Not in v1_research | `npm create tauri-app@latest` is the current command. Vanilla TS + Vite is recommended. |

## What Could Not Be Fully Verified

1. **Gemini `GEMINI_SYSTEM_MD` env var** — stated in v1_research but not confirmed in current headless docs. The headless reference does not cover system prompt injection at all. This should be validated with a live spike before the Gemini adapter is built.

2. **Codex `--skip-git-repo-check`** — stated in v1_research but not seen in current CLI reference. May have been renamed or removed. Verify with `codex exec --help` before use.

3. **Gemini `stream-json` output format** — referenced in PR #10883 but may not be in stable release v0.35.3. Check `gemini --help` for available output-format values.

4. **Claude version as of today** — v2.1.81 was the locally installed version as of 2026-03-29. The docs do not expose a current version number. Run `claude --version` to confirm.

---

## Sources

- [Claude Code CLI reference](https://code.claude.com/docs/en/cli-usage)
- [Claude Code permission modes](https://code.claude.com/docs/en/permission-modes)
- [Codex CLI reference](https://developers.openai.com/codex/cli/reference)
- [Codex config advanced](https://developers.openai.com/codex/config-advanced)
- [Codex config reference](https://developers.openai.com/codex/config-reference)
- [Gemini CLI headless (Google)](https://google-gemini.github.io/gemini-cli/docs/cli/headless.html)
- [Gemini CLI headless (geminicli.com mirror)](https://geminicli.com/docs/cli/headless/)
- [Gemini CLI GitHub](https://github.com/google-gemini/gemini-cli)
- [Gemini stream-json PR #10883](https://github.com/google-gemini/gemini-cli/pull/10883)
- [Tauri v2 create project](https://v2.tauri.app/start/create-project/)
- [tokio::process docs](https://docs.rs/tokio/latest/tokio/process/)
- [Codex CLI approval modes - SmartScope](https://smartscope.blog/en/generative-ai/chatgpt/codex-cli-approval-modes-no-approval/)
- [OpenAI Codex CLI cheat sheet](https://computingforgeeks.com/codex-cli-cheat-sheet/)
