# Markdown Brief Viewer — Tier 1 & 2 Plan

## Problem

The `BriefView` component renders the research brief inside a `<pre>` tag, displaying raw markdown text. The Rust backend (`render_brief()`) produces well-structured GFM with headings, tables, bold, italics, blockquotes, and lists — all of which are wasted as plain text. Users can't scan structure, distinguish themes, or interact with the content.

## Requirements

- **P0-1** Render markdown as formatted HTML (headings, bold, italic, lists, blockquotes, horizontal rules)
- **P0-2** Render GFM tables as styled `<table>` elements
- **P0-3** Prose typography (readable font sizes, spacing, max-width) matching the app's dark theme
- **P0-4** Loading skeleton while brief is fetching
- **P1-1** Syntax highlighting for any fenced code blocks
- **P1-2** Copy-to-clipboard button on code blocks
- **P1-3** Auto-generated table of contents from headings
- **P1-4** Collapsible theme sections (h2-level)

## Non-goals

- Tailwind CSS setup (disproportionate for one component)
- Dark/light theme toggle (Tier 2 item deferred — app is dark-only)
- Print/export to PDF (Tier 3)
- Editing markdown in the UI
- User-authored markdown input

## Constraints

- CSP: `default-src 'self'; connect-src ipc: http://ipc.localhost` — no `style-src 'unsafe-inline'`. All CSS must be statically imported (Vite bundles as `<link>`, which works under `default-src 'self'`).
- No CSS framework — app uses inline styles throughout. We'll introduce a single scoped CSS file.
- ESM-only project (`"type": "module"`). react-markdown v10 is ESM-only — compatible.
- Biome linter, strict TypeScript (`noUncheckedIndexedAccess`, `noUnusedLocals`).
- Vite build target: `safari13`.

---

## Summary

Replace `BriefView`'s `<pre>` with `<ReactMarkdown>` (react-markdown v10 + remark-gfm + rehype-highlight), add a scoped prose CSS file for dark-mode typography, then layer on UX enhancements (copy button, loading skeleton, ToC, collapsible sections) via react-markdown's `components` prop. No CSS framework needed — one CSS file scoped under `.brief-prose`.

## Current State

- `BriefView` (`ArtifactViewer.tsx:138-161`): renders `brief` string in a `<pre>` with inline styles
- Brief arrives via `getBrief(sessionId)` → Tauri `invoke("get_brief")` → Rust `render_brief()` → GFM string
- No markdown library in the project. No CSS files. All styling is inline `style={{}}` objects.
- Tests use RTL + vitest, mock Tauri `invoke` calls.

## Files to Change

| File | Change |
|------|--------|
| `package.json` | Add `react-markdown`, `remark-gfm`, `rehype-highlight` |
| `src/features/artifact-viewer/ArtifactViewer.tsx` | Replace `BriefView` `<pre>` with `<ReactMarkdown>`, add loading skeleton, wire custom components |

## Files to Create

| File | Purpose | Pattern follows |
|------|---------|-----------------|
| `src/features/artifact-viewer/brief-prose.css` | Scoped dark-mode typography for rendered markdown | N/A (first CSS file; scoped under `.brief-prose`) |
| `src/features/artifact-viewer/components/CodeBlock.tsx` | Custom `pre` component with copy-to-clipboard | Existing inline-style component pattern in ArtifactViewer |
| `src/features/artifact-viewer/components/BriefToC.tsx` | Heading collector + clickable table of contents sidebar | Same pattern |
| `src/features/artifact-viewer/components/CollapsibleSection.tsx` | Expandable `<details>/<summary>` wrapper for h2 sections | Same pattern |
| `src/__tests__/BriefView.test.tsx` | Unit tests for markdown rendering, copy button, ToC | Follows `App.test.tsx` patterns (RTL + vitest + mocked invoke) |

---

## Milestones

### Phase 1: Core Rendering (Tier 1)

- [ ] **M1: Basic markdown rendering** — Replace `<pre>` with `<ReactMarkdown>` + remark-gfm + rehype-highlight, add `brief-prose.css` with dark-mode typography. Verify headings, tables, lists, blockquotes, bold/italic all render correctly.
  - [ ] Step 1 — Install deps: `pnpm add react-markdown remark-gfm rehype-highlight` → verify: `pnpm ls react-markdown`
  - [ ] Step 2 — Update Tauri CSP to add `style-src 'self' 'unsafe-inline'` (Vite dev injects CSS via `<style>` tags, blocked without this) → verify: grep csp in tauri.conf.json
  - [ ] Step 3 — Create `src/features/artifact-viewer/brief-prose.css` with scoped dark-mode typography → verify: file exists
  - [ ] Step 4 — Replace `BriefView` in `ArtifactViewer.tsx`: swap `<pre>` for `<Markdown>` with remarkGfm + rehypeHighlight, import CSS → verify: `pnpm build && pnpm test`
  Commit: "feat: add markdown rendering for research brief"

- [ ] **M2: Copy button + loading skeleton** — Add `CodeBlock` component with clipboard copy. Replace "Loading session artifacts..." text with animated skeleton bars. Add unit tests for BriefView.

### Phase 2: Navigation & Interaction (Tier 2)

- [ ] **M3: Table of contents** — Add a `BriefToC` sidebar that auto-collects h2/h3 headings and provides smooth-scroll navigation. Layout: ToC on left, brief content on right.

- [ ] **M4: Collapsible sections** — Override h2 rendering with `CollapsibleSection` component. First 3 sections expanded by default, rest collapsed. "Expand all / Collapse all" toggle.

---

## Testing Strategy

| Milestone | Tests |
|-----------|-------|
| M1 | Render `<BriefView brief={sampleMarkdown} />`, assert `getByRole("heading", { level: 1 })`, `getByRole("table")`, `querySelector("blockquote")` |
| M2 | Assert copy button exists in code blocks, mock `navigator.clipboard.writeText`, assert "Copied!" flash. Assert skeleton renders when `brief` is null and loading. |
| M3 | Render with multi-heading markdown, assert ToC links exist, assert `scrollIntoView` called on click |
| M4 | Assert `<details>` elements render for h2 sections, assert first 3 are `open`, assert expand-all toggle works |

## Risks

| Risk | Mitigation |
|------|------------|
| CSP blocks highlight.js CSS at runtime | Import statically via `import 'highlight.js/styles/...'` — Vite bundles as `<link>`, allowed by `default-src 'self'`. If it fails, add `style-src 'self' 'unsafe-inline'` to CSP. |
| react-markdown ESM import issues in vitest | Project already ESM (`"type": "module"`), vitest v3 handles ESM. If issues arise, add `deps.inline` in vitest config. |
| TypeScript strict mode vs react-markdown component props | Use explicit type annotations on custom component props; null-guard `children` with `?? null`. |
| Brief markdown structure changes in future | Tests use a realistic sample string; `render_brief()` is deterministic — changes there should update the sample. |

## Open Questions

None — ready to start.
