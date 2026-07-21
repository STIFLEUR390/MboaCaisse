---
project_name: "MboaCaisse"
user_name: "Herold"
date: "2026-07-21"
sections_completed:
  - technology_stack
  - language_rules
  - framework_rules
  - testing_rules
  - code_quality_and_style
  - development_workflow
  - critical_dont_miss
status: complete
rule_count: 42
optimized_for_llm: true
existing_patterns_found: 8
---

# Project Context for AI Agents

---

## Technology Stack & Versions

### Core
- **Nuxt** 4.3.0 — SSR disabled (mandatory for Tauri)
- **Vue** 3.5.27
- **Vue Router** 4.6.4
- **TypeScript** 5.9.3 (resolution: `typescript` → `tslite` via package.json resolutions)
- **Bun** 1.3.6 — sole package manager (enforced via `preinstall: only-allow bun`)

### Desktop
- **Tauri** 2.9.5 — features: `unstable`, `tray-icon` on desktop
- **Rust** edition 2021 — crate `nuxtor_lib` (lib), `mboacaisse` (bin)
- **Tauri plugins**: shell, notification, os, fs, store

### Frontend Dependencies
- **Nuxt UI** 4.4.0 — green primary, zinc neutral
- **Zod** 4.3.6 — auto-imported as `z` (value) and `zInfer` (type-only)
- **@vueuse/core** 14.1.0 + @vueuse/nuxt
- **nuxt-svgo** 4.2.6 — auto-import SVG from `@/assets/`

### Tooling
- **ESLint** 9.39.2 + @antfu/eslint-config 7.2.0 — tabs, double quotes, semicolons, no Prettier
- **bumpp** 10.4.0 — version bump (updates 3 files, no commit/tag/push)

### Rust Dependencies
- `tauri` 2.9.5, `tauri-build` 2.5.3
- `serde` 1 + `serde_json` 1
- Plugins: `tauri-plugin-{shell,notification,os,fs,store}`

## Critical Implementation Rules

### Language-Specific Rules

- **Vue 3 Composition API** only — always use `<script lang="ts" setup">`, never Options API
- **State management**: use `ref()` / `computed()` for local state; no Pinia store exists yet
- **Zod schemas**: define with `z.object({...})`, extract type via `zInfer<typeof schema>`
- **Tabs** for indentation everywhere (ESLint enforces `style/indent: tab`)
- **Semicolons** required (`style/semi: ["error", "always"]`)
- **No `any`** — prefer `unknown` + narrow or proper types
- **No manual imports** from `@tauri-apps/*` — use auto-imported `useTauri<Prefix><Name>` functions
- **`definePageMeta`** required in every page — must include `name`, `icon`, `description`, `category`, `layout`

### Framework-Specific Rules

#### Nuxt 4
- **App dir**: all source lives in `app/`; `.nuxtignore` excludes everything else
- **SSR disabled** (`ssr: false` in `nuxt.config.ts`) — mandatory for Tauri compatibility
- **Pages**: `app/pages/`, Nuxt 4 file-based routing with `typedPages: true`
- **Layouts**: 3 layouts (`default.vue`, `blank.vue`, `home.vue`) — select via `definePageMeta({ layout })`
- **Page transitions**: enabled globally (`page: { name: "page", mode: "out-in" }`)
- **Layout transitions**: enabled globally (`layout: { name: "layout", mode: "out-in" }`)
- **Composables**: place in `app/composables/`, auto-imported by Nuxt — no manual import needed
- **App config**: define in `app/app.config.ts`; access via `useAppConfig()`
- **Router options**: custom `scrollBehavior` in `app/router.options.ts` (100ms delay, hash support)
- **Modules**: custom modules in `app/modules/` (dir overridden in nuxt.config)
- **SVG**: auto-import from `@/assets/` via nuxt-svgo (e.g. `<SvgoLogo />`)
- **Devtools**: disabled
- **HMR**: WS on port 3001 when `TAURI_DEV_HOST` is set

