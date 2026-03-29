# Multi-Model Research Synthesizer

---

## The Problem

Researching across Claude, Gemini, GPT, and local models means copying prompts, reading 3 walls of 2,000-word markdown, manually spotting where they agree and disagree, then writing a "combine them" prompt. For power users, this wastes 45+ minutes daily.

The manual workflow looks like this:

1. Write prompt in Claude → 2,000 words of markdown
2. Copy prompt to Gemini → 2,000 words of markdown
3. Copy prompt to ChatGPT → 2,000 words of markdown
4. Read all three (~15 minutes)
5. Mentally diff them (~10 minutes)
6. Write a "synthesize these" prompt (~5 minutes)
7. Read the synthesis (~5 minutes)
8. Realize it lost nuance, go back to the originals (~10 minutes)

Addy Osmani (Google, March 2026) documents a related workflow: use Opus/GPT-5 for planning, Sonnet/Codex-mini for implementation, a security model for review. He documents routing in `MODEL_ROUTING.md`. But the synthesis step — combining insights from multiple models into a single coherent brief — is done entirely in his head.

---

## The Competitive Landscape (March 2026)

### Multi-Model Comparison Tools

| Tool | What It Does | What It Doesn't Do |
|------|-------------|-------------------|
| **Perplexity Model Council** ($200/mo, Max only) | Runs query across 3 frontier models, 4th model synthesizes. Side-by-side + unified answer. | Opaque synthesis. No angle variation. No structured export. No API. Locked to Perplexity's model selection. |
| **ChatHub** ($19.99/mo) | 20+ models side-by-side in browser | Display only. No synthesis. No structured output. |
| **AiZolo** ($9.90/mo) | Multi-model display, BYOK | Selection not synthesis. "Compare and choose the best result quickly." |
| **TypingMind** (one-time purchase) | Clean UI for individual model sessions, BYOK | No fan-out. No comparison. No synthesis. |
| **Poe** (Quora) | Easy model access via single API key | No synthesis. No comparison mode. |

**The universal gap:** None implement (a) angle-varied prompting, (b) structured synthesis schemas, (c) evidence-preserving disagreement capture, or (d) export to project artifacts.

Perplexity Model Council is the closest competitor and validates demand — but it's $200/month with no API, opaque synthesis logic, and results are conversational, not artifact-ready.

### Orchestration Frameworks

- **LangGraph v1.0** (Oct 2025) — DAG-based, checkpointing, human-in-the-loop. Production-grade for deterministic workflows. Not designed for ad-hoc fan-out.
- **CrewAI v1.1.0** (Oct 2025) — Role-based crews. Most-used for AI business workflows (~70% adoption). But: architectural inconsistencies, memory issues, unresolved high-severity bugs.
- **AutoGen v0.4** (Microsoft) — Async event-driven actor model. Converging with Semantic Kernel. Less deterministic than LangGraph.
- **OpenRouter** — 500+ models, auto-routing for cost/latency. Adds ~15ms latency. No synthesis.
- **LiteLLM** — De facto standard for API normalization across providers. 100+ models. Normalization only — no synthesis.

**None of these frameworks include comparison, synthesis, or disagreement detection.** They're workflow substrates, not synthesis tools.

---

## What This Tool Would Be

Input a research question. Fan it out to multiple LLMs with angle-varied prompts. Cluster results into consensus, disagreement, and uncertainty. Produce one structured brief with confidence scoring and provenance.

### The Core Loop

```
Research Question
  ↓
Angle Decomposition
  ├── Default perspective
  ├── Security/risk perspective
  ├── Performance/scalability perspective
  ├── Devil's advocate / "why this is wrong"
  └── (Custom angles per query)
  ↓
Fan-out to N models × M angles
  ↓
Response Collection + Structured Extraction
  ↓
Synthesis Engine
  ├── Consensus claims (all models agree)
  ├── Disagreements (models conflict, with evidence from each)
  ├── Minority reports (one model says something others don't)
  ├── Uncertainty zones (all models hedge or conflict)
  └── Action items / recommendations
  ↓
Structured Brief + Raw Responses (always accessible)
```

