# Milestone Plan: Multi-Model Research Synthesizer

## Contract

### 1. Problem

No tool exists to fan a research prompt across multiple local LLM CLIs (Claude Code, Codex CLI, Gemini CLI), apply varied analytical perspectives, and consolidate the results into one structured markdown brief — while preserving raw outputs for verification. Users must manually run each tool, mentally cross-reference outputs, and lose traceability.

### 2. Requirements

| # | Requirement | Priority |
|---|------------|----------|
| R1 | Accept a text prompt and optional directory/file context | P0 |
| R2 | Fan out prompt to selected CLI tools (Claude, Codex, Gemini) in parallel | P0 |
| R3 | Apply selectable perspective transformations before fan-out | P0 |
| R4 | Collect and preserve all raw CLI outputs to disk | P0 |
| R5 | Normalize raw outputs into a canonical structured format | P0 |
| R6 | Synthesize one consolidated markdown brief per run | P0 |
| R7 | Session-based filesystem storage (no database) | P0 |
| R8 | Tauri desktop UI as control surface | P0 |
| R9 | Provider health probing (installed, version, auth) | P0 |
| R10 | Selectable consolidation strategy (consensus, comprehensive, executive) | P1 |
| R11 | User-defined custom perspectives | P1 |
| R12 | Session list, archive, delete | P1 |
| R13 | Context manifest recording what was included/excluded | P1 |
| R14 | Custom consolidation templates | P2 |

### 3. Acceptance Criteria

- **Given** Claude, Codex, and Gemini are installed and authenticated, **when** the user submits a prompt with 2 perspectives and all 3 models, **then** 6 jobs run in parallel, raw outputs are saved, and one `brief.md` is produced.
- **Given** a session exists, **when** the user opens it in the UI, **then** they can view the brief, drill into each raw output, and see metadata (models, perspectives, timestamps, durations).
- **Given** a provider is not installed or not authenticated, **when** the user starts a run selecting it, **then** the UI shows a clear diagnostic and the run proceeds with available providers.
- **Given** sessions exist in the app-data directory, **when** the user deletes a session, **then** only files within that session's directory are removed — no files outside app-data are touched.

### 4. Non-Goals

- **No repo mutation.** v1 is read-only research. No code writing, no shell execution in user repos.
- **No API key management.** Subscription-backed CLI tools only. No BYOK.
- **No chat/conversation mode.** Artifact generation is the primary workflow, not multi-turn chat.
- **No cross-session continuity.** Each run is independent.
- **No LLM-generated perspectives.** Deterministic templates in v1 for reproducibility.
- **No Gemini parity guarantees.** Gemini is included as a first-class adapter, but its headless quirks (no standalone auth check, broken `--include-directories`) mean it gets workarounds where Claude/Codex have clean solutions. These workarounds are acceptable — not blockers.

### 5. Constraints

- **Local-first.** No hosted server, no database, no account with this product.
- **Subscription auth only.** Must strip `ANTHROPIC_API_KEY`, `CODEX_API_KEY`, `GEMINI_API_KEY` from spawned process environments to prevent accidental API billing.
- **macOS first.** Linux support is desirable but not blocking. Windows deferred.
- **Sessions in app-data.** `~/Library/Application Support/com.multimodel.synthesizer/sessions/` on macOS. Never inside user project directories.
- **Tauri v2.** Not v1.
- **Rust backend.** `tokio::process::Command` for subprocess management, not Tauri shell plugin.

---

## Implementation Plan

### 1. Summary

Build a Tauri v2 desktop app with a Rust async backend that orchestrates CLI-based LLMs through a provider adapter pattern. Each CLI gets its own adapter conforming to a shared trait. The orchestrator manages the job matrix (models × perspectives), runs jobs via `tokio::process::Command`, persists raw outputs, normalizes results into canonical JSON, and pipes them through a synthesis step that produces a structured markdown brief. The frontend is a thin control surface in vanilla TypeScript + Vite — no heavy framework, because the UI is forms + lists + markdown rendering, not a complex interactive app.

Core architectural decision: **adapter-based orchestration with post-execution normalization.** The three CLIs are too different to unify at the input layer. We normalize only after execution, giving each adapter full control over its CLI's quirks.

