# Milestone 2 Handoff Prompt

You are starting **Milestone 2: Provider fan-out and artifact capture** for this project.

Your job is to implement Milestone 2 from the current plan with a strong bias toward:

- simple orchestration
- deterministic artifact capture
- explicit blocked states
- safe subprocess behavior
- high observability
- minimal scope creep

Do not jump ahead into Milestone 3 synthesis work unless a tiny amount of structural preparation is required to keep Milestone 2 clean. If you find issues that affect later milestones, document them briefly, but keep implementation focused on fan-out, execution, and raw evidence preservation.

## Read First

Treat these as the current baseline, in this order:

1. `docs/requirements.md`
2. `.plans/plan.md`
3. `.plans/research.md`
4. `.plans/research_gemini.md`
5. `docs/v1_research.md`
6. the actual code produced by Milestone 1

If the codebase differs from the plan, trust the real codebase first, then decide whether the code or the plan should be corrected. Do not assume Milestone 1 landed exactly as imagined. Reconcile before building on top of it.

If there is a conflict in research, prefer newer `.plans/` artifacts and freshly verified official docs over older notes.

## Milestone 2 Scope

Implement only what is needed for **Milestone 2**:

- deterministic perspective templates
- context-pack generation and manifesting
- job matrix expansion for providers × perspectives
- async job supervision with concurrency, timeout, cancel, and cleanup behavior
- Claude unattended execution
- Codex unattended execution
- Gemini unattended execution with validated safe fallbacks
- raw artifact persistence for each job
- explicit blocked/error/timeout/cancel states
- lightweight execution observability

Milestone 2 is complete when a run with 2 providers × 2 perspectives produces one complete app-owned session directory with inspectable raw artifacts, deterministic states, and working Claude/Codex execution. Gemini should be functional if verified workable, or else explicitly blocked with clear diagnostics without breaking the flow.

## Delivery Philosophy

- Keep orchestration thin and boring.
- Prefer explicit state machines and plain structs over clever abstractions.
- Persist raw evidence first; anything derived can come later.
- Normalize only as much as Milestone 2 requires for job bookkeeping, not for final synthesis.
- Build enough UI/backend surface to run and inspect jobs, but do not drift into Milestone 3 or 4 polish.

## Research Expectations

Before and during implementation, verify unstable assumptions carefully.

Areas that deserve fresh verification if they affect code you are writing:

- exact non-interactive CLI invocation patterns for Claude, Codex, and Gemini
- current output formats and parsing expectations
- approval/sandbox behavior for each provider
- timeout, cancellation, and process cleanup behavior in Tokio
- provider-specific exit codes and blocked-state signals
- any current drift in Gemini behavior, especially around `GEMINI_SYSTEM_MD`, `--output-format json`, `--include-directories`, and approval mode behavior

Use official vendor docs as the source of truth where possible.

For OpenAI/Codex documentation, use the OpenAI Docs MCP tooling first.

Do not encode remembered CLI behavior without re-validating it if that behavior is unstable or safety-sensitive.

## Known Facts Worth Preserving

- This is a **local-first** product. No hosted service, no database.
- App cleanup must remain bounded to app-owned session directories only.
- Raw provider artifacts are a core product requirement, not just debugging residue.
- Blocked states are a first-class product outcome.
- Claude and Codex are currently the lowest-risk providers on this machine.
- Gemini must be attempted, but should not be allowed to block milestone success if headless behavior proves unsafe or unreliable.
- Gemini has no standalone auth status command.
- Gemini `--include-directories` is not reliable for context delivery.
- Gemini session persistence is vendor-owned and outside app cleanup scope.
- Subscription-backed local CLI usage only. Strip API-key env vars from spawned process environments.

## Implementation Guidance

### 1. Start from the Milestone 1 boundaries

Do not bypass the session root and safe path primitives established in M1.

All raw artifacts for M2 should be written under the app-owned session directory with predictable structure and stable naming. Favor layouts that are easy for both humans and future agents to inspect.

### 2. Keep perspectives deterministic

Perspectives in M2 should be local data/templates, not an LLM pre-pass.

Requirements:

- built-ins should be stored as static project assets
- custom perspectives should be easy to add later without redesign
- prompt expansion should be reproducible
- the exact perspective used for each job should be persisted

### 3. Keep the context strategy explicit