### Angle-Varied Prompting

The most valuable innovation. Instead of sending the same prompt to 3 models, send *different* prompts that approach the question from different angles:

```
Base question: "What's the best approach to implement rate limiting
               for a multi-tenant API?"

Angle 1 (Default):
  "What's the best approach to implement rate limiting
   for a multi-tenant API?"

Angle 2 (Security):
  "From a security and abuse prevention perspective, what are
   the critical considerations for rate limiting a multi-tenant API?
   Focus on attack vectors and bypass risks."

Angle 3 (Performance):
  "From a performance and scalability perspective, what are
   the tradeoffs between rate limiting approaches for a multi-tenant
   API handling 10k+ req/sec?"

Angle 4 (Devil's advocate):
  "What are the strongest arguments AGAINST implementing
   rate limiting at the application layer for a multi-tenant API?
   What alternatives are underexplored?"
```

Each angle goes to each model. With 3 models and 4 angles, you get 12 responses — but the synthesis engine compresses them into one brief.

### The Synthesis Output

```
Research Brief: Rate Limiting for Multi-Tenant APIs
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

CONSENSUS (all models agree):
• Token bucket is the standard algorithm for multi-tenant rate limiting
  - Confidence: High | Sources: Claude [default, perf], GPT [default], Gemini [default, security]
• Per-tenant limits should be configurable, not hardcoded
  - Confidence: High | Sources: all models, all angles
• Redis is the standard backing store for distributed rate limiting
  - Confidence: High | Sources: Claude [perf], GPT [default, perf], Gemini [default]

DISAGREEMENT:
• Where to enforce rate limiting:
  - API gateway layer: Claude [default], Gemini [security] — "single enforcement point"
  - Application layer: GPT [default, perf] — "more context-aware, can rate limit by business action"
  - Both: Claude [security] — "defense in depth"

MINORITY REPORTS:
• Gemini [devil's advocate]: "Consider not rate limiting at all — use
  adaptive throttling based on system load instead of arbitrary limits.
  Rate limits are a UX problem disguised as a scaling solution."
  - No other model raised this. Worth investigating.

UNCERTAINTY:
• No model gave a confident answer on: cost allocation per tenant
  when rate limits are shared across a cluster

ACTION ITEMS:
1. Prototype token bucket with Redis — consensus approach
2. Investigate gateway vs. application enforcement — test both
3. Research adaptive throttling as alternative (Gemini's minority view)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Models: Claude Opus 4.6, GPT-5, Gemini 3.1
Angles: default, security, performance, devil's advocate
Cost: ~$0.08 | Tokens: 14,200 in / 8,400 out
```

### Critical Design Constraint: Show Raw Alongside Synthesis

An ACL 2025 finding revealed that LLMs are significantly more likely to take a definite stance when acting as a judge than when generating answers directly. This means the synthesizer model will appear more confident than the underlying uncertainty warrants.

The tool must **always show raw model outputs alongside the synthesis**. The synthesis is the fast path; the raw responses are the verification path. Users should be able to click any claim in the synthesis to see the exact source text from each model.

---

## Architecture

```
┌──────────────────────────────────────────────────────┐
│                   CLI / Web UI                        │
│  input question → configure angles/models → view brief│
└──────────┬───────────────────────────┬────────────────┘
           │                           │
  ┌────────▼──────────┐    ┌───────────▼──────────────┐
  │  Angle Decomposer  │    │  Model Config            │
  │  question → N      │    │  BYOK keys, model        │
  │  angle-varied       │    │  selection, cost limits   │
  │  prompts            │    │                          │
  └────────┬──────────┘    └───────────┬──────────────┘
           │                           │
  ┌────────▼───────────────────────────▼──────────────┐
  │              Fan-Out Engine                        │
  │  N prompts × M models (parallel via LiteLLM)      │
  │  structured output extraction per response         │
  │  semantic caching (skip if similar query cached)    │
  └────────┬──────────────────────────────────────────┘
           │
  ┌────────▼──────────────────────────────────────────┐
  │              Synthesis Engine                       │
  │  1. Extract claims from each response              │
  │  2. Cluster by semantic similarity                 │
  │  3. Classify: consensus / disagreement / minority  │
  │  4. Score confidence (verbalized, not logprob)     │
  │  5. Generate brief with provenance links           │
  └────────┬──────────────────────────────────────────┘
           │
  ┌────────▼──────────────────────────────────────────┐
  │              Output                                │
  │  structured brief (markdown + JSON)                │
  │  + raw responses (always preserved)                │
  │  + cost/token accounting                           │
  └───────────────────────────────────────────────────┘
```

