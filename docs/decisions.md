# Architecture Decision Records

### ADR-001: Gemini auth probing deferred to runtime
**Date:** 2026-03-31
**Status:** accepted
**Context:** Gemini CLI has no `auth status` command. The original probe ran `gemini -p "ok"` as a lightweight auth check, but this loads MCP servers, makes a real API call, hits rate limits, and routinely exceeds any reasonable probe timeout.
**Decision:** The probe only checks `gemini -v` (fast, no API call). Auth failures (exit code 41) are detected at runtime and reported with remediation text.
**Consequences:** Gemini shows as "Ready" at startup even if auth is expired. Users see the auth error only when they run a job, not at probe time. This is the correct tradeoff given the CLI's constraints.

### ADR-002: Codex exec uses no approval flag
**Date:** 2026-03-31
**Status:** accepted
**Context:** `codex exec` is inherently non-interactive. The `-a` / `--ask-for-approval` flag does not exist on this subcommand (it belongs to the interactive REPL). Early code incorrectly passed `-a never`, causing exit code 2 on every Codex run.
**Decision:** The adapter uses only `-s read-only --ephemeral` with no approval override. Read-only sandbox prevents writes; no approval prompts are possible in non-interactive mode.
**Consequences:** Codex runs succeed. Behavior is controlled entirely via `--sandbox` mode.

### ADR-003: Provider-owned persistence is out of scope for cleanup
**Date:** 2026-03-31
**Status:** accepted
**Context:** Claude, Codex, and Gemini each write their own session history outside the app's session root. Gemini has no opt-out mechanism.
**Decision:** The app uses `--no-session-persistence` (Claude) and `--ephemeral` (Codex) to minimize side effects where possible. Gemini's persistence is documented but not managed. The app never deletes anything outside its own session root.
**Consequences:** Users must understand that Gemini retains session history in `~/.gemini/tmp/`. This is documented in `PROVIDER_PERSISTENCE.md`.

### ADR-004: 90-minute default job timeout
**Date:** 2026-03-31
**Status:** accepted
**Context:** The original 120-second timeout caused all jobs to time out for any non-trivial prompt. Research prompts fanned out across multiple LLMs can legitimately take 20+ minutes.
**Decision:** Default timeout is 90 minutes (5400 seconds). Configurable via the `timeoutSecs` API parameter.
**Consequences:** Hung processes can still be cleaned up, but users won't hit timeouts during normal use. A truly stuck provider process will consume resources for up to 90 minutes before being killed.