Context inclusion is sensitive and provider behavior differs.

Requirements:

- build a context pack/manifest that records what was included
- avoid silent whole-repo inclusion
- keep context delivery inspectable
- do not rely on Gemini `--include-directories`

When a tradeoff is required, prefer smaller explicit context over broader implicit workspace access.

### 4. Keep job state transitions explicit

Use a clear, finite set of states such as:

- queued
- running
- completed
- failed
- timed_out
- blocked
- cancelled

Transitions should be intentional and observable. Avoid hidden magic, inferred states, or ad hoc strings scattered through the code.

### 5. Preserve raw artifacts aggressively

For each provider job, persist enough data to reconstruct what happened without rerunning the model.

At minimum, persist:

- invocation metadata
- provider identity and version if available
- prompt/perspective references
- stdout
- stderr
- structured event streams where available
- timestamps, duration, exit code, and terminal state

If parsing fails, keep the raw bytes/text anyway. Raw evidence matters more than pretty parsing at this stage.

### 6. Make cancellation and cleanup real

Milestone 2 is where subprocess lifecycle becomes important.

Requirements:

- cancellation should produce deterministic final state
- timed-out jobs should not remain running in the background
- partial artifacts should remain inspectable and clearly marked
- supervisor shutdown should not leave orphan processes when avoidable

Prefer a simple, explicit cleanup model over a complicated generalized process framework.

### 7. Treat Gemini carefully

Gemini should be implemented with real effort, but not with reckless assumptions.

Guidance:

- validate `GEMINI_SYSTEM_MD` in practice before depending on it
- do not rely on `--approval-mode plan` as a headless safety boundary
- do not rely on `--include-directories`
- if headless behavior is unreliable, surface Gemini as blocked with actionable diagnostics
- do not let Gemini edge cases compromise Claude/Codex reliability

### 8. Preserve observability

Make it easy for future agents and humans to understand failed or partial runs.

Add lightweight observability such as:

- structured per-job metadata
- JSONL or similarly inspectable event logging
- clear blocked/failure reason categories
- concise logs around process launch, timeout, cancellation, and artifact persistence

Do not build a telemetry system. Simple local observability is enough.

## Quality Bar

Maintain a strong local developer loop.

Frontend:

- `pnpm biome check .`
- `pnpm vitest run`
- `pnpm build`

Backend:

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`

Testing guidance:

- prefer fixture-driven tests for parsing and state transitions
- make live provider integration tests sparse and intentional
- keep default test runs independent of paid live model calls
- add targeted tests for timeout, cancel, blocked states, and artifact persistence

## Suggested Execution Order

1. Re-read the planning and research docs.
2. Audit the actual Milestone 1 code and identify what M2 can safely build on.
3. Re-verify any provider CLI behavior you will encode directly.
4. Implement perspective templates and prompt expansion.
5. Implement the context pack and manifest model.
6. Implement the job supervisor with clear states, timeout, cancellation, and cleanup.
7. Add Claude execution.
8. Add Codex execution.
9. Add Gemini execution with fallbacks and guarded failure handling.
10. Persist raw artifacts and event logs.
11. Add tests for artifact layout, job state transitions, timeout/cancel paths, and blocked-state handling.
12. Run the full quality gate and fix issues before stopping.
13. Record any material drift or unresolved provider issues briefly in `.plans/` without broadening scope.

## Non-Negotiables

- Do not drift into synthesis, evidence grouping, or `brief.md` generation.
- Do not lose raw artifacts because parsing or normalization is incomplete.
- Do not make cleanup logic touch vendor-owned persistence.
- Do not depend on undocumented or unverified Gemini behavior.
- Do not overbuild orchestration abstractions before proving the job matrix and supervisor work.

## Deliverables

At the end of Milestone 2, provide:

- real provider fan-out across selected providers and perspectives
- deterministic job state handling
- raw artifact persistence per run/job
- working Claude and Codex unattended execution
- Gemini execution or a clear blocked-state fallback
- tests and quality gates passing
- a brief note of any verified provider drift or follow-up risks

## If You Need To Make Tradeoffs

Prefer:

1. raw evidence preservation over derived convenience
2. explicit state handling over abstraction
3. provider isolation over faux unification
4. stable artifact formats over clever runtime tricks
5. safe fallbacks over risky parity claims
