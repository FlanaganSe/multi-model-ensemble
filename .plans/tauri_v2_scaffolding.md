# Tauri v2 Scaffolding Research
**Date:** 2026-03-30
**Purpose:** Precise reference for manually scaffolding a Tauri v2 + React + TypeScript + Vite project. Focused on the manual path for full control over project structure.

---

## 1. Current State

The project plan (`.claude/plans/plan.md:51`) specifies: "Tauri v2. Not v1." and "vanilla TS + Vite frontend." The implementation plan for M1 (`plan.md:76`) references a Rust project structure of `src-tauri/src/{main.rs, lib.rs, provider/, types/}`. No Tauri project has been scaffolded yet — the repo is greenfield.

**NOTE:** The plan says "vanilla TS + Vite" with no framework (`plan.md:59`). React is explicitly deferred: "Vanilla TypeScript + Vite. No React, no SolidJS — the UI is simple enough to not need a framework." (`plan.md:213`). This research covers the React/TS variant because it was asked for — but the project plan has a stated preference for vanilla TS. The structural information below (Cargo.toml, tauri.conf.json, Rust entry points) is identical regardless of whether the frontend uses React or vanilla TS. Only the frontend-layer files differ.

**Latest verified versions (2026-03-30):**
- `tauri` crate: **2.10.3** (released 2026-03-04, confirmed via docs.rs and web search)
- `tauri-build` crate: **2.5.3** (confirmed via docs.rs)
- `@tauri-apps/api` npm: **2.10.1**
- `@tauri-apps/cli` npm: **2.10.1**

---

## 2. Project Creation Command

### 2a. Interactive scaffolding (fastest start)

```bash
# npm
npm create tauri-app@latest

# pnpm
pnpm create tauri-app

# yarn
yarn create tauri-app

# shell (no package manager)
sh <(curl https://create.tauri.app/sh)

# Cargo (least common)
cargo install create-tauri-app --locked && cargo create-tauri-app
```

The interactive wizard prompts for:
1. Project name and bundle identifier (e.g., `com.multimodel.synthesizer`)
2. Frontend language: TypeScript/JavaScript, Rust, or .NET
3. Package manager: pnpm, yarn, npm, or bun
4. UI template: Vanilla, Vue, Svelte, React, SolidJS, Angular, Preact (JS) or Yew, Leptos, Sycamore (Rust)
5. UI flavor: TypeScript or JavaScript

For this project, select: TypeScript → pnpm → Vanilla (not React per plan.md:213).

### 2b. Manual path

No special command needed. Create the directory structure by hand and write each file. This is the recommended approach when you need precise control (the project plan is explicit about the Rust module layout). Manual setup is described in full below.

---

## 3. Default Project Structure

The canonical structure for a Tauri v2 project with a Vite frontend:

```
<project-root>/
├── package.json                   # Frontend dependencies + tauri script
├── index.html                     # Frontend entry HTML
├── vite.config.ts                 # Vite configuration
├── tsconfig.json                  # TypeScript config
├── src/                           # Frontend source
│   └── main.ts                    # Frontend entry point (main.tsx for React)
└── src-tauri/                     # Rust/Tauri backend
    ├── Cargo.toml                 # Rust manifest
    ├── Cargo.lock                 # Committed for consistent builds
    ├── build.rs                   # Tauri build script (one line)
    ├── tauri.conf.json            # Tauri configuration
    ├── src/
    │   ├── main.rs                # Desktop entry point (minimal — calls lib)
    │   └── lib.rs                 # All app logic + run() function
    ├── icons/                     # App icons (required by bundler)
    │   ├── icon.png
    │   ├── icon.icns              # macOS
    │   └── icon.ico               # Windows
    └── capabilities/
        └── default.json           # Permissions/ACL for the app
```

**Important:** For this project (M1 target), the `src-tauri/src/` directory will be expanded as:

