# Milestone 3 Handoff Prompt

You are starting **Milestone 3: Normalization and synthesis** for this project.

Your job is to implement Milestone 3 from the current plan with a strong bias toward:

- trustworthy structured artifacts
- disagreement preservation
- inspectable synthesis
- minimal false consensus
- deterministic rendering
- low scope creep

Do not jump ahead into Milestone 4 workflow polish unless a tiny amount of UI/backend wiring is needed to make Milestone 3 testable and inspectable. If you discover issues that affect later milestones, document them briefly, but keep implementation focused on normalization, evidence grouping, synthesis, and artifact viewing.

## Read First

Treat these as the current baseline, in this order:

1. `docs/requirements.md`
2. `.plans/plan.md`
3. `.plans/research.md`
4. `.plans/research_gemini.md`
5. `docs/v1_research.md`
6. the actual code produced by Milestones 1 and 2

If the codebase differs from the plan, trust the real codebase first, then decide whether the code or the plan should be corrected. Do not assume M1 or M2 landed exactly as imagined. Reconcile before building on top of them.

If there is a conflict in research, prefer newer `.plans/` artifacts and freshly verified official docs over older notes.

## Milestone 3 Scope

Implement only what is needed for **Milestone 3**:

- define the normalized run schema
- normalize/extract structured data from raw provider outputs
- build the evidence matrix and disagreement tracking
- implement synthesis strategies: `consensus`, `comprehensive`, `executive`
- execute synthesis in a controlled, inspectable way
- render `brief.md` from structured synthesis data
- add backend/frontend session artifact viewer support needed to inspect the results
- add strategy/template storage for built-ins and local custom templates
- support Gemini normalization using its JSON output structure where available

Milestone 3 is complete when an end-to-end run produces normalized artifacts, an evidence matrix, and `brief.md`, and the resulting brief explicitly captures disagreements and uncertainty. If synthesis fails, the session must still remain valid and inspectable with raw and normalized artifacts preserved.

## Delivery Philosophy

- Structured JSON is the source of truth; markdown is a rendering output.
- Preserve disagreements instead of smoothing them away.
- Prefer deterministic code for grouping and rendering over freeform model prose.
- Use model assistance only where it materially improves extraction or synthesis and remains inspectable.
- Keep the UI/backend additions narrow and focused on artifact visibility, not broad workflow polish.

## Research Expectations

Before and during implementation, verify unstable assumptions carefully.

Areas that deserve fresh verification if they affect code you are writing:

- current schema-constrained output options for Claude and Codex
- practical extraction prompt shape for turning raw outputs into structured claims
- exact Gemini JSON output shape and failure semantics
- whether the existing M2 artifact layout supports efficient normalization and viewer access
- failure modes when synthesis model calls fail, timeout, or return malformed structured output

Use official vendor docs as the source of truth where possible.

For OpenAI/Codex documentation, use the OpenAI Docs MCP tooling first.

Do not encode synthesis assumptions just because they seem plausible. Test with representative fixtures and verify that the chosen approach preserves uncertainty instead of laundering it away.

## Known Facts Worth Preserving

- This is a **local-first** product. No hosted service, no database.
- Raw artifacts are a core product requirement and must remain preserved.
- Structured normalized artifacts are the canonical internal state.
- `brief.md` should be rendered from structured synthesis output, not treated as the source of truth.
- Synthesis is post-processing. A failed synthesis step must not invalidate the session.
- Cross-provider behavior is materially different. Normalize after execution, not before.
- Claude and Codex currently have the strongest schema-guided automation surface.
- Gemini normalization should use its actual JSON output structure where available.

## Implementation Guidance

### 1. Start from real Milestone 2 artifacts

Do not design normalization in the abstract.

Inspect the real raw artifacts produced by M2 and make the normalized schema fit reality while still preserving a stable internal contract. If M2 drifted from the plan, decide whether to adapt the normalization layer, fix M2 artifact shape, or both.

### 2. Make the normalized schema explicit and durable

Your normalized schema should be small, legible, and versionable.

It should preserve enough information to support:

- per-run summaries
- claims/themes
- recommendations/action items
- caveats/uncertainty
- disagreement/support relationships
- provenance back to raw provider runs

Avoid stuffing everything into untyped metadata blobs if the information matters downstream.

### 3. Preserve provenance aggressively

