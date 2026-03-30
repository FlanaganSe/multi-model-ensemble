# Build & Run Guide

## Prerequisites

- **Node.js** LTS (20+)
- **pnpm** 9+
- **Rust** stable toolchain (1.75+)
- **Tauri v2** desktop prerequisites for macOS:
  - Xcode Command Line Tools: `xcode-select --install`
- At least one provider CLI installed and authenticated:
  - `claude` — [Install](https://code.claude.com/docs/en/getting-started), then `claude auth login`
  - `codex` — `npm install -g @openai/codex`, then `codex login`
  - `gemini` — `see https://google-gemini.github.io/gemini-cli/`, then run `gemini` interactively to auth

## Install

```sh
pnpm install
```

## Development

```sh
pnpm tauri dev
```

This starts both the Vite dev server and the Tauri desktop window.

## Build

```sh
pnpm build          # frontend only (tsc + vite)
pnpm tauri build    # full desktop app bundle
```

## Test

### Frontend

```sh
pnpm vitest run     # unit tests
pnpm biome check .  # lint + format check
pnpm build          # type check + build
```

### Backend (Rust)

```sh
cd src-tauri
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

### Full quality gate

```sh
pnpm biome check . && pnpm vitest run && pnpm build && \
cd src-tauri && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test
```

## Provider Auth

The app probes each provider CLI at startup. Providers that are not installed or not authenticated are shown as blocked with remediation instructions.

| Provider | Auth command | Subscription |
|----------|-------------|-------------|
| Claude | `claude auth login` | Anthropic subscription |
| Codex | `codex login` | ChatGPT subscription |
| Gemini | Run `gemini` interactively | Google account |

**Important:** The app strips `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `CODEX_API_KEY`, and `GEMINI_API_KEY` from spawned process environments to prevent accidental API billing. Only subscription-backed auth is supported.

## Session Storage

Sessions are stored in the platform app-data directory:
- macOS: `~/Library/Application Support/multi-model-synthesizer/sessions/`

Each session is a self-contained directory with raw artifacts, normalized data, and the synthesized brief.

## Known Caveats

- **Gemini `--version` / `-v`** can hang on some machines. The probe uses a 10-second timeout.
- **Gemini `--include-directories`** is broken (issue #13669). Context is delivered via CWD and app-level context packs instead.
- **PATH discovery**: When launched from Finder/Dock, the app uses `/bin/sh -lc "which ..."` to find provider binaries, since Tauri apps do not inherit shell PATH.
- **Gemini session persistence**: Gemini always writes session history to `~/.gemini/tmp/`. See `PROVIDER_PERSISTENCE.md` for details.