```
src-tauri/src/
├── main.rs
├── lib.rs
├── provider/
│   ├── mod.rs
│   ├── claude.rs
│   ├── codex.rs
│   └── gemini.rs
└── types/
    └── mod.rs
```

Source: `v2.tauri.app/start/project-structure/`, `plan.md:76–78`.

---

## 4. Cargo.toml for Tauri v2

### `src-tauri/Cargo.toml`

```toml
[package]
name = "multi-model-synthesizer"
version = "0.1.0"
edition = "2021"

# Required in v2 for mobile support (cdylib/staticlib) and shared library (rlib)
# Do NOT omit this — without it, mobile builds fail and lib.rs is unreachable from main.rs
[lib]
name = "app_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["rt-multi-thread", "process", "io-util", "time"] }
```

### Key points

- **`tauri` and `tauri-build` versions**: Use `"2"` (not `"2.0"` or `"2.10.3"`) to get the latest compatible patch. Pin with `=2.10.3` only if you need exact reproducibility.
- **`[lib]` section**: Required for v2. The `crate-type` triplet (`staticlib`, `cdylib`, `rlib`) allows the same codebase to compile for desktop (binary via main.rs) and mobile (library). The `name = "app_lib"` must match the call `app_lib::run()` in `main.rs`.
- **`tauri` features**: Leave empty to start. `tauri dev` and `tauri build` populate features automatically from `tauri.conf.json`. Add `devtools` for debug builds: `features = ["devtools"]`.
- **`tokio`**: Tauri v2 already has tokio as a transitive dependency. Adding it directly is fine and necessary to use `tokio::process::Command`. The features needed are `rt-multi-thread`, `process`, `io-util`, and `time` (for timeout).
- **`serde` / `serde_json`**: Required for any `#[tauri::command]` that returns/accepts structured data, and for JSON parsing of CLI outputs.

### What changed from v1

In v1, `Cargo.toml` had no `[lib]` section — `main.rs` was the sole entry point. The v2 split (`main.rs` → thin delegate, `lib.rs` → all logic) requires the `[lib]` block. This is a **breaking structural change**, not just a version bump.

Source: `v2.tauri.app/start/migrate/from-tauri-1/`, `v2.tauri.app/develop/configuration-files/`.

---

## 5. `build.rs`

Exactly one line of content plus boilerplate:

```rust
fn main() {
    tauri_build::build()
}
```

This is mandatory. Without `build.rs`, the Tauri CLI cannot locate the Rust project and capabilities/permissions won't be generated.

Source: `v2.tauri.app/start/project-structure/`.

---

