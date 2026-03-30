# Milestone 4 Handoff Prompt

You are starting **Milestone 4: Desktop workflow and hardening** for this project.

Your job is to implement Milestone 4 from the current plan with a strong bias toward:

- complete end-to-end usability
- reliability over novelty
- minimal workflow friction
- clear diagnostics and remediation
- packaging/build reproducibility
- low scope creep

This is the finishing and hardening milestone for the MVP, not an invitation to redesign the product. Focus on making the core workflow coherent, robust, and inspectable. If you discover larger improvements that belong in a future version, record them briefly and move on.

## Read First

Treat these as the current baseline, in this order:

1. `docs/requirements.md`
2. `.plans/plan.md`
3. `.plans/research.md`
4. `.plans/research_gemini.md`
5. `docs/v1_research.md`
6. the actual code produced by Milestones 1, 2, and 3

If the codebase differs from the plan, trust the real codebase first, then decide whether the code or the plan should be corrected. Do not assume the earlier milestones landed exactly as imagined. Reconcile before building on top of them.

If there is a conflict in research, prefer newer `.plans/` artifacts and freshly verified official docs over older notes.

## Milestone 4 Scope

Implement only what is needed for **Milestone 4**:

- complete the run composer UX
- complete the session browser UX
- add stronger blocked-state remediation text
- add retry behavior for transient failures
- add smoke e2e coverage for the main desktop workflow
- tighten packaging/build docs for local macOS use
- reduce provider-owned side effects where supported
- document provider-owned persistence boundaries clearly

Milestone 4 is complete when the full desktop workflow works end to end, the main user flow has smoke e2e coverage, and build/lint/test commands are documented and reproducible.

## Delivery Philosophy

- Finish the product you planned, not a more ambitious one.
- Prefer clarity and reliability over feature breadth.
- Keep the UI opinionated and minimal.
- Preserve the established artifact-first workflow.
- Harden the existing system instead of rewriting core architecture late.

## Research Expectations

Before and during implementation, verify unstable assumptions carefully.

Areas that deserve fresh verification if they affect code you are writing:

- current Tauri packaging/build steps for this exact app shape
- any provider flags related to side-effect reduction, especially Claude and Codex persistence behavior
- desktop-specific runtime behavior when launched outside the shell
- current best practice for smoke e2e coverage in Tauri v2
- any provider-side changes that affect blocked-state remediation text

Use official vendor docs as the source of truth where possible.

For OpenAI/Codex documentation, use the OpenAI Docs MCP tooling first.

Do not assume a feature belongs in M4 just because it would be nice. If it does not materially improve the planned MVP workflow, leave it out.

## Known Facts Worth Preserving

- This is a **local-first** product. No hosted service, no database.
- Artifact generation and inspection are the primary workflow, not chat.
- App cleanup must remain bounded to app-owned session directories only.
- Provider-owned persistence is out of scope for cleanup and must be communicated clearly.
- Claude may support `--no-session-persistence`.
- Codex may support `--ephemeral`.
- Gemini does not offer the same persistence controls and should be documented accordingly.
- The v1 UI should remain opinionated and should not expose a wide surface of provider-specific advanced flags.

## Implementation Guidance

### 1. Start from the real end-to-end flow

Before changing UI or UX structure, run through the actual workflow built so far and identify where it is incomplete, confusing, or brittle.

Focus on the core loop:

- compose run
- choose providers/perspectives/strategy/context
- execute
- monitor progress
- inspect brief and raw artifacts
- browse past sessions
- archive/delete safely

Fix real usability and reliability gaps in that loop. Do not widen the product surface beyond it.

### 2. Keep the run composer simple

The composer should expose what the plan requires and no more.

Requirements:

- provider toggles
- perspective toggles
- synthesis strategy selection
- context selection
- clear run action and feedback

Do not add provider-specific advanced tuning panels unless truly necessary for the planned workflow.

### 3. Make blocked states actionable

