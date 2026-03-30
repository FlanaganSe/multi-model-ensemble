# Research: Multi-Model Research Synthesizer

## Executive Summary

The product is feasible. The right shape is a local desktop orchestrator with:

- a Tauri UI as a control surface,
- a Rust orchestration core,
- one adapter per provider CLI,
- a canonical internal JSON result format,
- a filesystem session store,
- and a synthesis pipeline that normalizes first and writes markdown second.

Key architectural conclusions:

1. Treat Claude Code, Codex CLI, and Gemini CLI as different execution environments with different auth, approval, workspace, and output semantics.
2. Normalize only after execution.
3. Default to read-oriented research and artifact generation, not repo mutation.

## Product Intent

The product is:

- local-first orchestration,
- subscription-backed CLI fan-out,
- perspective-based prompt expansion,
- markdown brief synthesis,
- raw artifact preservation,
- session-based filesystem storage,
- and a Tauri desktop control surface.

It is not:

- a hosted orchestration service,
- a LiteLLM broker,
- a BYOK-first API product,
- a chat app with session history as the primary UX,
- or a code-writing agent platform in v1.

"Local-first" means: no hosted orchestrator, no central server, no database, no account with this product. The vendor CLIs still call remote model services. The privacy boundary is: prompts go to model providers through their official tools, and all artifacts stay on disk.

## Feasibility

Feasible for v1 if scope is disciplined:

- headless CLI execution,
- adapter-based orchestration,
- deterministic prompt perspectives,
- raw artifact preservation,
- structured normalization,
- one final brief per session.

Becomes much riskier if v1 tries to promise:

- every prompt can run fully unattended,
- identical behavior across vendors,
- full write-enabled autonomy by default,
- or silent recovery from all auth/trust/prompt edge cases.

## Local Environment

Verified on this machine on 2026-03-29:

- `claude` installed at `/Users/seanflanagan/.local/bin/claude`, v2.1.81, subscription auth active
- `codex` installed at `/opt/homebrew/bin/codex`, v0.117.0, ChatGPT login active
- `gemini` not installed

Auth status checks:

- Claude: `claude auth status` — exits 0 if logged in, 1 if not
- Codex: `codex login status` — exits 0 if authenticated
- Gemini: no official status command; check `~/.gemini/oauth_creds.json` existence (weak — token may be expired)

Implications:

1. MVP must not hard-require Gemini.
2. Provider availability is a first-class runtime concept.
3. Validation spike targets Claude and Codex immediately.

## Architecture

### Tauri Frontend

The UI is a control surface, not the orchestration brain. It owns: prompt entry, context selection, provider/perspective/strategy selection, run monitoring, session browsing, brief viewing, and raw artifact drill-down.

### Rust Orchestration Core

The backend owns: provider discovery, version/auth probing, prompt expansion, context-pack construction, subprocess spawning, timeout/cancellation, output collection, normalization, synthesis triggering, and session persistence.

Use `tokio::process::Command` directly, not the Tauri shell plugin. This gives full control over process lifecycle, timeouts, env sanitization, and output capture without shell-plugin permission wiring.

### Provider Adapter Layer

One adapter per CLI: `ClaudeAdapter`, `CodexAdapter`, `GeminiAdapter`.

Each adapter exposes one stable contract to the orchestrator:

```
ProviderProbe {
  provider: "claude" | "codex" | "gemini"
  installed: bool
  executablePath?: string
  version?: string
  authReady: bool
  notes: string[]
}

ProviderJobSpec {
  sessionId: string
  provider: "claude" | "codex" | "gemini"
  prompt: string
  cwd?: string
  extraDirectories: string[]
  mode: "read_only" | "workspace_write" | "isolated_mutation"
  timeoutMs: number
  model?: string
}

NormalizedRunResult {
  provider: "claude" | "codex" | "gemini"
  status: "ok" | "error" | "timeout" | "blocked" | "cancelled"
  startedAt: string
  endedAt: string
  durationMs: number
  exitCode?: number
  finalText?: string
  usage?: { inputTokens, outputTokens, cachedTokens, toolCalls }
  metadata: Record<string, unknown>
}
```

The orchestrator works against this contract, not raw CLI outputs.

## Provider Constraints

### Claude Code

Strengths: strong non-interactive `-p` mode, JSON and streaming JSON output, explicit system-prompt support, good fit for synthesis.

Constraints:

- Do not use `--bare` — requires API key, incompatible with subscription auth.
- Do not use `auto` permission mode — classifier fallback can abort non-interactive sessions.
- Use `--permission-mode dontAsk` for unattended runs (denies anything not pre-approved).
- `--allowedTools` can pre-approve a narrow tool surface.
- `-p` skips the workspace trust dialog — only run in directories the user intentionally selected.
- `--max-turns 1` prevents agentic loops.

Default posture: normal `-p` mode, JSON output, `--system-prompt` for perspective injection, narrow tool scope, read-oriented.

### Codex CLI

Strengths: clear non-interactive `codex exec`, JSONL event stream, output-to-file, schema-constrained output (`--output-schema`), strong automation ergonomics.

Constraints:

- `--ask-for-approval never` does not exist in codex-cli 0.117.0. Use `-c approval_policy="never"` instead.
- `--full-auto` is not a safe default for read-oriented research — use explicit sandbox + explicit approval policy.
- `--skip-git-repo-check` is required for non-repo or scratch-directory usage.
- No `--system-prompt` flag. Use `-c developer_instructions="..."` to inject perspective text (appends to default system prompt).
- ChatGPT sign-in is the official auth path for local CLI usage.

### Gemini CLI

Strengths: headless `-p`, JSON output with rich stats, good metadata for session accounting.

Constraints:

- Headless mode only uses Google-account auth if credentials were already cached from an interactive login; otherwise requires env-var auth.
- `--include-directories` is broken — silently ignored (P1 issue #13669). Workaround: set CWD to target, or pipe content.
- Trusted Folders are off by default, but if enabled can block headless execution.
- Not installed on this machine.

Design as a first-class adapter but not a hard MVP dependency. Plan context delivery via app-level context pack plus CWD, not `--include-directories`.

## Architectural Rules

### 1. Adapter-based orchestration, not faux unification

The three CLIs differ on auth, approvals, workspace handling, output format, metadata richness, and trust/safety behavior. Normalize after execution, not before.

### 2. Read-only research mode as default

No repo edits, no shell execution, no silent escalation into write-enabled behavior.

### 3. Structured JSON first, markdown second

Canonical internal state is structured data. `brief.md` is rendered from normalized structured data, optionally refined by a final model pass for prose quality.

### 4. Deterministic perspectives in v1

Local templates, not an LLM pre-pass. Reproducible, debuggable, cheaper, testable.

### 5. Hierarchical synthesis

Per-run normalization → evidence extraction → claim grouping → final synthesis. Preserves disagreements instead of collapsing into fake consensus.

### 6. Aggressive raw evidence preservation

Always keep: raw stdout, raw stderr, raw JSON/JSONL, normalized structured output, final markdown render.

## Context Strategy

Vendors have different workspace semantics. Relying only on vendor-native context leads to divergence unrelated to model quality.

### Layer 1: App-level context pack

Session-local artifact referenced by every provider: selected repo root(s), file tree summary, selected file excerpts, optional README/architecture snippets, optional git metadata, context manifest.

### Layer 2: Vendor-native workspace

- Claude: CWD + `--allowedTools Read,Grep`
- Codex: `-C /path` + `--add-dir`
- Gemini: CWD only (`--include-directories` is broken)

Do not mutate the user's repo. Do not assume vendor-native workspace alone is sufficient. Do not silently include entire repos without a manifest.

Context inclusion is data export to remote model providers. The session must record what was included, excerpted, excluded, and why.

## Perspective System

Store perspectives as data (YAML templates):

```yaml
id: devil-advocate
label: Devil's Advocate
instructions: |
  Reframe the task from the strongest skeptical perspective.
  Search for weak assumptions, hidden risks, edge cases, and likely failure modes.
  Do not ask follow-up questions. Make explicit assumptions and continue.
```

Built-in: `default`, `creative`, `adversarial`, `performance`, `devils-advocate`.

Custom perspectives are user-defined template files stored locally and versioned with the session.

### Injection per CLI

- **Claude:** `--system-prompt "..."` — replaces entire system prompt, strips ~11.6k tokens of tool schema. Preferred for research-only runs.
- **Codex:** `-c developer_instructions="..."` — appends to default system prompt. No full-replacement flag.
- **Gemini:** `GEMINI_SYSTEM_MD=/path/to/file.md` env var — full replacement. To preserve defaults while adding perspective, export default first with `GEMINI_WRITE_SYSTEM_MD=1`, then combine into temp file.

## Orchestration

Job matrix: `selected models × selected perspectives`.

### Job states

`queued` → `running` → `completed` | `failed` | `timed_out` | `blocked` | `cancelled`

`blocked` captures: missing login, trust prompt required, unsupported flag, approval policy conflict, provider refusing task shape.

### Concurrency

Start with 4 concurrent jobs, configurable up to 6.

### Events

Persist to JSONL: `job.started`, `job.stdout.chunk`, `job.stderr.chunk`, `job.completed`, `job.failed`, `job.blocked`, `job.timed_out`, `synthesis.started`, `synthesis.completed`.

## Synthesis Pipeline

### Per-run extraction

Normalize each provider result:

```json
{
  "summary": "short synthesis of the run",
  "claims": [],
  "recommendations": [],
  "caveats": [],
  "openQuestions": [],
  "confidenceSignals": [],
  "source": { "provider": "codex", "perspective": "devils-advocate" }
}
```

### Cross-run evidence matrix

Group by claim/theme: which model/perspective supports, which disputes, evidence text, what remains uncertain.

### Consolidation strategies

Prompt templates, not hardcoded logic:

- **Consensus** — agreement, shared conclusions, next actions. Default.
- **Comprehensive** — full union of findings, grouped by theme.
- **Executive** — concise, decision-oriented, 1-page TL;DR.
- **Custom** — user-provided template.

### Brief structure

1. Consensus summary
2. Major disagreements
3. Uncertainty and evidence gaps
4. Action items
5. Source references to underlying runs

## Session Layout

```text
sessions/
  <timestamp>_<slug>/
    session.json
    brief.md
    prompts/
      base.md
      context-pack.md
      perspectives/
        default.md
        creative.md
        devils-advocate.md
    runs/
      claude/
        default/
          invocation.json
          stdout.json
          stderr.txt
          normalized.json
      codex/
        default/
          stdout.jsonl
          final.md
          normalized.json
    synthesis/
      evidence-matrix.json
      synthesis-prompt.md
      synthesis-raw.md
      synthesis-normalized.json
    logs/
      events.jsonl
```

Storage location: platform-appropriate app-data directory (`~/Library/Application Support/` on macOS), not inside the project repo.

### Session metadata

At minimum: app version, OS, session id, created timestamp, selected providers/perspectives, synthesis strategy, working directory, extra included directories/files, provider executable path/version, auth readiness snapshot, exit code, timeout flag, blocked reason, token/tool stats when available, git branch/commit/dirty flag.

## Security Boundary

**Do:**

- Launch the official installed CLIs
- Let each CLI manage its own auth/session/cache
- Sanitize spawned process environment — remove `ANTHROPIC_API_KEY`, `CODEX_API_KEY`, `GEMINI_API_KEY` unconditionally (a user was billed $1,818 from an env var override — claude-code #3040)
- Preserve outputs locally
- Make context inclusion explicit
- Default to read-oriented behavior

**Do not:**

- Extract or proxy vendor OAuth tokens
- Reimplement vendor login flows
- Silently reuse browser sessions
- Mutate the user's repo to inject control files
- Blur "research synthesizer" and "full autonomous coding agent" in v1

## MVP Scope

**Include:** provider probing/health, prompt input, context selection, provider/perspective selection, parallel fan-out, raw artifact preservation, normalization, synthesized brief generation, session browser, delete/archive.

**Defer:** LLM-generated perspective transformation, interactive follow-up rescue, repo mutation, cross-session continuity, semantic caching, embedded provider protocols beyond CLI.

## Implementation Phases

### Phase 0: Capability Spike

Deliver: provider detection, version/auth probes, one run per available provider, normalized result envelope.

Exit criterion: can programmatically run `claude -p` and `codex exec` and get structured responses without human intervention.

### Phase 1: Orchestrator Core

Deliver: session creation, prompt assembly, context pack builder, job supervisor, raw artifact persistence, timeout/cancel.

Exit criterion: 2 models × 2 perspectives from CLI → 4 raw outputs saved to a session directory.

### Phase 2: Normalization and Synthesis

Deliver: canonical normalized schema, evidence matrix, strategy templates, final `brief.md`.

Exit criterion: end-to-end prompt → fan-out → synthesize → session with brief.md.

### Phase 3: Desktop UI

Deliver: run composer, provider health surface, progress monitor, session browser, artifact viewer.

Exit criterion: full workflow (compose → run → view brief → drill into raw outputs) from the desktop app.

### Phase 4: Hardening

Deliver: blocked-state diagnostics, retry for transient failures, Gemini hardening, archive management, presets and profiles.

## Product Decisions

Settled for planning purposes:

1. **Gemini not required for MVP.** Ship with available providers only.
2. **v1 forbids repo mutation by default.** Avoids approval/sandbox edge cases.
3. **Synthesizer is selectable.** Default to Claude for narrative briefs. Consider Codex for schema-heavy extraction.
4. **Context is explicit.** Never "entire repo by default." Selected paths plus file-tree summaries.
5. **Sessions in app-data directory.** Platform-appropriate default with export/open support.
6. **Deterministic normalization, optional prose pass.** Traceability without sacrificing readability.
7. **Binary discovery at startup via `/bin/sh -lc "which ..."`**. Store absolute paths. Required — Tauri apps from Finder/Dock do not inherit shell PATH.

## References

- [docs/requirements.md](docs/requirements.md) — authoritative product spec
- [Claude Code CLI reference](https://code.claude.com/docs/en/cli-reference)
- [Claude Code permission modes](https://code.claude.com/docs/en/permission-modes)
- [Claude Code headless](https://code.claude.com/docs/en/headless)
- [Claude Code authentication](https://code.claude.com/docs/en/authentication)
- [Codex CLI reference](https://developers.openai.com/codex/cli/reference/)
- [Codex config reference](https://developers.openai.com/codex/config-reference/)
- [Codex authentication](https://developers.openai.com/codex/auth/)
- [Gemini CLI headless](https://google-gemini.github.io/gemini-cli/docs/cli/headless.html)
- [Gemini CLI authentication](https://google-gemini.github.io/gemini-cli/docs/get-started/authentication.html)
- [Gemini CLI trusted folders](https://google-gemini.github.io/gemini-cli/docs/cli/trusted-folders.html)
- [Gemini `--include-directories` bug: #13669](https://github.com/google-gemini/gemini-cli/issues/13669)
- [Tauri sidecar/shell docs](https://v2.tauri.app/develop/sidecar/)