#### Nuxt UI v4
- **Wrapper**: `UApp` must wrap app content (in `app.vue`)
- **Theme**: green primary, zinc neutral (configured in `app.config.ts`)
- **Component slots**: cursor-pointer on button/navigationMenu accordion trigger
- **Form pattern**: `UForm` + `UFormField` for labels + validation
- **Icons**: lucide collection (e.g. `i-lucide-terminal`, `i-lucide-folder`)

### Code Quality & Style Rules

- **ESLint only** — no Prettier; @antfu/eslint-config handles all formatting
- **Tabs** for indentation
- **Double quotes** (`style/quotes: double`)
- **Semicolons** required (`style/semi: ["error", "always"]`)
- **Vue SFC block order**: `<template>` → `<script>` → `<style>` (enforced by `vue/block-order`)
- **Comma dangle**: never (`style/comma-dangle: ["warn", "never"]`)
- **Arrow parens**: always (`style/arrow-parens: ["error", "always"]`)
- **Brace style**: 1tbs (`style/brace-style: ["warn", "1tbs"]`)
- **Vue files**: `style/indent` set to `off` — `vue/script-indent` handles Vue script indentation (tab, baseIndent: 1)
- **Naming**: PascalCase for components/files, camelCase for variables/functions
- **Top-level functions**: allowed (`antfu/top-level-function: off`)
- **`console.*`**: allowed (`no-console: off`)
- **`no-new-func`**: allowed (`off`)
- **`curly`**: off (no braceless-statement enforcement)

### Development Workflow Rules

- **`bun run tauri:dev`** — dev launcher: finds free port 3000–3099, overrides tauri devUrl, starts Tauri
- **`bun run dev`** — Nuxt standalone dev server (no Tauri)
- **`bun run tauri:build`** — prod: `nuxt generate` then `tauri build`
- **`bun run lint`** — ESLint with `--fix` enabled
- **`bun run bump`** — version bump: updates `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` (no commit/tag/push)
- **`bun run generate`** — used as `beforeBuildCommand` in tauri.conf.json
- **Commits**: manual only — no automated committing by agents
- **CI/CD**: none configured
- **Testing**: no test framework configured — add tests when implementing new features

### Critical Don't-Miss Rules

#### Anti-Patterns
- **NEVER** manually `import` from `@tauri-apps/api/*` or `@tauri-apps/plugin-*` — use auto-imported `useTauri<Prefix><Name>` functions instead
- **NEVER** use `fetch()` for local API calls — use Tauri IPC commands or the embedded Axum server
- **NEVER** re-enable SSR (`ssr: true`) — breaks Tauri compatibility
- **NEVER** use `npm`/`pnpm`/`yarn` — `bun` is the only allowed package manager
- **NEVER** add a Tauri plugin without completing all 4 required steps: npm dep, Rust crate + `lib.rs` plugin registration, `capabilities/main.json` permissions, `app/modules/tauri.ts` entry

#### Constraints
- **LAN-only** — no internet access; all data is local SQLite
- **SQLite DB file**: `mboacaisse.db` — no remote database
- **Window min size**: 375×812
- **CSP**: null (no Content Security Policy restrictions)
- **Tauri features**: `unstable` enabled, `tray-icon` on desktop (Quit menu in tray)

#### Plugin Addition Checklist
1. Add npm dependency (e.g. `bun add @tauri-apps/plugin-xyz`)
2. Add Rust crate to `src-tauri/Cargo.toml`
3. Register plugin in `src-tauri/src/lib.rs` via `.plugin(...)`
4. Grant permissions in `src-tauri/capabilities/main.json`
5. Add entry to `tauriModules` array in `app/modules/tauri.ts`

---

## Usage Guidelines

**For AI Agents:**
- Read this file before implementing any code
- Follow ALL rules exactly as documented
- When in doubt, prefer the more restrictive option
- Update this file if new patterns emerge

**For Humans:**
- Keep this file lean and focused on agent needs
- Update when technology stack changes
- Review quarterly for outdated rules
- Remove rules that become obvious over time

Last Updated: 2026-07-21
