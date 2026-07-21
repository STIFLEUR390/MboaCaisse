# MboaCaisse (fork of Nuxtor)

Nuxt 4 + Tauri 2 desktop app. SSR off, Axum embedded in Tauri process,
SQLite DB (file `mboacaisse.db`), LAN-only (no internet).

## Commands

```sh
bun run tauri:dev        # dev launcher — finds free port 3000–3099, then starts Tauri
bun run dev              # Nuxt standalone dev server (no Tauri)
bun run tauri:build      # prod build (nuxt generate + tauri build)
bun run tauri:build:debug
bun run lint             # ESLint flat config (@antfu/eslint-config), --fix enabled
bun run generate         # nuxt generate (used as beforeBuildCommand in tauri.conf.json)
bun run bump             # bumpp — updates package.json, src-tauri/tauri.conf.json, src-tauri/Cargo.toml (no commit/tag/push)
```

`bun` enforced via `packageManager` + `preinstall: only-allow bun`.

## Architecture

| Path | Role |
|------|------|
| `app/` | Nuxt 4 frontend — **only** `app/` is included via `.nuxtignore` |
| `src-tauri/` | Tauri 2 Rust backend |
| `app/modules/tauri.ts` | Custom Nuxt module — auto-imports Tauri APIs as `useTauri<Prefix>` |
| `scripts/tauri-dev.ts` | Custom dev runner — finds free port, overrides tauri devUrl |
| `src-tauri/capabilities/main.json` | Tauri permissions (shell, fs, notification, os, store) |

No CI/CD workflows. No test framework.

## Rust backend

- Crate name (edition 2021) in `Cargo.toml`: `nuxtor_lib` (lib), `mboacaisse` (package)
- Entry: `main.rs` calls `nuxtor_lib::run()`. Plugins registered in `lib.rs`.
- Tauri features: `unstable`, `tray-icon` on desktop (Quit menu in tray).
- Window min: 375×812. CSP: null.

## Conventions

- **SSR disabled** (`ssr: false` in nuxt.config) — mandatory for Tauri
- **App dir**: `app/` (Nuxt 4), `.nuxtignore` ignores everything else
- **ESLint** (@antfu/eslint-config): tabs, double quotes, semicolons, `vue/block-order: [template, script, style]`. `style/indent` off in Vue files (`vue/script-indent` handles it)
- **No Prettier** — ESLint handles all formatting
- **zod** auto-imported: `z`, `zInfer` (type-only)
- **Env vars**: `VITE_` and `TAURI_` prefixes, dev HMR ws on port 3001 when `TAURI_DEV_HOST` set
- **TypeScript**: `typescript` resolution aliased to `tslite` (package.json resolutions)
- **Nuxt devtools**: disabled, `typedPages: true` (experimental)
- **Nuxt UI v4**: wraps `UApp` in `app.vue`, config uses green primary + zinc neutral
- **Version bump**: `bun run bump` updates 3 files, no commit/tag/push
- **`bun run dev`** runs Nuxt standalone; **`bun run tauri:dev`** runs Tauri (which wraps Nuxt)

## Tauri plugin auto-imports

Do not manually import from `@tauri-apps/plugin-*` or `@tauri-apps/api/*`. The custom module re-exports everything as `useTauri<Prefix><ExportedName>`:

| Prefix | Source package |
|--------|---------------|
| `useTauriApp*` | `@tauri-apps/api/app` |
| `useTauriWebviewWindow*` | `@tauri-apps/api/webviewWindow` |
| `useTauriShell*` | `@tauri-apps/plugin-shell` |
| `useTauriOs*` | `@tauri-apps/plugin-os` |
| `useTauriNotification*` | `@tauri-apps/plugin-notification` |
| `useTauriFs*` | `@tauri-apps/plugin-fs` |
| `useTauriStore*` | `@tauri-apps/plugin-store` |

Example: `await useTauriAppGetTauriVersion()` instead of `getTauriVersion`.

## Adding a new Tauri plugin

1. Add npm dep + Rust crate to `src-tauri/Cargo.toml`.
2. Register in `src-tauri/src/lib.rs` via `.plugin(...)`.
3. Grant permissions in `src-tauri/capabilities/main.json`.
4. Add entry to `tauriModules` array in `app/modules/tauri.ts`.
