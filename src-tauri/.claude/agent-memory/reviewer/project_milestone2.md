---
name: milestone2_review
description: Key findings from the Milestone 2 code review (provider fan-out, orchestrator, artifact capture)
type: project
---

M2 reviewed 2026-03-30. Covers perspectives.rs, context.rs, orchestrator/, providers/ execute paths, commands/runs.rs.

**Why:** Milestone 2 adds fan-out execution, env sanitization, artifact persistence, and JSONL event logging on top of the M1 session foundation.

**How to apply:** When reviewing M3+ changes, these issues are the known carry-forwards to track.

Known issues as of review:
- `assemble_prompt` accepts `_perspective` param but ignores it — perspective instructions are NOT included in the assembled prompt, only injected via provider-specific mechanisms. Caller must use both the assembled prompt AND spec.perspective_instructions. Looks intentional per comment, but the unused param is confusing and could become a bug if a provider misses the injection step.
- Gemini temp file cleanup happens after `cmd.output().await` — if the process is killed externally or the future is dropped (e.g., timeout fires between spawn and await), the cleanup line never runs. Temp files leak under timeout.
- `run_probe_command` in providers/mod.rs does NOT call env_clear() — probe commands inherit the full parent env including API keys. Auth probe for Gemini runs a real LLM prompt (`-p "ok"`), costing a token. This is an intentional design decision.
- RunSummary.total_jobs is set to `results.len()` (only successfully-joined tasks), not `jobs.len()`. A panicking task is silently dropped from the count.
- JSONL event log file is opened without a file lock — concurrent EventLogger instances (spawned per job) may interleave at the OS level, though in practice writeln! on a short line is unlikely to produce torn writes.
- `codex exec -c developer_instructions=<value>` passes the perspective instructions as a shell-level config value without quoting or escaping. If perspective_instructions contains `=` or whitespace the argument is malformed.
- context.rs `build_context_pack` does not validate that file paths are within any sandbox — attacker-controlled context_paths could read arbitrary files.
- RunCompleted event omits `cancelled` count (present in RunSummary but not in Event::RunCompleted).