### 2. Current State

Greenfield. The repo contains only `docs/requirements.md` and `docs/v1_research.md`. No code, no configuration, no tests.

### 3. Milestone Outline

#### M1: Project Scaffold + Provider Spike

**Goal:** Tauri v2 project with working provider adapters that can probe and execute CLI tools.

This milestone delivers the foundation everything else builds on. It validates that we can actually run the CLIs programmatically and parse their output — the highest-risk assumption in the entire project.

**What gets built:**
- Tauri v2 project scaffold (vanilla TS + Vite frontend, Rust backend)
- Rust project structure: `src-tauri/src/{main.rs, lib.rs, provider/, types/}`
- Provider trait: `ProviderAdapter` with `probe()` and `execute()` methods
- `ClaudeAdapter` — probe (binary discovery, version, auth check), execute (`claude -p` with JSON output)
- `CodexAdapter` — probe (binary discovery, version, auth check), execute (`codex exec` with approval policy)
- `GeminiAdapter` — probe (binary discovery, version, auth via test call), execute (`gemini -p` with JSON output)
- Shared types: `ProviderProbe`, `ProviderJobSpec`, `NormalizedRunResult`
- Binary discovery via `which` with fallback paths
- Environment sanitization (strip API keys from spawned process env)
- Integration tests that validate probe + execute against real CLIs
- Unit tests for output parsing, env sanitization, binary discovery
- `clippy` and `rustfmt` in CI-ready config
- `.gitignore`, `rust-toolchain.toml`, editor config

**Exit criterion:** `cargo test` passes. Can programmatically run `claude -p "What is 2+2?"`, `codex exec "What is 2+2?"`, and `gemini -p "What is 2+2?"` and get parsed structured responses without human intervention.

**Key decisions for the implementing agent:**
- Use `tokio::process::Command` with `.stdout(Stdio::piped())` and `.stderr(Stdio::piped())`
- Claude command: `claude -p "<prompt>" --output-format json --permission-mode dontAsk --max-turns 1 --allowedTools Read Grep Glob`
- Codex command: `codex exec -a never -s read-only "<prompt>"`
- Gemini command: `gemini -p "<prompt>" --output-format json`
- Strip ANSI escape codes from all CLI output before parsing
- Binary discovery: `which claude`, `which codex`, `which gemini` — store absolute paths
- Auth check: `claude auth status` (exit code), `codex login status` (exit code), Gemini: run lightweight probe call and check exit code (no standalone auth command)
- Gemini perspective injection: test `GEMINI_SYSTEM_MD=/path/to/file.md` env var during spike. Fallback: prepend perspective text to prompt.
- Timeout: default 120s per job, configurable
- All file writes go to a temp directory or app-data directory, never to the user's project

**CLI flag reference (verified 2026-03-30):**

Claude Code:
```
claude -p "<prompt>" \
  --output-format json \
  --permission-mode dontAsk \
  --max-turns 1 \
  --system-prompt "<perspective text>" \
  --allowedTools "Read" "Grep" "Glob"
```

Codex CLI:
```
codex exec \
  -a never \
  -s read-only \
  -c developer_instructions="<perspective text>" \
  "<prompt>"
```

Gemini CLI:
```
GEMINI_SYSTEM_MD=/path/to/perspective.md \
gemini -p "<prompt>" \
  --output-format json
# Fallback if GEMINI_SYSTEM_MD doesn't work: prepend perspective to prompt
```

#### M2: Orchestrator + Sessions

**Goal:** Run prompt × models × perspectives, persist everything to a session directory.

This milestone builds the orchestration layer. It takes user inputs (prompt, selected models, selected perspectives, optional context) and executes the full fan-out, saving all raw outputs in a structured session directory.

