---
name: multi-model-ensemble project context
description: Core facts about the multi-model research synthesizer project — what it is, its phase, and key architectural decisions
type: project
---

A local-first Tauri desktop app that fans out a research prompt to Claude Code CLI, Codex CLI, and Gemini CLI in parallel, applies perspective-varied prompting, and synthesizes results into a markdown brief.

**Why:** Subscription-backed, no API keys, privacy-preserving (outputs stay on disk), not a chat app — artifact generation is the primary workflow.

**How to apply:** Treat each CLI as a separate execution environment with different auth, approval, workspace, and output semantics. Normalize after execution, not before. Default to read-only research mode.

Key settled decisions (as of 2026-03-30):
- Rust + Tauri v2 backend; vanilla TS + Vite frontend
- tokio::process::Command for subprocess management (not Tauri shell plugin)
- Binary discovery via `/bin/sh -lc "which ..."` at startup (Finder/Dock apps don't inherit PATH)
- Sessions stored in `~/Library/Application Support/` (macOS), filesystem-based, no database
- Env sanitization: always remove ANTHROPIC_API_KEY, CODEX_API_KEY, GEMINI_API_KEY before spawning (prior billing incident)
- Phase 0 target: provider detection + one headless run per available provider
