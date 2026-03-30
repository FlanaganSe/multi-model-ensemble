# Milestone 1 Handoff Prompt

You are starting **Milestone 1: Foundation and safe boundaries** for this project.

Your job is to implement Milestone 1 from the current plan with a strong bias toward:

- simple architecture
- low scope creep
- safe filesystem boundaries
- inspectable behavior
- testability
- clear diagnostics

Do not jump ahead into Milestone 2 unless a very small amount of probing is required to validate a Milestone 1 decision. If you discover something important that affects later milestones, document it, but keep implementation scope tight.

## Read First

Treat these as the current planning baseline, in this order:

1. `docs/requirements.md`
2. `.plans/plan.md`
3. `.plans/research.md`
4. `.plans/research_gemini.md`
5. `docs/v1_research.md`

If there is a conflict, prefer the newer planning artifacts in `.plans/` over older assumptions in `docs/v1_research.md`, and prefer freshly verified official docs over both when the topic is time-sensitive.

## Milestone 1 Scope

Implement only what is needed for **Milestone 1**:

- scaffold the Tauri v2 app with React + TypeScript + Vite
- establish the Rust backend module layout and provider adapter trait/contracts
- implement provider probing for Claude, Codex, and Gemini
- surface provider readiness and blocked reasons in a minimal UI shell
- establish the app-owned session root under platform app-data
- implement path-safe session create/archive/delete primitives
- define the canonical session layout and metadata schema version
- wire lint, typecheck, test, build, and Rust quality checks

Milestone 1 is complete when the app launches, provider probe results render, session operations are path-safe inside the app-owned root only, and the planned quality gates pass.

## Delivery Philosophy

- Keep the code straightforward and modular.
- Prefer plain data structures and pure helper functions where practical.
- Avoid premature abstractions, plugin systems, generic factories, or speculative config layers.
- Build the minimum UI needed to validate provider health and safe storage boundaries.
- Do not build fan-out orchestration, synthesis, or artifact viewers in this milestone.

## Research Expectations

Before and during implementation, verify unstable assumptions carefully.

Areas that deserve fresh verification if they affect code you are writing:

- exact CLI flags and auth/status behavior
- Tauri v2 project scaffolding and current best practices
- macOS app-data conventions and Finder/Dock PATH behavior
- subprocess lifecycle and cancellation behavior in Rust/Tokio
- any provider-specific exit codes or output shapes used by M1 probes

Use official vendor docs as the source of truth where possible.

For OpenAI/Codex-related documentation, use the OpenAI Docs MCP tooling first.

Do not blindly trust older research if current docs or current local behavior disagree. When you find drift:

- adjust the implementation to the verified behavior
- add a brief note to project planning artifacts only if the finding materially affects future milestones

## Known Facts Worth Preserving

- This is a **local-first** product. No hosted orchestrator, no database.
- Subscription-backed local CLIs only. Do not design around API-key-first flows.
- App cleanup must only touch app-owned session directories.
- Vendor-owned persistence is out of scope for app cleanup.
- Claude and Codex are the lowest-risk providers on this machine today.
- Gemini must be considered, but Milestone 1 should not become blocked on Gemini parity.
- Gemini has no standalone auth status command; auth readiness is probed via a minimal headless invocation and exit code.
- Tauri apps launched from Finder/Dock may not inherit shell PATH. Plan provider discovery accordingly.

## Implementation Guidance

### 1. Start with boundaries

Implement the session root and safe path utilities early.

Requirements:

- canonicalize paths before destructive operations
- reject any delete/archive target outside the app-owned session root
- never recurse into user-selected project directories for cleanup
- keep session metadata versioned from day one

This is a safety-critical area. Favor explicitness over convenience.

### 2. Keep provider probing small and deterministic

Milestone 1 only needs provider health checks, not full execution adapters.

Provider probing should answer:

- is the binary discoverable?
- what version is installed?
- is auth usable?
- if blocked, why?
- what remediation text should the UI show?

Treat blocked states as a first-class product concept, not as exceptional failures.

### 3. Keep the UI intentionally thin

The UI only needs enough surface area to prove:

- the app launches
- provider status is visible
- session safety primitives can be exercised or inspected

Do not overbuild state management. Keep the frontend small and legible.

### 4. Preserve observability

Even in M1, make debugging easy for future agents and humans.

Add lightweight observability such as:

- structured probe results
- clear error categories for blocked states
- explicit version/path/auth fields in provider status data
- concise backend logs around probe attempts and session path validation

Do not over-engineer a logging platform. Simple structured logs and readable error messages are enough.

### 5. Defend against unknown-unknowns

Be especially cautious around:

- PATH lookup differences between terminal and desktop-launched apps
- exit-code-based auth probing
- subprocess hangs or partial output
- accidental environment leakage to spawned processes
- path traversal or symlink edge cases in cleanup code
- assumptions that a CLI is stable because it worked once

Where a small additional check materially reduces risk, add it.

## Quality Bar

Maintain a high-quality local developer loop from the first commit.

Frontend:

- `pnpm biome check .`
- `pnpm vitest run`
- `pnpm build`

Backend:

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`

Also keep TypeScript strict and Rust types explicit. Prefer compile-time guarantees over comments.

When testing:

- prefer deterministic unit tests and fixture-style tests
- keep live provider tests minimal and targeted
- do not make the default test loop depend on paid model calls

## Suggested Execution Order

1. Re-read the planning and research docs.
2. Verify any time-sensitive tooling assumptions you will encode in M1.
3. Scaffold the Tauri app and baseline project tooling.
4. Implement the session root, layout, metadata versioning, and safe path operations.
5. Implement provider discovery and probe contracts in Rust.
6. Add a minimal UI for provider health and basic session-boundary visibility.
7. Add tests for path safety, provider probe parsing, and blocked-state handling.
8. Run the full quality gate and fix issues before stopping.
9. If you find material drift, record it briefly in `.plans/` without expanding the milestone.

## Non-Negotiables

- Do not broaden Milestone 1 into full provider execution orchestration.
- Do not assume Gemini behavior beyond what has been researched or freshly verified.
- Do not implement unsafe cleanup shortcuts.
- Do not rely on undocumented CLI behavior when a documented alternative exists.
- Do not add complexity to “prepare for the future” unless it is clearly justified by Milestone 1.

## Deliverables

At the end of Milestone 1, provide:

- the working app scaffold
- provider probe functionality with clear statuses
- path-safe session primitives
- minimal UI shell
- tests and quality tooling wired and passing
- a brief note of any verified drift or follow-up risks discovered during implementation

## If You Need To Make Tradeoffs

Prefer:

1. safety over convenience
2. clear behavior over cleverness
3. thin vertical slices over broad scaffolding
4. deterministic tests over live integration breadth
5. documented verified behavior over remembered behavior
