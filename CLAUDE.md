<!-- GSD:project-start source:PROJECT.md -->
## Project

**Subtitle**

A cross-platform desktop subtitle overlay app built with Tauri v2 and Svelte 5. It captures audio from the microphone or system audio stream, transcribes speech in real-time using configurable ASR engines (on-device or remote), overlays the text as a transparent floating window on top of other applications, and optionally translates subtitles using configurable translation backends (local LLM, dedicated translation models, or remote APIs). Designed primarily for live event captioning.

**Core Value:** Real-time, accurate subtitle overlay that works across any application -- users see live captions floating on screen regardless of what they're doing.

### Constraints

- **Tech stack:** Tauri v2 (Rust) + Svelte 5 (TypeScript) + Tailwind CSS v4 -- already in place, must build on this
- **ASR engines:** ONNX Runtime and Vosk for on-device; OpenAI-compatible API for remote
- **Translation:** Ollama for local LLM, ONNX-based NLLB/M2M for dedicated local models, OpenAI-compatible API for remote
- **Platforms:** Linux (primary), macOS, Windows -- all three must work
- **Audio:** PipeWire + PulseAudio fallback on Linux; platform-native APIs on macOS/Windows
- **No server component:** Fully offline-capable for on-device engines; remote engines optional
<!-- GSD:project-end -->

<!-- GSD:stack-start source:codebase/STACK.md -->
## Technology Stack

## Runtime & Language Versions
- TypeScript ~6.0.2 (target: ES2023, module: ESNext)
- Svelte ^5.55.2 (using runes: `$state`, `$derived`, `$effect`, `$props`)
- Rust Edition 2021 (minimum rust-version: 1.77.2)
- Crate type: `staticlib`, `cdylib`, `rlib`
- Node.js v24.14.1
- pnpm (installed via mise shim, lockfile version 9.0)
## Frontend Stack
- Svelte 5 (runes API exclusively -- no legacy Svelte stores)
- Mounting via `svelte/mount` in `src/main.ts`
- No component library. All UI is hand-built Svelte 5 components in `src/components/`
- SVG icons inline (no icon library)
- Tailwind CSS v4 (^4.2.2) via `@tailwindcss/vite` plugin
- Global styles in `src/app.css` with `@import "tailwindcss"`
- Component-scoped `<style>` blocks for specific CSS (animations, custom controls)
- Dark theme only (black background, white text)
- Svelte 5 runes (`$state`, `$derived`, `$effect`) in `src/lib/speech.svelte.ts`
- Component-level state via `$state` in `src/app.svelte`
- No external state management library
- Vite ^8.0.4
- `@sveltejs/vite-plugin-svelte` ^7.0.0
- `@tailwindcss/vite` ^4.2.2
- Path alias: `$lib` -> `./src/lib`
- Dev server: port 5173 with HMR on 5174
## Backend Stack
- Tauri v2 (^2.10.3) -- desktop application shell
- `tauri-build` ^2.5.6 (build dependency)
- Rust (Edition 2021)
- None. No persistent storage. All state is in-memory on the frontend.
- Not applicable
## Dev Tools
- pnpm (lockfile version 9.0, lockfile present)
- No `.npmrc` file
- Not detected (no ESLint, Biome, or other linter config)
- Not detected (no Prettier config)
- Not detected (no test framework, no test files, no test scripts in `package.json`)
- TypeScript ~6.0.2 with strict settings:
## Key Dependencies
| Package | Version | Purpose |
|---------|---------|---------|
| `svelte` | ^5.55.2 | UI framework (runes API) |
| `@sveltejs/vite-plugin-svelte` | ^7.0.0 | Vite plugin for Svelte compilation + preprocessing |
| `tailwindcss` | ^4.2.2 | Utility-first CSS framework |
| `@tailwindcss/vite` | ^4.2.2 | Vite plugin for Tailwind CSS v4 |
| `vite` | ^8.0.4 | Build tool and dev server |
| `typescript` | ~6.0.2 | Type checking |
| `@tauri-apps/cli` | ^2.10.1 | Tauri CLI for dev/build commands |
| `@types/node` | ^24.12.2 | Node.js type definitions |
| `tauri` (Rust) | 2.10.3 | Tauri core framework (Rust side) |
| `tauri-plugin-log` (Rust) | 2 | Structured logging in debug builds |
| `serde` / `serde_json` (Rust) | 1.0 | Serialization (available but not actively used yet) |
## Configuration
- Target: ES2023
- Module: ESNext with bundler resolution
- Types: `vite/client`
- Plugins: `tailwindcss()`, `svelte()`
- Path alias: `$lib` -> `./src/lib`
- Env prefix: `VITE_`, `TAURI_`
- HMR support for Tauri dev host
- Preprocessor: `vitePreprocess()` only
- Product: "Real-time Subtitles" v0.1.0
- Identifier: `com.tauri.dev`
- Window: 800x600, resizable
- CSP: `default-src 'self'; connect-src https://api.deepinfra.com; style-src 'self' 'unsafe-inline'`
- Bundle targets: all (macOS, Windows, Linux)
## Platform Requirements
- Node.js (v24.x detected)
- pnpm package manager
- Rust toolchain (Edition 2021, min 1.77.2)
- Tauri v2 system dependencies (webkit2gtk on Linux, WebKit on macOS, WebView2 on Windows)
- Tauri bundles native installers for all platforms
- No server component -- fully desktop application
- Requires internet connection for:
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

