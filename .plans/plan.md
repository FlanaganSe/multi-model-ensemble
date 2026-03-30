# Plan: Multi-Model Research Synthesizer

## Contract

### 1. Problem

Build a local-first desktop product that sends one user prompt through multiple selected local subscription-backed model CLIs, applies selectable perspectives, preserves raw outputs, and produces one standardized synthesized brief per session.

The plan must optimize for:

- low scope creep
- controllability
- inspectability
- understandability
- safe cleanup boundaries
- modern tooling
- clear testing and buildability

### 2. Requirements

- Use `docs/requirements.md` as the product source of truth.
- Use `docs/v1_research.md`, `.plans/research.md`, and `.plans/research_gemini.md` as planning inputs.
- Support local CLI providers:
  - Claude Code
  - Codex CLI
  - Gemini CLI
- Accept:
  - prompt
  - optional file/directory context
  - selected providers
  - selected perspectives
- Generate one session per run with:
  - brief
  - raw outputs
  - metadata
- Support session list, archive, and delete.
- Default to read-oriented behavior.
- Avoid unsafe deletion outside app-owned storage.
- Make provider readiness and blocked states explicit.
- Be testable, lintable, and buildable from the start.

### 3. Acceptance criteria

- A developer can build and run the desktop app locally with documented prerequisites.
- The app can detect provider installation/auth readiness and surface blocked reasons.
- A run with selected providers and perspectives creates one app-owned session directory with:
  - raw provider artifacts
  - normalized artifacts
  - `brief.md`
  - metadata
- Session delete/archive operations are path-safe and affect only the app session root.
- Claude and Codex support end-to-end unattended runs in the first usable release.
- Gemini is either:
  - supported and verified, or
  - clearly marked experimental/blocked without breaking the main flow.
- `pnpm` lint/test/build and `cargo` fmt/clippy/test pass for the release branch.

### 4. Non-goals

- Hosted orchestration or cloud sync
- API-key-first provider integration
- Full autonomous code-writing workflows
- Automatic follow-up question handling across providers
- Cross-session memory or semantic caching
- Perfect behavioral parity across vendor CLIs
- Deleting or managing vendor-owned caches/history outside the app session root

### 5. Constraints

- The repo is currently greenfield.
- The product should stay local-first and filesystem-based.
- Subscription auth only. Strip `ANTHROPIC_API_KEY`, `CODEX_API_KEY`, `GEMINI_API_KEY` from spawned process environments to prevent accidental API billing.
- Provider CLIs are unstable external dependencies and may change.
- Cleanup safety matters more than aggressive convenience.
- The milestone count should stay as low as possible while preserving verifiability.

## Implementation plan

### 1. Summary

Build the product in 4 milestones:

1. Foundation and safe boundaries
2. Provider fan-out and artifact capture
3. Normalization and synthesis
4. Desktop workflow and hardening

This keeps the sequence simple while forcing the risky pieces early:

- provider readiness probing
- cleanup boundaries
- deterministic artifact formats
- test harnesses

### 2. Current state

- No application code exists yet.
- No frontend or Rust project structure exists yet.
- The planning baseline is strong enough to begin implementation without further discovery.
- Claude and Codex are currently the lowest-risk providers to target first on this machine.
- Gemini should be attempted during implementation with verified workarounds and safe fallbacks, but milestone completion should not depend on early Gemini parity.

### 3. Files to create

Recommended initial project shape:

- `package.json`
- `pnpm-lock.yaml`
- `tsconfig.json`
- `vite.config.ts`
- `biome.json`
- `playwright.config.ts`
- `src/main.tsx`
- `src/App.tsx`
- `src/features/run-composer/*`
- `src/features/session-browser/*`
- `src/features/artifact-viewer/*`
- `src/lib/api.ts`
- `src/lib/types.ts`
- `src/styles/*`
- `src-tauri/Cargo.toml`
- `src-tauri/build.rs`
- `src-tauri/tauri.conf.json`
- `src-tauri/src/main.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/commands/providers.rs`
- `src-tauri/src/commands/runs.rs`
- `src-tauri/src/commands/sessions.rs`
- `src-tauri/src/providers/mod.rs`
- `src-tauri/src/providers/claude.rs`
- `src-tauri/src/providers/codex.rs`
- `src-tauri/src/providers/gemini.rs`
- `src-tauri/src/providers/types.rs`
- `src-tauri/src/orchestrator/mod.rs`
- `src-tauri/src/orchestrator/context_pack.rs`
- `src-tauri/src/orchestrator/jobs.rs`
- `src-tauri/src/orchestrator/perspectives.rs`
- `src-tauri/src/orchestrator/synthesis.rs`
- `src-tauri/src/session_store/mod.rs`
- `src-tauri/src/session_store/layout.rs`
- `src-tauri/src/session_store/safe_paths.rs`
- `src-tauri/src/session_store/metadata.rs`
- `src-tauri/src/models/*.rs`
- `src-tauri/tests/*`
- `tests/fixtures/providers/*`
- `.gitignore`