Blocked states should help the user recover, not just signal failure.

Requirements:

- clear explanation of what is blocked
- likely cause when known
- practical remediation text
- graceful behavior when one provider is unavailable but others can still run

Avoid vague error messages and internal jargon where a user-facing explanation is possible.

### 4. Keep retry behavior narrow and safe

Retry logic should target transient failures, not mask structural problems.

Guidance:

- prefer explicit retry actions or narrowly scoped automated retry
- do not silently loop on failures
- preserve prior artifacts and failure details when retrying
- keep retry state inspectable

If a retry policy becomes complicated, it is probably too broad for this milestone.

### 5. Preserve artifact-first inspection

The desktop flow should keep the user close to the artifacts.

Requirements:

- easy path from session list to brief
- easy path from brief to raw/normalized evidence
- safe archive/delete actions
- visible metadata for sessions and job outcomes

Do not let UI polish hide or obscure inspectability.

### 6. Reduce provider-owned side effects where supported

Apply side-effect reduction only where it is documented and safe.

Guidance:

- review Claude `--no-session-persistence`
- review Codex `--ephemeral`
- do not overpromise equivalent behavior for Gemini when it is not supported
- document the actual boundary between app-owned artifacts and vendor-owned persistence

### 7. Preserve observability

M4 should improve debuggability, not reduce it behind a cleaner UI.

Add lightweight observability such as:

- visible session/job status in the UI
- readable failure/blocked messages
- concise logs for retry attempts and packaging/runtime issues
- documentation of where artifacts and provider-owned state live

Do not build a telemetry platform. Local, readable observability is enough.

### 8. Tighten documentation and reproducibility

The MVP is not really complete until another agent or developer can build and run it predictably.

Requirements:

- document the local build/run/test flow clearly
- document required prerequisites
- document provider auth expectations
- document known provider caveats, especially Gemini persistence and headless limitations

Prefer short, accurate build docs over long aspirational documentation.

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

End-to-end:

- add smoke coverage for the main desktop workflow
- keep smoke tests focused on the critical path rather than exhaustive permutations

Testing guidance:

- prioritize the main user path
- test archive/delete safeguards
- test blocked-state UI
- test the app when one provider is unavailable
- test that artifact inspection still works after failures and retries

## Suggested Execution Order

1. Re-read the planning and research docs.
2. Audit the real M1-M3 code and run the actual workflow manually.
3. Identify the minimum set of gaps blocking a coherent MVP flow.
4. Complete the run composer UI.
5. Complete the session browser and artifact navigation flow.
6. Improve blocked-state messaging and remediation text.
7. Add narrow retry behavior for transient failures.
8. Review and reduce provider-owned side effects where supported.
9. Document provider-owned persistence boundaries clearly.
10. Add smoke e2e coverage for the main workflow.
11. Tighten build/run/test/packaging docs for local macOS use.
12. Run the full quality gate and fix issues before stopping.
13. Record any future-looking improvements briefly without expanding scope.

## Non-Negotiables

- Do not turn M4 into a product redesign.
- Do not add broad advanced-settings surfaces that dilute the MVP.
- Do not weaken artifact inspectability in pursuit of a cleaner UI.
- Do not make cleanup logic touch vendor-owned persistence.
- Do not silently hide failures behind retry behavior.

## Deliverables

At the end of Milestone 4, provide:

- a coherent end-to-end desktop workflow
- completed run composer and session browser UX
- actionable blocked-state remediation
- narrow safe retry behavior
- smoke e2e coverage for the main user flow
- tightened build/run/test/packaging docs
- clear documentation of provider-owned persistence boundaries
- tests and quality gates passing

## If You Need To Make Tradeoffs

Prefer:

1. workflow clarity over feature breadth
2. reliability over polish flourishes
3. artifact visibility over UI abstraction
4. actionable diagnostics over generic messaging
5. reproducible local operation over optional conveniences