## 6. `tauri.conf.json` Structure for Vite

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Multi-Model Synthesizer",
  "version": "0.1.0",
  "identifier": "com.multimodel.synthesizer",
  "build": {
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist"
  },
  "app": {
    "withGlobalTauri": false,
    "windows": [
      {
        "title": "Multi-Model Synthesizer",
        "width": 1200,
        "height": 800,
        "minWidth": 900,
        "minHeight": 600,
        "resizable": true,
        "fullscreen": false,
        "center": true
      }
    ],
    "security": {
      "csp": "default-src 'self'; connect-src ipc: http://ipc.localhost"
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

### Key field notes

| Field | Notes |
|-------|-------|
| `$schema` | Points to `https://schema.tauri.app/config/2`. Enables IDE autocomplete. |
| `identifier` | Reverse domain notation. Used for macOS bundle ID, app-data path (`~/Library/Application Support/<identifier>/`). Must be set correctly before first build — changing it invalidates existing sessions. |
| `build.devUrl` | Must match the port in `vite.config.ts` (`server.port: 5173`). |
| `build.frontendDist` | Relative to `src-tauri/`. `../dist` points to `<project-root>/dist/`. |
| `build.beforeDevCommand` | Runs before `tauri dev`. For pnpm Vite: `pnpm dev`. |
| `app.withGlobalTauri` | `false` means `window.__TAURI__` is not injected. Use the `@tauri-apps/api` npm package instead (recommended). |
| `bundle.icon` | Paths relative to `src-tauri/`. Must exist or `tauri build` fails. |

### What changed from v1

- The entire `"tauri"` top-level key no longer exists. It is split into `"app"` (runtime config) and `"bundle"` (packaging config) at the root level.
- `"build.distDir"` → `"build.frontendDist"`. `"build.devPath"` → `"build.devUrl"`.
- `"tauri.allowlist"` is **entirely removed** — replaced by capability files in `src-tauri/capabilities/`.
- `"package.productName"` and `"package.version"` move to top-level `"productName"` and `"version"`.

Source: `v2.tauri.app/start/migrate/from-tauri-1/`, `v2.tauri.app/reference/config/`.

---

## 7. Rust Entry Points: `main.rs` and `lib.rs`

### `src-tauri/src/main.rs`

```rust
// Prevents additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    app_lib::run()
}
```

Do not add logic here. This file is the desktop-only thin wrapper. The `app_lib` name must match `[lib] name` in `Cargo.toml`.

### `src-tauri/src/lib.rs`

```rust
// Mobile entry point attribute — no-op on desktop, required for mobile targets
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // register commands here, e.g.:
            // commands::get_providers,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Adding commands

Commands are defined with `#[tauri::command]` and registered in the `generate_handler!` macro:

```rust
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

For larger projects (like M1's provider module layout), commands live in their module files and are re-exported:

```rust
// In lib.rs
mod provider;
mod types;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            provider::get_providers,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### What changed from v1

In v1, `main.rs` contained all app logic and the `Builder`. In v2, this is split:
- `lib.rs` gets the `#[cfg_attr(mobile, tauri::mobile_entry_point)]` attribute + `pub fn run()` containing the `Builder`.
- `main.rs` becomes a one-line delegate: `app_lib::run()`.

This split exists because mobile builds compile the app as a library (`.so`/`.dylib`), not a binary. The `[lib]` section in `Cargo.toml` exposes `lib.rs` as a library target reachable from both the binary (`main.rs`) and the mobile harness.

Source: `v2.tauri.app/start/project-structure/`, `v2.tauri.app/develop/calling-rust/`, `v2.tauri.app/start/migrate/from-tauri-1/`.

---

## 8. Capabilities File: `src-tauri/capabilities/default.json`

The capabilities system replaces v1's `allowlist`. It is an ACL that controls which Tauri commands are accessible from the JavaScript frontend.

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:path:default",
    "core:event:default",
    "core:window:default",
    "core:app:default",
    "core:resources:default",
    "core:menu:default",
    "core:tray:default"
  ]
}
```

### Notes

- The `$schema` path `../gen/schemas/desktop-schema.json` is generated by `tauri dev`/`tauri build` into `src-tauri/gen/`. It does not exist yet in a fresh project — `tauri dev` creates it on first run.
- `"windows": ["main"]` refers to the window label. The default window created by Tauri has label `"main"`.
- Permission format: `"<plugin>:<permission-group>"`. For core Tauri commands, the plugin prefix is `core:`. For external plugins (e.g., `tauri-plugin-fs`), the prefix is `fs:`.
- For custom commands defined in your own app code, you only need the permission name without a plugin prefix — but first you need to define the permission in a `permissions/` directory or the command is accessible by default if registered via `invoke_handler`.
- For this project (Phase 0/M1), no file system, dialog, or shell plugins are needed — the Rust backend does all subprocess work directly. Keep the capabilities minimal.

Source: `v2.tauri.app/security/capabilities/`, `v2.tauri.app/learn/security/using-plugin-permissions/`.

---

## 9. `vite.config.ts`

```typescript
import { defineConfig } from 'vite';
// For React, also import: import react from '@vitejs/plugin-react';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  // plugins: [react()],  // Add for React only
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: {
    // macOS/Linux: safari13. Windows: chrome105.
    target:
      process.env.TAURI_ENV_PLATFORM === 'windows'
        ? 'chrome105'
        : 'safari13',
    minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
});
```

Source: `v2.tauri.app/start/frontend/vite/`.

---

## 10. `package.json` Scripts

```json
{
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.10.1"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.10.1",
    "typescript": "^5.4.0",
    "vite": "^5.0.0"
  }
}
```

For React, also add: `"react": "^18"`, `"react-dom": "^18"`, `"@vitejs/plugin-react": "^4"`, `"@types/react"`, `"@types/react-dom"`.

---

## 11. v1 vs v2 Differences That Affect Project Setup

| Area | Tauri v1 | Tauri v2 |
|------|----------|----------|
| **Rust entry point** | Single `main.rs` with all Builder logic | `lib.rs` with `pub fn run()` + `#[cfg_attr(mobile, tauri::mobile_entry_point)]`; `main.rs` is a one-line delegate |
| **`[lib]` in Cargo.toml** | Not needed | Required: `crate-type = ["staticlib", "cdylib", "rlib"]` |
| **`tauri.conf.json` top level** | `{ "tauri": {...}, "build": {...}, "package": {...} }` | `{ "app": {...}, "build": {...}, "bundle": {...}, "identifier": "...", "productName": "...", "version": "..." }` |
| **`build.distDir`** | `"build": { "distDir": "../dist" }` | `"build": { "frontendDist": "../dist" }` |
| **`build.devPath`** | `"build": { "devPath": "http://localhost:3000" }` | `"build": { "devUrl": "http://localhost:5173" }` |
| **Permissions system** | `"tauri": { "allowlist": {...} }` — a single global on/off map | Capability files in `src-tauri/capabilities/*.json` — per-window, per-domain ACL |
| **Plugins** | Many APIs built into `tauri` crate (shell, dialog, http, etc.) | Most APIs extracted to separate crates (`tauri-plugin-shell`, `tauri-plugin-dialog`, `tauri-plugin-fs`, etc.) |
| **`@tauri-apps/api` JS modules** | `@tauri-apps/api/tauri` for `invoke()` | `@tauri-apps/api/core` for `invoke()` |
| **Window type in Rust** | `Window` | `WebviewWindow` |
| **`get_window()` in Rust** | `app.get_window("main")` | `app.get_webview_window("main")` |

