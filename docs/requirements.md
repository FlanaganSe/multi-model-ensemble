# Product Requirements: Multi-Model Research Synthesizer

## What It Is

A local-first desktop tool that fans out a research prompt across multiple local LLM subscription CLI tools (Claude Code, Codex CLI, Gemini CLI), applies perspective-varied prompting, and consolidates results into one structured markdown brief — while preserving raw outputs for verification.

**Not** an API wrapper. Uses existing subscription-backed CLI tools already on the user's machine.

## Core Loop

```
Prompt + optional directory context
  → Perspective transformation (N fixed perspectives)
  → Fan-out to local CLI tools in parallel
  → Collect raw responses to disk
  → Consolidate into one structured markdown brief
  → Session saved as a unit (brief + raw + metadata)
```

## Requirements

### Input
- Accept a text prompt
- Accept optional directory/file context to include with the prompt
- Select which models to use per run (any combination of Claude, Codex, Gemini)
- Select which perspectives to apply per run

### Perspectives
- Allow selectable multiple perspectives: default, creative, adversarial, performance, devil's advocate
- Support user-defined custom perspectives
- Each perspective transforms the base prompt (using LLM?) through its analytical frame before fan-out
- Results in up to dozens of prompts.

### Fan-Out
- Route work to local CLI tools (Claude Code, Codex CLI, Gemini CLI)
- No API keys — subscription-backed local tools only (enforce subscription in CLI)
- Run model invocations in parallel where possible
- Include directory context in each invocation when provided

### Synthesis
- Produce one consolidated markdown brief per run
- Default Brief structure: Consensus summary, disagreements, uncertainty, action items
- Consolidation strategy should be selectable (e.g., consensus, comprehensive, executive) and editable
- Raw model outputs are preserved alongside the brief

### Sessions
- Each run creates a session directory on disk
- Session contains: brief, raw responses, metadata (models, perspectives, timestamps, cost/time)
- Sessions support: list, archive, delete (individually or bulk)
- Filesystem-based — no database, git-friendly, inspectable

### Interface
- Tauri desktop app as the primary interface
- UI is a control surface: prompt input, perspective/model toggles, run list, artifact viewer
- View synthesized brief and drill into raw outputs
- Session browser with metadata (date, models, perspectives, status)
- Chat-like interaction is optional/secondary — artifact generation is the primary workflow

## Open Questions

- Sometimes CLI tools may ask follow up questions? Can this be handled cleanly (e.g. toggle for "prevent follow-ups" or for answering questions?)
- What is the optimal session directory structure for traceability + easy cleanup?