Every normalized claim or extracted insight should be traceable back to where it came from.

Requirements:

- preserve provider and perspective origin
- preserve references to the raw run/job artifact
- make it easy to drill from `brief.md` into supporting evidence
- avoid irreversible collapsing of competing claims too early

If a simplification makes provenance weaker, assume it is the wrong simplification.

### 4. Prevent fake consensus

This is the main product trust risk in M3.

Requirements:

- track support, disagreement, and uncertainty explicitly
- do not merge similar-looking claims unless you can still preserve dissent
- keep missing evidence visible
- if the models conflict, say so plainly

The evidence matrix should make disagreement first-class, not an afterthought.

### 5. Keep model-assisted extraction bounded

If you use a model to extract structured claims from raw text, keep that step tightly constrained.

Guidance:

- prefer schema-constrained output when supported
- keep extraction prompts deterministic and auditable
- capture the extraction input/output as artifacts
- treat malformed extraction output as a recoverable failure, not a catastrophic one
- keep enough raw context to retry or inspect extraction later

### 6. Keep brief rendering deterministic

`brief.md` should be rendered by code from structured data.

Requirements:

- stable section order
- explicit disagreements and uncertainty sections
- clear action items
- source/provenance references where useful

Do not let freeform model prose become the only durable artifact.

### 7. Keep strategy/template handling simple

Built-in strategies should be local assets with clear names and controlled behavior.

Requirements:

- keep built-ins simple and inspectable
- support local custom templates without redesigning the system
- avoid creating a broad templating engine if simple file-backed templates are enough

### 8. Add just enough viewer support

Milestone 3 needs artifact visibility, not full workflow polish.

Add only the backend/frontend support needed to:

- inspect normalized artifacts
- inspect the evidence matrix
- view `brief.md`
- drill into underlying raw runs

Do not broaden this into full M4 experience design.

### 9. Preserve observability

Make it easy for future agents and humans to understand why synthesis did or did not produce a trustworthy result.

Add lightweight observability such as:

- structured normalization results
- explicit extraction/synthesis step status
- clear failure categories for malformed output, timeout, or schema mismatch
- concise logs around extraction, evidence grouping, and brief rendering

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

- prefer fixture-driven tests for normalization, evidence grouping, and brief rendering
- add adversarial fixtures where providers disagree or outputs are malformed
- keep live provider integration tests sparse and intentional
- do not make the default test loop depend on paid live model calls
- test that failed synthesis still leaves a valid session with preserved artifacts

## Suggested Execution Order

1. Re-read the planning and research docs.
2. Audit the real M1/M2 code and artifact layout.
3. Re-verify any schema-constrained provider behavior you will encode directly.
4. Define the normalized schema and artifact layout for derived outputs.
5. Implement per-provider normalization/parsing from raw artifacts.
6. Implement the extraction path for structured claims where needed.
7. Build the evidence matrix and disagreement tracking.
8. Implement built-in synthesis strategies.
9. Implement the synthesis execution step with recoverable failure handling.
10. Render `brief.md` from structured synthesis output.
11. Add narrow backend/frontend artifact viewer support for normalized artifacts and brief inspection.
12. Add tests for normalization, disagreement handling, brief rendering, and synthesis failure modes.
13. Run the full quality gate and fix issues before stopping.
14. Record any material drift or follow-up risks briefly in `.plans/` without broadening scope.

## Non-Negotiables

- Do not treat `brief.md` as the canonical data model.
- Do not collapse disagreement into fake consensus for the sake of nicer output.
- Do not lose provenance from normalized artifacts back to raw runs.
- Do not let synthesis failure invalidate otherwise successful sessions.
- Do not overbuild a generalized prompt/template framework before proving the basic strategies work.

## Deliverables

At the end of Milestone 3, provide:

- normalized artifacts per run/session
- an evidence matrix with disagreement tracking
- built-in synthesis strategies
- rendered `brief.md`
- viewer support to inspect normalized artifacts and drill into raw evidence
- tests and quality gates passing
- a brief note of any verified provider drift or follow-up risks

## If You Need To Make Tradeoffs

Prefer:

1. provenance and disagreement preservation over polished prose
2. structured outputs over freeform text
3. deterministic rendering over model-authored formatting
4. narrow inspectable synthesis steps over magical end-to-end prompts
5. recoverable failures over silent degradation