**Critical for this project:** The subprocess work uses `tokio::process::Command` directly in Rust — none of the removed v1 APIs (shell plugin, etc.) were going to be used anyway. The v1→v2 differences that matter are the Cargo.toml `[lib]` section, the tauri.conf.json restructuring, and the capabilities system.

---

## 12. Options for Manual Scaffolding Approach

### Option A: Run `create-tauri-app` then restructure

Run the scaffold wizard, select vanilla TS, then manually add/move the Rust module directories (`provider/`, `types/`) and update `lib.rs`.

**Pros:** Gets icons, default capabilities, and package.json boilerplate for free. Guaranteed valid starting state.
**Cons:** May generate slightly different file structure than the plan specifies; requires cleanup pass.

### Option B: Write all files by hand

Write each file from scratch using the templates in this document.

**Pros:** Exact control. Every file matches the plan's module layout from day one. No cleanup.
**Cons:** Must manually generate or download icons (the Tauri build won't run without them). Must get all file contents exactly right.

### Option C: `create-tauri-app` for scaffold, then swap in manual Rust module tree

Same as Option A but preserve only the auto-generated: `icons/`, `capabilities/default.json`, `package.json`, `vite.config.ts`, `index.html`. Replace all Rust files with the hand-written module layout.

**Pros:** Gets the fiddly parts (icons, schema paths) right automatically. Rust files start clean.
**Cons:** Two-step process.

---

## 13. Recommendation

**Use Option C** for M1 scaffolding.

1. Run `pnpm create tauri-app` with vanilla TypeScript + Vite. This generates valid icons and the capabilities schema path for you.
2. Immediately replace `src-tauri/src/main.rs` and `src-tauri/src/lib.rs` with the hand-written versions from this document.
3. Add the `provider/` and `types/` module directories to `src-tauri/src/` per the plan.
4. Update `tauri.conf.json` with the correct `identifier` (`com.multimodel.synthesizer`) and `productName`.
5. Update `src-tauri/Cargo.toml` to add `tokio` with the correct features.

**Why:** The `create-tauri-app` tool reliably generates valid icons (a multi-format set is required for `tauri build`) and the capabilities schema. These are annoying to reproduce by hand. Everything else is straightforward to write manually — and must be written manually anyway to match the plan's module layout.

**Icon note:** If you want to skip the wizard entirely, use `tauri icon <source.png>` command after installing the CLI to generate the full icon set from a single 1024x1024 PNG. This is the production-quality path.

---

## 14. Sources of Truth

| Area | Canonical Source | Verification Method | Drift Risk |
|------|-----------------|---------------------|------------|
| Project creation command | `https://v2.tauri.app/start/create-project/` | WebFetch live doc | Low — create-tauri-app wizard commands are stable |
| Project structure | `https://v2.tauri.app/start/project-structure/` | WebFetch live doc | Low |
| Cargo.toml deps / versions | `https://docs.rs/crate/tauri/latest` + `https://docs.rs/crate/tauri-build/latest` | Check docs.rs for latest patch version | Medium — minor versions release every ~4–8 weeks |
| `tauri.conf.json` schema | `https://schema.tauri.app/config/2` | WebFetch + `$schema` IDE validation | Low — v2 schema is stable |
| `tauri.conf.json` reference | `https://v2.tauri.app/reference/config/` | WebFetch live doc | Low |
| Vite configuration | `https://v2.tauri.app/start/frontend/vite/` | WebFetch live doc | Low |
| main.rs / lib.rs pattern | `https://v2.tauri.app/start/project-structure/` + `https://v2.tauri.app/develop/calling-rust/` | WebFetch live doc | Low — this pattern is stable and motivated by mobile support |
| Capabilities system | `https://v2.tauri.app/security/capabilities/` | WebFetch live doc | Medium — permissions evolve as plugins are added |
| v1→v2 migration diff | `https://v2.tauri.app/start/migrate/from-tauri-1/` | WebFetch live doc | Low — reference doc for completed migration |
| tauri crate latest version | `https://docs.rs/crate/tauri/latest` | Check docs.rs before pinning | High — releases ~monthly |
| tauri-build latest version | `https://docs.rs/crate/tauri-build/latest` | Check docs.rs before pinning | High — tracks tauri crate minor versions |

---

## Sources

- [Tauri v2 Create Project](https://v2.tauri.app/start/create-project/)
- [Tauri v2 Project Structure](https://v2.tauri.app/start/project-structure/)
- [Tauri v2 Configuration Files](https://v2.tauri.app/develop/configuration-files/)
- [Tauri v2 Configuration Reference](https://v2.tauri.app/reference/config/)
- [Tauri v2 Vite Frontend Setup](https://v2.tauri.app/start/frontend/vite/)
- [Tauri v2 Calling Rust from Frontend](https://v2.tauri.app/develop/calling-rust/)
- [Tauri v2 Capabilities](https://v2.tauri.app/security/capabilities/)
- [Tauri v2 Using Plugin Permissions](https://v2.tauri.app/learn/security/using-plugin-permissions/)
- [Tauri v2 Migrate from v1](https://v2.tauri.app/start/migrate/from-tauri-1/)
- [Tauri v2 Core Ecosystem Releases](https://v2.tauri.app/release/)
- [tauri crate on docs.rs](https://docs.rs/crate/tauri/latest)
- [tauri-build crate on docs.rs (v2.5.3)](https://docs.rs/crate/tauri-build/latest)
- [Tauri v2 state example tauri.conf.json](https://github.com/tauri-apps/tauri/blob/dev/examples/state/tauri.conf.json)