### 4. Milestone outline

#### Milestone 1: Foundation and safe boundaries

- [x] Step 1 — Scaffold Tauri v2 + React + TS + Vite project with Biome, Vitest, cargo fmt/clippy/test → verify: `pnpm build`, `cargo check`
- [x] Step 2 — Implement session store: safe paths (canonicalize, reject outside root, symlink defense), layout, metadata schema v1 → verify: `cargo test`
- [x] Step 3 — Implement provider probing: binary discovery via `/bin/sh -lc "which ..."`, version check, auth check (Claude: `auth status`, Codex: `login status`, Gemini: `-p "ok"` exit code 41) → verify: `cargo test`
- [x] Step 4 — Wire Tauri commands and minimal UI: provider health cards, session list with create/delete → verify: `pnpm build`, `pnpm test`
- [x] Step 5 — Run full quality gate: biome check, vitest, tsc, cargo fmt/clippy/test → verify: all pass
Commit: "feat: milestone 1 — foundation and safe boundaries"

Drift notes:
- `gemini --version` and `gemini -v` both hang on this machine; probe must use tight timeout
- Tauri v2 requires `[lib]` section in Cargo.toml with `crate-type = ["staticlib", "cdylib", "rlib"]`
- Tauri v2 icons must be RGBA PNGs
- Biome must ignore `.pnpm-store` directory

#### Milestone 2: Provider fan-out and artifact capture

Goal:

- Execute real unattended provider runs and persist raw evidence reliably.

Scope:

- Implement deterministic perspective templates.
- Build context-pack generator and manifest.
- Implement job matrix expansion: providers × perspectives.
- Implement async job supervisor with queue, concurrency, timeout, cancel, blocked states, and subprocess cleanup on cancellation/app exit.
- Implement Claude adapter for unattended print-mode execution.
- Implement Codex adapter for unattended `codex exec` execution.
- Implement Gemini adapter for unattended `gemini -p` execution. Use `--output-format json`; validate the safest workable sandbox/approval combination during implementation; do not rely on `--include-directories` or on `--approval-mode plan` as a safety boundary in headless runs. Use `GEMINI_SYSTEM_MD` for perspective injection if validated in practice; otherwise prepend perspective text to the prompt. If Gemini headless fails validation entirely, degrade to blocked state with clear diagnostic — do not let it block the milestone.
- Persist raw artifacts:
  - invocation metadata
  - stdout
  - stderr
  - provider event stream where available
  - run status
- Strip ANSI escape codes from all CLI output before parsing.
- Record explicit blocked reasons for auth, trust, flag, timeout, and execution failures.

Why second:

- This is the real product kernel.
- It proves the product can orchestrate local CLIs safely before synthesis polish.

Exit criteria:

- One run with 2 providers × 2 perspectives produces one complete app-owned session directory.
- Raw artifacts are inspectable on disk.
- Cancellation and timeout produce deterministic states.
- Claude and Codex are end-to-end functional. Gemini is either end-to-end functional or explicitly blocked with diagnostic.

#### Milestone 3: Normalization and synthesis

Goal:

- Convert raw multi-provider outputs into one trustworthy standardized brief.

Scope:

- Define normalized run schema.
- Add extraction/normalization layer per provider output.
- Build evidence matrix and disagreement tracking.
- Implement synthesis strategies:
  - consensus
  - comprehensive
  - executive
- Render `brief.md` from normalized state (Rust-rendered from structured JSON, not raw LLM prose).
- If synthesis fails, the session remains valid with raw outputs preserved — synthesis is post-processing, not a gate.
- Add session artifact viewer backend endpoints and frontend views.
- Add strategy/template storage for built-ins and local custom templates.
- Gemini normalization: parse `{response, stats, error?}` JSON schema (see `.plans/research_gemini.md` section 4).

Why third:

- Synthesis is only trustworthy after raw artifact capture is stable.

Exit criteria:

- End-to-end run produces:
  - normalized artifacts
  - evidence matrix
  - `brief.md`
- Brief explicitly captures disagreements and uncertainty.
- Artifact viewer can drill from brief to underlying runs.