### API Normalization

Use **LiteLLM** as the normalization layer. It translates OpenAI/Anthropic/Google message formats into a unified interface. 100+ models. This eliminates per-provider API handling.

### Structured Extraction

Use **Instructor** (3M+ monthly downloads, 11k GitHub stars) for structured output from each model response. Define a Pydantic schema for extracted claims:

```python
class ExtractedClaim:
    claim: str           # The factual or opinionated claim
    confidence: str      # "high", "medium", "low" (verbalized)
    evidence: str        # Supporting reasoning from the response
    category: str        # "recommendation", "tradeoff", "risk", "fact"

class ModelResponse:
    claims: list[ExtractedClaim]
    recommendations: list[str]
    caveats: list[str]
```

Each model response is extracted into this schema before synthesis. This makes the synthesis step operate on structured data, not raw text.

### Confidence Scoring

**The logprob problem:** Claude doesn't expose logprobs. GPT does (via `logprobs: true`). Gemini does in some configurations. Entropy-based confidence weighting — the theoretically optimal approach — is impossible across all providers.

**The practical approach:** Verbalized confidence. Ask each model to rate its confidence per claim. This is universally available, less rigorous than entropy, but good enough for research synthesis. The CER framework (ACL 2025) validates this: each claim gets a confidence score (1–5) plus a probability (0.00–1.00), used for weighted aggregation.

### Semantic Caching

~31% of typical LLM queries exhibit semantic similarity. The tool caches responses keyed by (prompt embedding, model, angle). Before making an API call, check if a semantically similar query was asked recently. Cosine similarity threshold of 0.90 triggers cache hit.

This reduces the 3x cost multiplier significantly for repeat or similar queries.

### Cost Accounting

Every synthesis query shows:
- Token count per model (input + output)
- Cost per model
- Total cost
- Cache hit rate
- Comparison to single-model cost

At current prices (March 2026):
- 3 models × 2k input + 1k output ≈ $0.006–$0.025 per query before caching
- With semantic caching: under $0.01 for most queries
- A heavy research day (20 queries): ~$0.15–$0.50

---

## MVP Scope (~200–300 hours)

### What's in

- **CLI** — `synth ask "question"` with flags for angles and models
- **BYOK configuration** — bring your own API keys for Claude, GPT, Gemini
- **3 model families** — Anthropic, OpenAI, Google (one model each)
- **4 default angles** — default, security/risk, performance, devil's advocate
- **Custom angles** — define your own via config
- **Fan-out engine** — parallel queries via LiteLLM
- **Structured extraction** — claims, recommendations, caveats per response
- **Synthesis engine** — consensus/disagreement/minority clustering
- **Structured output** — markdown brief + JSON export
- **Raw response preservation** — always accessible alongside synthesis
- **Cost accounting** — per-query token and cost tracking
- **Semantic caching** — skip similar queries
- **Minimal web UI** — view briefs, compare responses, drill into claims

### What's out (V2+)

- Saved research workflows (query chains)
- Team collaboration / shared research
- MCP server integration (expose as context for other tools)
- Automated research pipelines (scheduled queries)
- Local model support (Ollama, llama.cpp)
- Angle auto-suggestion based on question type
- Citation/URL verification

---

## Key Technical Challenges

### 1. Cross-Model Prompt Normalization

Different models respond better to different prompt styles. Claude prefers structured XML-style prompts. GPT works well with system/user message separation. Gemini handles long-context differently.