**What gets built:**
- Session manager: create session directory, write metadata, manage lifecycle
- Session directory layout per the spec (see research doc section "Session Layout")
- Perspective system: YAML template loading, built-in perspectives (`default`, `creative`, `adversarial`, `performance`, `devils-advocate`)
- Prompt assembly: base prompt + perspective instructions + context
- Context pack builder: selected files/directories → packaged context string with manifest
- Job supervisor: build job matrix, run with `tokio::JoinSet`, enforce concurrency limit (4), handle timeout/cancel
- Raw artifact persistence: stdout, stderr, invocation metadata per job
- Event logging: JSONL event stream to `logs/events.jsonl`
- Tauri commands (IPC): `run_session`, `get_providers` (exposed but UI not built yet)
- Tests: session creation, perspective loading, prompt assembly, job matrix construction, concurrent execution

**Exit criterion:** From a Rust test or Tauri command, submit a prompt with 2 models × 2 perspectives → 4 jobs run in parallel → session directory created with all raw outputs, invocation metadata, and event log.

**Key decisions for the implementing agent:**
- Session directory: `~/Library/Application Support/com.multimodel.synthesizer/sessions/<timestamp>_<slug>/`
- Use `dirs::data_dir()` crate for platform-appropriate app-data path
- Perspectives are `.yaml` files in `src-tauri/perspectives/` (built-in) and `~/.config/multimodel/perspectives/` (user custom)
- Context pack: concatenate selected file contents with path headers, produce a manifest listing included/excluded files with byte counts
- Concurrency: `tokio::sync::Semaphore` with 4 permits
- Job timeout: `tokio::time::timeout` wrapping each adapter execute call
- JSONL events: append-only, one JSON object per line, timestamp + event type + payload
- Session metadata (`session.json`): app version, session id, created timestamp, providers, perspectives, strategy, working directory, git info if available
- All paths in session metadata should be relative to the session directory (for portability)

#### M3: Normalization + Synthesis

**Goal:** Transform raw CLI outputs into a canonical schema, build evidence matrix, produce `brief.md`.

This milestone closes the core value loop. Raw outputs become structured data, structured data becomes a synthesized brief.

**What gets built:**
- Normalization pipeline: raw CLI JSON → `NormalizedRunResult` → per-run extraction (summary, claims, recommendations, caveats, open questions, confidence signals)
- Per-run extraction: use one of the available CLIs (default: Claude) to extract structured data from raw output
- Evidence matrix: group claims by theme, track which model/perspective supports/disputes each
- Consolidation strategies: `consensus` (default), `comprehensive`, `executive` — each is a prompt template
- Synthesis execution: feed evidence matrix + strategy template to a CLI (default: Claude) → get structured synthesis → render to `brief.md`
- Brief renderer: structured synthesis JSON → markdown with sections (consensus, disagreements, uncertainty, action items, source references)
- Full pipeline integration: prompt → fan-out → normalize → synthesize → brief.md
- Tests: normalization parsing, evidence matrix construction, brief rendering, end-to-end pipeline

**Exit criterion:** End-to-end: submit prompt → fan-out to 2 models × 2 perspectives → normalize all outputs → build evidence matrix → synthesize → session directory contains `brief.md` + all intermediate artifacts.

**Key decisions for the implementing agent:**
- Normalization is a two-step process: (1) parse raw CLI output format into `NormalizedRunResult`, (2) use a CLI to extract structured claims/recommendations from the text content
- Step 1 is deterministic code (JSON parsing). Step 2 is an LLM call — use Claude by default since it has the best JSON output support
- Evidence matrix is a JSON file (`synthesis/evidence-matrix.json`) — not an LLM product, built programmatically from normalized extractions
- Consolidation prompt templates are `.md` files in `src-tauri/strategies/`
- The synthesis LLM call uses `--json-schema` (Claude) or `--output-schema` (Codex) to constrain output shape
- Brief.md is rendered from the structured synthesis JSON by Rust code, not by the LLM — this ensures consistent formatting
- If the synthesis CLI call fails, the session should still be valid with raw outputs preserved — synthesis is a post-processing step, not a gate

#### M4: Desktop UI

**Goal:** Full Tauri frontend — compose runs, monitor progress, browse sessions, view briefs.

This milestone makes the tool usable by a human. Everything before this can be driven by tests and Tauri commands; this adds the visual control surface.

