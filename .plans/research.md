# Research: Multi-Model Ensemble Planning Baseline

Date: 2026-03-30

## Scope

Ground the implementation plan for the product described in `docs/requirements.md`, using `docs/v1_research.md` as the starting point and updating unstable assumptions against the current local environment and primary vendor docs.

## Current repo state

- The repo is effectively greenfield.
- Present source-of-truth documents:
  - `docs/requirements.md`
  - `docs/v1_research.md`
- No application source, tests, package manager files, or Rust manifests exist yet.
- No existing `.plans/` artifacts existed before this pass.

## What remains true from `docs/v1_research.md`

The core shape still looks correct:

- local-first desktop app
- Tauri desktop shell
- Rust orchestration core
- adapter per provider CLI
- canonical internal structured result before markdown rendering
- filesystem session store
- default read-oriented posture
- explicit context inclusion
- raw artifact preservation

This remains the lowest-risk architecture for v1.

## Newly verified local environment

Verified locally on 2026-03-30:

- `claude` at `/Users/seanflanagan/.local/bin/claude`
  - version `2.1.81`
  - `claude auth status` succeeds
- `codex` at `/opt/homebrew/bin/codex`
  - version `0.117.0`
  - `codex login status` succeeds
- `gemini` at `/opt/homebrew/bin/gemini`
  - version `0.35.3`

Implication:

- Claude and Codex are immediately viable spike targets on this machine.
- Gemini is installed, but its headless auth readiness cannot be verified with a standalone status command in the same clean way.

## Current CLI surface findings

### Claude Code

Verified from local help and official docs:

- Non-interactive mode is still `claude -p` / `--print`.
- Structured output is available with:
  - `--output-format json`
  - `--output-format stream-json`
  - `--json-schema`
- Perspective injection is available with `--system-prompt` and `--append-system-prompt`.
- Read-oriented permission control is available with `--permission-mode`.
- `--max-turns` exists and should be used to prevent unintended loops.
- `--allowedTools` exists and can narrow tool use.
- `--no-session-persistence` exists for print mode.
- `--bare` now explicitly states Anthropic auth is API-key based only in that mode, so it is still the wrong default for subscription-backed local usage.

Planning implication:

- Claude should run in normal `-p` mode, not `--bare`.
- Claude is suitable for both synthesis prose and schema-constrained extraction.

### Codex CLI

Verified from local help and official OpenAI docs:

- The correct non-interactive command remains `codex exec`.
- Structured automation surface is strong:
  - `--json`
  - `--output-last-message`
  - `--output-schema`
- `--skip-git-repo-check` exists and is needed for non-repo usage.
- `--add-dir` exists for additional writable directories.
- `--ephemeral` exists and is valuable for reducing provider-owned persistence.
- `--full-auto` is still a workspace-write plus on-request shortcut, not the safest default.
- Official config docs still define `approval_policy = "never"` and `sandbox_mode`.
- `codex login status` exits successfully when authenticated.
- Official auth docs still position ChatGPT sign-in as the default subscription path, with `codex login --device-auth` available for headless cases.

Planning implication:

- Codex should be driven with explicit sandbox and approval settings, not `--full-auto`.
- Codex is the strongest provider for event streaming and machine-readable run capture.

### Gemini CLI

Verified from local help and official docs:

- Non-interactive mode remains `gemini -p` / `--prompt`.
- Output formats include `text`, `json`, and `stream-json`.
- Approval control includes `--approval-mode` with `plan`, `default`, `auto_edit`, and `yolo`.
- `--include-directories` still exists in help.
- Official docs state headless mode uses cached credentials if present; otherwise environment-based auth is required.
- The known `--include-directories` bug is still a live planning concern.
- No obvious `auth status` or ephemeral/no-session-persistence flag is visible in the current help surface.

Planning implication:

- Treat Gemini as a first-class adapter, but not as a milestone-1 dependency.
- Context delivery should rely on app-level context packs plus CWD, not `--include-directories`.
- Cleanup guarantees must exclude provider-owned Gemini history/state.

## Important planning updates vs the prior research doc

### 1. Provider-owned persistence is a first-class risk

This was underemphasized before.

- Claude exposes `--no-session-persistence`.
- Codex exposes `--ephemeral`.
- Gemini does not expose an equivalent flag in current help.

Implication:

- App cleanup must only delete app-owned session directories.
- The product must never promise to clean all vendor-created history/state.
- The UI and docs should distinguish:
  - app artifacts we own and can safely delete
  - vendor-owned caches/history we do not own

### 2. Claude now also has schema-constrained output

This matters.

- The earlier research emphasized Codex as the schema-heavy option.
- Claude’s current CLI now also exposes `--json-schema`.

Implication:

- v1 can standardize around schema-guided extraction for both Claude and Codex where useful.
- The internal normalized schema is still required because provider metadata and event formats remain different.

### 3. Health probing and blocked-state handling should be milestone-1 work

Because auth, trust, cwd behavior, and provider-specific failure modes differ, the app needs:

- startup probe results
- per-provider availability status
- explicit blocked reasons
- operator-readable remediation guidance

This should not be deferred to late hardening.

### 4. Safe cleanup must be boundary-based

Safe deletion should mean:

- delete only directories under the app-managed session root
- archive by moving within that same root
- reject deletion paths outside the root after canonicalization
- never recursively delete user-selected context directories

## Architectural conclusions for the plan

### Recommended product boundary for v1

- Fan out prompts to installed local subscription-backed CLIs.
- Preserve raw evidence and normalized artifacts locally.
- Produce one structured brief per session.
- Default to read-oriented execution.
- Make context inclusion explicit and inspectable.
- Keep orchestration deterministic and debuggable.

### Recommended technical shape

- Tauri desktop shell
- React + TypeScript frontend
- Rust backend with `tokio::process::Command`
- explicit provider adapter trait
- app-owned session store under platform app-data
- canonical JSON artifacts before markdown rendering

This Rust-first backend remains an inference from the product’s needs and the current CLI behavior, not a direct requirement from vendor docs.

## Testing and tooling implications

Because the repo is empty, the plan should establish these from day one:

- frontend package manager: `pnpm`
- frontend type/lint/format: TypeScript + Biome
- frontend unit tests: Vitest
- desktop/e2e smoke tests: Playwright for Tauri flow checks
- Rust formatting/lint/tests: `cargo fmt`, `cargo clippy`, `cargo test`
- prefer fixture-based provider transcript tests so most coverage does not require live paid runs

Reasoning:

- This keeps the stack small.
- Biome reduces formatter+linter configuration overhead on a greenfield TS app.
- Rust and frontend test loops remain independently runnable.

## Risks inventory

### Product risks

- Scope creep into chat app behavior, follow-up handling, or autonomous coding.
- Overpromising cross-provider uniformity when the CLIs differ materially.
- Treating markdown as the source of truth instead of structured artifacts.

### Technical risks

- Provider CLI behavior changes between releases.
- Gemini headless/auth and context semantics remain weaker for unattended runs.
- Hidden provider side effects outside the app session store.
- Session cleanup bugs if path canonicalization is not strict.
- Synthesis quality degrading into fake consensus if disagreement handling is weak.

### Operational risks

- PATH differences when launching from Finder/Dock on macOS.
- Providers may be installed but not currently usable due to auth or trust state.
- Context packs may unintentionally include too much data unless manifesting is explicit.

## Planning recommendations

- Keep the milestone count low.
- Make the first milestone prove safe execution, provider probing, artifact persistence, and cleanup boundaries.
- Defer advanced prompting, interactive recovery, and broad provider parity work.
- Make live provider integration tests opt-in and sparse; rely mostly on recorded fixtures.

## Primary references

- `docs/requirements.md`
- `docs/v1_research.md`
- Claude Code CLI reference: https://code.claude.com/docs/en/cli-reference
- Claude Code headless docs: https://code.claude.com/docs/en/headless
- Claude Code permission modes: https://code.claude.com/docs/en/permission-modes
- Codex CLI reference: https://developers.openai.com/codex/cli/reference/
- Codex auth docs: https://developers.openai.com/codex/auth/
- Codex config reference: https://developers.openai.com/codex/config-reference/
- Codex sandboxing docs: https://developers.openai.com/codex/concepts/sandboxing/
- Gemini headless docs: https://google-gemini.github.io/gemini-cli/docs/cli/headless.html
- Gemini auth docs: https://google-gemini.github.io/gemini-cli/docs/get-started/authentication.html
- Gemini trusted folders docs: https://google-gemini.github.io/gemini-cli/docs/cli/trusted-folders.html
- Gemini `--include-directories` issue: https://github.com/google-gemini/gemini-cli/issues/13669