LiteLLM handles API-level normalization (message format, function calling). But prompt-level normalization — adjusting wording and structure for each model's strengths — is a design choice. Start simple: same prompt to all models. Optimize per-model prompting in V2 if response quality varies.

### 2. Contradiction Clustering

The synthesis engine must identify when models disagree. The **DiscoUQ** framework (arxiv, March 2026) provides the best current approach: extract linguistic disagreement features (evidence overlap, argument strength, divergence depth) and embedding geometry (cluster distances, dispersion, cohesion).

Practical implementation: embed each claim, cluster by cosine similarity, then within each cluster check for semantic opposition using an NLI (Natural Language Inference) classifier. Claims classified as "contradiction" within a cluster become disagreement items.

The **"Disagreement as Data"** paper (arxiv, Jan 2026) found that model disagreement is not random noise — it has consistent patterns attributable to model-specific filtering preferences. This supports treating minority views as structured signal, not error to be discarded.

### 3. Confidence Without Fake Certainty

The synthesizer must not present a false sense of agreement. If all three models hedge on a topic, the synthesis should say "uncertain" — not pick the most confident-sounding answer.

Implementation: if verbalized confidence across all models for a claim averages below "medium," the claim goes into the "uncertainty" bucket, not "consensus." If one model is highly confident and others aren't, it's a "minority report" with a confidence flag.

### 4. The Value Convergence Risk

Models are converging. February 2026 was the first time all three major providers hit near-parity on SWE-bench Verified (within 0.84 percentage points). If three models give the same answer, what's the point of asking three?

The answer: **angles matter more than models.** The value isn't "Claude says X, GPT says Y." It's "the security perspective reveals X, the performance perspective reveals Y, the devil's advocate perspective reveals Z." Even with converging models, different analytical frames produce different insights.

As models converge on benchmarks, model personality differences and specialized strengths remain:
- Claude: best prose and nuanced reasoning
- GPT-5: best all-rounder, largest ecosystem
- Gemini 3.1: leads abstract reasoning
- Claude Opus 4.6: leads coding

The tool should lean into angle variation as the primary differentiator, with multi-model as secondary.

---

## Risks

| Risk | Severity | Mitigation |
|------|----------|-----------|
| Model convergence reduces value of comparison | **High** | Lean into angle variation as primary differentiator. Multi-model is secondary. |
| API cost multiplier (3x+ per query) | **High** | Semantic caching (~31% hit rate). Show cost per query. Token budgets. |
| Side-by-side tools exist (ChatHub, AiZolo) | **Medium** | Differentiate on synthesis, not comparison. No side-by-side tool does structured extraction + consensus clustering. |
| Synthesis hallucination (fake consensus) | **Medium** | Show raw responses alongside synthesis. ACL 2025 stance-bias constraint. |
| Perplexity Model Council improves and opens API | **Medium** | Differentiate on angle variation + artifact export + BYOK. Perplexity is locked to their model selection. |
| Prompt normalization across models is hard | **Low** | Start with identical prompts. Per-model tuning is V2 optimization. |

---

## Why This Scores High

1. **Most personally useful tool on day one** — you'd use it weekly for any research question
2. **Most feasible MVP** (~200–300 hours) — smallest scope, clearest path to working product
3. **The synthesis/clustering step is the real value** — not just multi-model querying (which ChatHub already does), but structured extraction + consensus detection + evidence-preserving disagreement capture
4. **Perplexity Model Council validates demand** at $200/month — but it's locked behind a paywall with no API and opaque synthesis
5. **Natural expansion** into automated research pipelines, team collaboration, and MCP server integration
6. **Angle-varied prompting is genuinely novel** — no existing tool varies the analytical frame per query

---

*Sources: perplexity.ai/hub/blog/introducing-model-council, arxiv.org/abs/2603.20975, arxiv.org/abs/2602.18693, arxiv.org/html/2510.08146v1, python.useinstructor.com, openrouter.ai, addyosmani.com/blog/ai-coding-workflow, dl.acm.org/doi/10.1145/3744238*