**What gets built:**
- Prompt composer: text input, optional context file/directory selector
- Provider selector: toggle which models to include, show probe status (installed, auth, version)
- Perspective selector: toggle perspectives, show descriptions
- Strategy selector: pick consolidation strategy
- Run button + progress view: show job matrix, per-job status (queued/running/done/failed/blocked), elapsed time
- Session browser: list sessions with metadata (date, models, perspectives, status), sort/filter
- Brief viewer: render `brief.md` with markdown formatting
- Raw output viewer: drill into individual run outputs
- Session actions: delete, archive (move to archive subdirectory)
- Tauri IPC: all remaining commands (`list_sessions`, `get_session`, `delete_session`, `archive_session`)
- Frontend tests: component rendering, IPC contract validation

**Exit criterion:** Full workflow from the desktop app: open app → see provider health → compose prompt → select models/perspectives → run → watch progress → view brief → drill into raw outputs → browse past sessions → delete a session.

**Key decisions for the implementing agent:**
- Vanilla TypeScript + Vite. No React, no SolidJS — the UI is simple enough to not need a framework. Forms, lists, markdown rendering.
- Use a markdown rendering library (e.g., `marked` or `markdown-it`) for brief display
- CSS: minimal, functional. Consider a small utility library like `pico.css` for baseline styling without a build step
- State management: simple event-driven pattern. Tauri events for progress updates during runs.
- File dialogs: use Tauri's built-in file dialog for context selection
- Session list: load from filesystem on demand, paginate if needed
- All IPC calls should have loading/error states in the UI
- Accessibility: semantic HTML, keyboard navigation, screen reader labels on interactive elements

### 4. Testing Strategy

**Every milestone must pass `cargo clippy`, `cargo fmt --check`, and `cargo test` before it is committed.**

| Milestone | Test Focus | Test Type |
|-----------|-----------|-----------|
| M1 | Binary discovery, env sanitization, output parsing, probe/execute contract | Unit + integration (real CLI calls) |
| M2 | Session creation/lifecycle, perspective loading, prompt assembly, job matrix, concurrency | Unit + integration |
| M3 | Normalization parsing, evidence matrix construction, brief rendering, pipeline | Unit + integration (real CLI synthesis) |
| M4 | IPC contract validation, frontend rendering | Rust-side IPC tests + manual UI testing |

Integration tests that call real CLIs should be behind a feature flag (`#[cfg(feature = "integration")]`) so `cargo test` is fast by default and `cargo test --features integration` runs the full suite.

Frontend: `npm run lint` (eslint) + `npm run typecheck` (tsc) at every milestone that touches frontend code.

### 5. Manual Setup Tasks

| Task | When Needed | Milestone Dependency |
|------|------------|---------------------|
| Install Rust toolchain (`rustup`) | Before any work | M1 |
| Install Node.js + npm | Before Tauri scaffold | M1 |
| Install Tauri prerequisites (Xcode CLT, etc.) | Before Tauri scaffold | M1 |
| Verify `claude` CLI is authenticated | Before integration tests | M1 |
| Verify `codex` CLI is authenticated | Before integration tests | M1 |
| Verify `gemini` CLI is authenticated (run `gemini -p "ping"` interactively once) | Before integration tests | M1 |
| Decide on app bundle identifier | Before first Tauri build | M1 |

### 6. Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| **CLI flags change between versions** | Medium | High — adapters break silently | Pin expected CLI versions in adapter code. Probe includes version check. Log warnings on version mismatch. Integration tests catch regressions. |
| **CLI produces unexpected output** (warnings, update prompts, ANSI codes) | High | Medium — parsing fails | Strip ANSI codes. Parse JSON output specifically (not raw text). Handle non-JSON prefix/suffix gracefully. Log raw output before parsing. |
| **Auth expires mid-run** | Medium | Medium — jobs fail | Per-job error handling with clear status. Session continues with remaining providers. UI shows which jobs failed and why. |
| **Subprocess zombies on app crash** | Low | Medium — resource leak | Use process groups. Register cleanup on Tauri app exit. Document `kill` command for stuck processes. |
| **Session directory cleanup deletes wrong files** | Very Low | Very High — data loss | All session operations are scoped to the app-data directory. Path traversal validation on all delete operations. Never accept user-provided paths for delete. |
| **Tauri v2 build issues on macOS** | Medium | Medium — blocks progress | Follow official setup guide exactly. Pin Tauri version. Document working toolchain versions. |
| **Synthesis LLM call produces poor-quality brief** | Medium | Medium — low user trust | Brief is rendered from structured data, not raw LLM prose. Strategy templates are tunable. Raw outputs always preserved as fallback. |
| **Rate limiting during parallel fan-out** | Low | Low — retry handles it | Concurrency limit (4 jobs). Exponential backoff on rate limit errors. |
| **Gemini CLI headless quirks** | Medium | Medium — adapter needs workarounds | Auth probe via lightweight test call (no standalone auth command). Context via CWD + context pack (not `--include-directories`). Perspective injection via `GEMINI_SYSTEM_MD` env var — verify in M1 spike, fall back to prompt prepend. |
| **Context pack too large for CLI input** | Medium | Medium — truncated/failed | Size limit on context pack (configurable, default ~50KB). File-tree summary instead of full file contents for large directories. Manifest records truncation. |