## Code Style
### TypeScript
- **Target:** ES2023 with ESNext modules (configured in `tsconfig.app.json`)
- **Strictness:** `noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch` enabled
- **Module syntax:** `verbatimModuleSyntax: true` -- use `import type` for type-only imports
- **Module resolution:** `bundler` mode with `allowImportingTsExtensions: true`
- **No emit:** `noEmit: true` -- TypeScript is only for type checking; Vite handles compilation
### Svelte 5 Patterns
- **Script block:** Always `<script lang="ts">` at the top of the component
- **Props:** Declared using `interface Props { ... }` followed by `let { ... }: Props = $props()`
- **Reactive state:** Use `$state()` for mutable state, `$derived()` for computed values, `$effect()` for side effects
- **No `export let`** -- props are declared via `$props()` destructuring, never `export let`
### Rust
- **Edition:** Rust 2021, minimum version 1.77.2
- **Naming:** Standard snake_case for functions and variables (e.g., `app_lib::run()`)
- **Error handling:** Uses `expect()` for startup failures in `src-tauri/src/lib.rs`; no custom error types defined yet
- **Module structure:** `main.rs` calls `app_lib::run()` from `lib.rs`; crate name is `app_lib`
- **Logging:** Uses `log` crate with `tauri_plugin_log`, configured at `Info` level in debug builds only
### CSS/Tailwind
- **Tailwind CSS v4** via `@tailwindcss/vite` plugin -- no `tailwind.config.*` file needed
- **Global styles** in `src/app.css` with `@import "tailwindcss";` plus custom resets and keyframe animations
- **Mixed approach:** Components use a combination of:
- **Arbitrary values** used extensively in Tailwind: `bg-[rgba(255,255,255,0.08)]`, `rounded-t-[20px]`, etc.
- **Color palette:** Custom dark theme with semi-transparent white values (e.g., `rgba(255, 255, 255, 0.08)` for backgrounds, `text-white/90` for text)
- **Keyframe animations** defined in `src/app.css` and referenced by name in component styles (e.g., `slideUp`, `pulse`, `pulseDot`, `dotBounce`)
## Component Patterns
### Props Interface
- `onStart`, `onStop`, `onRetry`, `onClose`
- `onSettings`, `onLanguageChange`, `onTargetLangChange`, `onSubtitlePositionChange`
- `onchange` (lowercase) for the Dropdown's native-style change handler
### Event Handling
- Parent-to-child communication via callback props (no Svelte `createEventDispatcher`)
- DOM events use inline handlers: `onclick={handler}` or `onclick={(e) => ...}`
- Type assertions for DOM events: `(e.target as HTMLSelectElement).value`
- Event propagation stopped with `onclick={(e) => e.stopPropagation()}`
### Component Structure Order
### Conditional Rendering
### List Rendering
## State Management Patterns
### Svelte 5 Runes
- **`$state()`** for reactive mutable state -- used in both components and the `Speech` class
- **`$derived()`** for computed values -- e.g., `sourceLabel`, `targetLabel` in `src/app.svelte`
- **`$effect()`** for reactive side effects -- e.g., translation trigger in `src/app.svelte`
### Singleton Service Pattern
- Exported as a single instance: `export const speech = new Speech();`
- Uses `$state()` fields directly on the class for reactive properties
- Methods are arrow functions assigned to class fields (preserving `this` context): `start = () => { ... }`
- Private fields use standard `private` keyword: `private recognition`, `private stopping`
### Application State Location
- **Global app state:** `src/app.svelte` manages top-level state (`targetLang`, `translations`, `settingsOpen`, `subtitlePosition`)
- **Speech state:** Encapsulated in `src/lib/speech.svelte.ts` (`status`, `subtitles`, `language`, `errorMessage`)
- **No shared store pattern** -- no Svelte stores (`writable`, `readable`, `derived`) are used; everything uses runes
### Data Flow
- Top-down via props from `src/app.svelte` to child components
- Child-to-parent via callback props (onXxx functions)
- Translation state (`translations` Record) managed in `src/app.svelte` and passed down
## Import/Export Patterns
### Import Order
### Path Aliases
- `$lib` resolves to `./src/lib`
### Export Patterns
- **Types:** `export type` and `export interface` for shared types (`src/lib/types.ts`)
- **Constants:** `export const` for shared data (`src/lib/languages.ts`)
- **Singletons:** `export const speech = new Speech()` for service instances
- **Functions:** `export function` for utility functions (`getLangName` in `src/lib/languages.ts`)
- **Components:** Default export via Svelte's implicit behavior (no explicit `export default` needed in `.svelte` files)
### File Naming Convention for Library Files
## Naming Conventions
### Files
- **Components:** PascalCase `.svelte` -- `IdleScreen.svelte`, `ListeningScreen.svelte`, `ErrorScreen.svelte`, `Settings.svelte`, `SubtitleLine.svelte`, `Dropdown.svelte`
- **Library modules:** camelCase `.ts` or `.svelte.ts` -- `languages.ts`, `types.ts`, `speech.svelte.ts`
- **Entry point:** `main.ts`
- **App component:** `app.svelte` (lowercase)
- **Styles:** `app.css` (lowercase)
### Directories
- `src/components/` -- PascalCase files, plural directory name
- `src/lib/` -- lowercase files, shared library code
### Variables and Functions
- **camelCase** for all variables and functions: `targetLang`, `translations`, `settingsOpen`, `translateLine()`, `getLangName()`
- **SCREAMING_SNAKE_CASE** for constants: `MAX_SUBTITLES`, `SOURCE_LANGUAGES`, `TARGET_LANGUAGES`, `LANG_NAMES`
- **PascalCase** for types and interfaces: `AppStatus`, `SubtitleLine`, `Props`, `Speech`
### CSS Classes
- Tailwind utilities in HTML templates (no custom class names)
- Scoped CSS uses lowercase descriptive names: `.status-bar`, `.status-dot`, `.lang-badge`, `.subtitle-container`, `.slider`
- BEM-like descriptive naming but not strict BEM
## Git Conventions
### Branch Strategy
- Single branch: `main` -- all commits are on main, no feature branches observed
### Commit Message Format
- `feat:` -- New features (most common)
- `fix:` -- Bug fixes
- `refactor:` -- Code restructuring
### No Linting or Formatting Enforcement
- No ESLint configuration found
- No Prettier configuration found
- No Biome configuration found
- No pre-commit hooks configured (no `.husky/`, no `lint-staged`)
- Code style is maintained manually
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