#### Milestone 4: Desktop workflow and hardening

Goal:

- Make the product usable as a polished local tool rather than a backend prototype.

Scope:

- Complete run composer UX:
  - provider toggles
  - perspective toggles
  - synthesis strategy selection
  - context selection
- Complete session browser UX:
  - list
  - archive
  - delete
  - open artifact location
- Add richer blocked-state remediation text.
- Add retry behavior for transient failures.
- Add smoke e2e coverage for the full desktop flow.
- Tighten packaging/build docs for macOS local use.
- Review and reduce provider-owned side effects where supported:
  - Claude `--no-session-persistence`
  - Codex `--ephemeral`
- Document provider-owned persistence boundaries clearly, especially Gemini session history outside app cleanup scope.

Why fourth:

- Hardening is valuable, but should land after the core workflow is proven.

Exit criteria:

- Full desktop workflow works end to end.
- Main user flow is covered by smoke e2e tests.
- Build/lint/test commands are documented and reproducible.

### 5. Testing strategy

Principles:

- Prefer deterministic fixture-based tests for most coverage.
- Keep live paid-provider tests small and explicit.

Frontend:

- `pnpm biome check .`
- `pnpm vitest run`
- Playwright smoke tests for:
  - provider health rendering
  - run submission
  - session list
  - archive/delete safeguards

Backend:

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`

Rust test focus:

- path safety and canonicalization
- session layout generation
- provider probe parsing
- job state transitions
- normalization logic
- synthesis evidence grouping

Fixtures:

- Save representative stdout/stderr/JSONL outputs for each provider.
- Test normalization and synthesis against fixtures without requiring live model calls.

Live integration tests:

- Opt-in via Rust feature flag: `#[cfg(feature = "integration")]` so `cargo test` stays fast by default; `cargo test --features integration` runs the full suite
- Limited to smoke validation for installed/authenticated providers
- Never required for the default local fast test loop

### 6. Migration and rollback

There is no existing product to migrate, so the initial plan should still prepare for artifact evolution:

- version `session.json` and normalized schemas from day one
- make artifact readers tolerant to additive fields
- keep raw artifacts immutable once written
- render markdown from normalized artifacts so brief regeneration remains possible

Rollback:

- use normal git revert semantics
- because the app is local-first and greenfield, rollback risk is mostly around on-disk session compatibility
- if a later schema change breaks reading, keep a compatibility reader rather than rewriting raw artifacts in place

### 7. Manual setup tasks

- Install Node.js LTS and `pnpm`
- Install Rust stable toolchain
- Install Tauri desktop prerequisites for macOS
- Ensure `claude`, `codex`, and optionally `gemini` are installed and on PATH
- Ensure provider auth is completed outside the app
- Pick app identifier and session root conventions before scaffolding

Recommended day-one commands:

- `pnpm install`
- `pnpm lint`
- `pnpm test`
- `pnpm build`
- `pnpm tauri dev`
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`

### 8. Risks

#### Highest risk

- Provider CLI behavior drift breaks assumptions between milestones.
- Gemini headless has known gaps: no auth status command (probe via test call), no session persistence opt-out, `--include-directories` broken (#13669), `plan` approval mode auto-transitions to yolo in headless. Workarounds exist for all (see `.plans/research_gemini.md`).
- Unsafe deletion bugs could damage non-app directories if path handling is sloppy.

#### Medium risk

- Synthesis may collapse disagreements into false consensus.
- Context packs may become too large or too implicit. Default size limit ~50KB with manifest recording truncation.
- Provider-owned persistence may confuse users if not clearly separated from app-owned artifacts.

#### Lower risk

- Frontend framework choice in Tauri is straightforward if kept conventional.
- Build/test/lint setup is standard on a greenfield repo.

Risk response:

- Keep provider adapters isolated.
- Treat blocked states as a normal product concept.
- Attempt Gemini fully with safe fallbacks; degrade gracefully if headless behavior breaks.
- Test path safety aggressively.
- Keep raw artifacts and normalized artifacts both.

### 9. Decisions

- Gemini is attempted fully in M2 with known workarounds (see `.plans/research_gemini.md`). If headless behavior breaks during implementation, the adapter degrades to blocked state — it does not block milestones or other providers.
- Brief synthesis defaults to Claude (best JSON output and `--json-schema` support). Configurable in settings, not per-run UI.
- v1 UI is opinionated and minimal. No provider-specific advanced flags exposed.
- Custom perspectives and synthesis templates live in app-data only in v1; export/import is deferred to avoid extra scope and state-management risk.