### 7. Open Questions

1. **Synthesis model preference.** Should the user pick which CLI does the synthesis, or should it always be Claude (best JSON output support)?
   - *Recommendation:* Default to Claude, make it configurable in settings (not per-run UI).

2. **Context pack format.** Should context be passed as prompt text, temporary files, or CWD manipulation?
   - *Recommendation:* Inline in prompt for small contexts (<10KB), temporary file referenced in prompt for larger contexts. Never mutate user directories.

3. **Session archival semantics.** Does "archive" mean move to a subdirectory, compress to `.tar.gz`, or just mark as archived in metadata?
   - *Recommendation:* Move to `archived/` subdirectory within app-data. Simple, reversible, inspectable.

4. **Frontend framework.** Is vanilla TS sufficient, or should we start with a lightweight framework (SolidJS, Preact)?
   - *Recommendation:* Start vanilla. The UI is simple enough. Add a framework only if complexity demands it during M4.

5. **Gemini perspective injection.** Does `GEMINI_SYSTEM_MD` actually work in headless mode?
   - *Recommendation:* Verify during M1 spike. If it doesn't work, fall back to prepending perspective text to the prompt (simple, reliable, slightly less clean).

---

## Appendix: Verified CLI Commands (2026-03-30)

### Claude Code (v2.1.81)

```bash
# Probe
which claude              # binary path
claude --version          # version string
claude auth status        # exit 0 = authenticated

# Execute
claude -p "<prompt>" \
  --output-format json \
  --permission-mode dontAsk \
  --max-turns 1 \
  --system-prompt "<perspective>" \
  --allowedTools "Read" "Grep" "Glob"
```

### Codex CLI (v0.117.0)

```bash
# Probe
which codex               # binary path
codex --version            # version string
codex login status         # exit 0 = authenticated

# Execute
codex exec \
  -a never \
  -s read-only \
  -c developer_instructions="<perspective>" \
  "<prompt>"
```

**Note:** The v1_research.md incorrectly claims `--ask-for-approval never` does not exist. It is the correct canonical flag (alias: `-a never`).

### Gemini CLI (v0.35.3)

```bash
# Probe
which gemini               # binary path
gemini --version           # version string
# Auth: no standalone command — probe via lightweight test call

# Execute
GEMINI_SYSTEM_MD=/path/to/perspective.md \
gemini -p "<prompt>" --output-format json
# If GEMINI_SYSTEM_MD doesn't work: prepend perspective to prompt
```

## Appendix: Research Corrections

The following items from `docs/v1_research.md` were found to be incorrect or unverifiable during verification research (2026-03-30):

1. **Line 156:** Claims `--ask-for-approval never` does not exist in Codex CLI 0.117.0. **Incorrect.** `--ask-for-approval never` (alias `-a never`) is the correct canonical flag per the official CLI reference.

2. **Line 240:** Claims `GEMINI_SYSTEM_MD` env var replaces system prompt. **Unverified** in current headless docs. Needs live testing before the Gemini adapter is built.

3. **Line 171:** Claims Gemini CLI is "not installed on this machine." **Outdated** — Gemini was verified as installed at `/opt/homebrew/bin/gemini` v0.35.3 during 2026-03-29 research.