## High-Level Architecture
- Thin-server architecture: Rust backend is a passive window host, not a logic layer
- Frontend is the entire application: state, API calls, speech recognition all happen client-side
- No database, no local filesystem access, no IPC beyond Tauri's built-in windowing
- Single-window desktop app (800x600, resizable)
## Frontend Architecture
### Component Hierarchy
```
```
### Data Flow
- `'idle'` -- Initial/waiting state. Shows IdleScreen with Start button.
- `'listening'` -- Active speech recognition. Shows ListeningScreen with live subtitles.
- `'error'` -- Error occurred. Shows ErrorScreen with retry option.
- `idle -> listening`: User clicks Start, calls `speech.start()`
- `listening -> idle`: User clicks Stop, calls `speech.stop()`
- `listening -> error`: Speech recognition error fires
- `error -> listening`: User clicks Try Again, calls `speech.start()`
### State Management
- `$state()` -- Reactive state declared in the `Speech` class (`status`, `subtitles`, `language`, `errorMessage`) and in `app.svelte` (`targetLang`, `translations`, `settingsOpen`, `subtitlePosition`)
- `$derived()` -- Computed values for language labels (`sourceLabel`, `targetLabel`)
- `$effect()` -- Side-effect that watches `speech.subtitles` and triggers translation for new finalized lines
### Translation Pipeline
- `translatedIds` Set -- prevents re-translating the same line
- `inFlight` Set -- prevents duplicate in-progress requests for the same ID
- Context window: keeps last 3 source lines for translation context
### Speech Recognition
- Runs in `continuous` mode with `interimResults` enabled
- Interim results are shown with IDs prefixed `interim-` and replaced when final results arrive
- Auto-restarts on unexpected `onend` events (unless explicitly stopped)
- Language can be changed mid-session via `setLanguage()` which stops and restarts recognition after 100ms delay
- Caps subtitle history at 12 lines (`MAX_SUBTITLES`)
## Backend Architecture
### Rust Module Structure
- `src-tauri/src/main.rs` -- Entry point. Calls `app_lib::run()`. Includes `windows_subsystem = "windows"` attribute for release builds.
- `src-tauri/src/lib.rs` -- `run()` function that constructs the Tauri app. Only custom setup: logging plugin in debug mode.
### Tauri Configuration
- Single window: 800x600, resizable, not fullscreen
- CSP policy restricts connections to `https://api.deepinfra.com` only
- Frontend served from `../dist` (production) or `http://localhost:5173` (dev)
- Build commands use `pnpm`
- Bundle targets: all platforms
### Tauri Commands
- `tauri-plugin-log` -- Debug logging (debug builds only)
- `core:default` -- Standard Tauri permissions (from `src-tauri/capabilities/default.json`)
### Frontend-Backend Communication
- No `invoke()` calls from frontend to Rust
- No event channel communication
- No filesystem access from frontend
- All external API calls go directly from the webview
## Key Design Decisions
### 1. Thin Rust Shell
### 2. Browser Speech API in a Desktop WebView
### 3. Direct API Calls from Frontend
### 4. Singleton Speech Instance
### 5. Context-Aware Translation
<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->
## Project Skills

No project skills found. Add skills to any of: `.claude/skills/`, `.agents/skills/`, `.cursor/skills/`, or `.github/skills/` with a `SKILL.md` index file.
<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
