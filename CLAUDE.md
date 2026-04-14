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
- Rust Edition 2021 (minimum rust-version: 1.85)
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
- **Persistent settings** via `tauri-plugin-store` (JSON file `settings.json`)
- **Secure API key storage** via `tauri-plugin-stronghold` (encrypted vault with argon2 KDF)
- **Audio capture** via `cpal` ^0.17 (cross-platform, captures at native rate, resamples to 16kHz mono)
- **On-device ASR** via `vosk` ^0.3.1 bindings
- **Remote ASR** via `reqwest` ^0.12 (OpenAI-compatible Whisper API, multipart WAV upload)
- **Async runtime:** `tokio` ^1 (features: sync, time, rt, macros)
- **Unique IDs:** `uuid` ^1.0 (v4 feature)
- **Keyring:** `keyring` ^3 (deprecated, replaced by Stronghold)
## Dev Tools
- pnpm (lockfile version 9.0, lockfile present)
- No `.npmrc` file
- **Biome** for TS/JS/JSON linting + formatting (`biome.json`)
- **Prettier** for Svelte formatting (`.prettierrc`, `.prettierignore`)
- **Lefthook** for git hooks (`lefthook.yml`): pre-commit (typecheck, biome, prettier, rustfmt), pre-push (vitest, cargo test, typecheck, audit)
- **Vitest** for frontend tests (`src/lib/__tests__/`, `vitest.config.ts`)
- **Cargo test** for Rust unit tests (module-level `#[cfg(test)]` blocks)
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
| `@tauri-apps/api` | ^2.10.1 | Tauri JS API (invoke, listen) |
| `@tauri-apps/plugin-stronghold` | ^2.0.0 | Stronghold JS API for frontend key management |
| `@types/node` | ^24.12.2 | Node.js type definitions |
| `tauri` (Rust) | 2.10.3 | Tauri core framework (Rust side) |
| `tauri-plugin-log` (Rust) | 2 | Structured logging in debug builds |
| `tauri-plugin-store` (Rust) | 2 | Persistent JSON key-value store for settings |
| `tauri-plugin-stronghold` (Rust) | 2 | Encrypted vault for API keys (argon2 KDF) |
| `serde` / `serde_json` (Rust) | 1.0 | Serialization for settings, events, API payloads |
| `cpal` (Rust) | 0.17 | Cross-platform audio capture |
| `tokio` (Rust) | 1 | Async runtime, mpsc channels, timed futures |
| `reqwest` (Rust) | 0.12 | HTTP client for remote ASR (rustls-tls, multipart) |
| `vosk` (Rust) | 0.3.1 | On-device speech recognition bindings |
| `uuid` (Rust) | 1.0 | Unique IDs for subtitle events |
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
- Identifier: `com.subtitle.realtime`
- Window: 800x600, resizable
- CSP: `default-src 'self'; connect-src https://api.deepinfra.com; style-src 'self' 'unsafe-inline'`
- Bundle targets: all (macOS, Windows, Linux)
## Platform Requirements
- Node.js (v24.x detected)
- pnpm package manager
- Rust toolchain (Edition 2021, min 1.85)
- Tauri v2 system dependencies (webkit2gtk on Linux, WebKit on macOS, WebView2 on Windows)
- Tauri bundles native installers for all platforms
- No server component -- fully desktop application
- Requires internet connection for:
  - Remote ASR engine (OpenAI-compatible API)
  - Browser engine (Web Speech API)
  - Remote translation engine
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
- **Edition:** Rust 2021, minimum version 1.85
- **Naming:** Standard snake_case for functions and variables (e.g., `app_lib::run()`)
- **Error handling:** `Result<T, String>` for all tauri commands; `.map_err(|e| e.to_string())` pattern
- **Module structure:** `main.rs` calls `app_lib::run()` from `lib.rs`; crate name is `app_lib`
- **Async:** `tokio::spawn` for long-running tasks; `mpsc` channels for audio data flow
- **State management:** Tauri `State<>` wrapper for shared state (AudioState, VoskAsr)
- **Logging:** Uses `log` crate with `tauri_plugin_log`, configured at `Info` level in debug builds only
- **Event emission:** `app.emit("event-name", payload)` for Rust→Frontend streaming
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
- `onEngineChange`, `onTranslationEngineChange`, `onRemoteEndpointChange`, `onApiKeyChange`
- `onOverlayTransparencyChange`, `onFontSizeChange`
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
- **Global app state:** `src/app.svelte` manages top-level state (`targetLang`, `translations`, `settingsOpen`, `subtitlePosition`, `selectedEngine`, `remoteEndpoint`, `apiKey`, `overlayTransparency`, `fontSize`, `translationEngine`)
- **Speech state:** Encapsulated in `src/lib/speech.svelte.ts` (`status`, `subtitles`, `language`, `errorMessage`, `engine`, `remoteEndpoint`, `apiKey`)
- **Settings persistence:** Rust side via `tauri-plugin-store` (JSON), loaded on mount in `app.svelte`
- **API key storage:** `src/lib/stronghold.ts` wraps `@tauri-apps/plugin-stronghold` for encrypted storage
- **No shared store pattern** -- no Svelte stores (`writable`, `readable`, `derived`) are used; everything uses runes
### Data Flow
- Top-down via props from `src/app.svelte` to child components
- Child-to-parent via callback props (onXxx functions)
- Translation state (`translations` Record) managed in `src/app.svelte` and passed down
- Rust→Frontend streaming via Tauri events (`backend://subtitle/*`, `subtitle`, `asr-error`)
## Import/Export Patterns
### Import Order
### Path Aliases
- `$lib` resolves to `./src/lib`
### Export Patterns
- **Types:** `export type` and `export interface` for shared types (`src/lib/types.ts`)
- **Constants:** `export const` for shared data (`src/lib/languages.ts`)
- **Singletons:** `export const speech = new Speech()` for service instances
- **Functions:** `export function` for utility functions (`getLangName` in `src/lib/languages.ts`)
- **Stronghold helpers:** `export async function` for key management (`src/lib/stronghold.ts`)
- **Components:** Default export via Svelte's implicit behavior (no explicit `export default` needed in `.svelte` files)
### File Naming Convention for Library Files
## Naming Conventions
### Files
- **Components:** PascalCase `.svelte` -- `IdleScreen.svelte`, `ListeningScreen.svelte`, `ErrorScreen.svelte`, `Settings.svelte`, `SubtitleLine.svelte`, `Dropdown.svelte`
- **Library modules:** camelCase `.ts` or `.svelte.ts` -- `languages.ts`, `types.ts`, `speech.svelte.ts`, `stronghold.ts`
- **Entry point:** `main.ts`
- **App component:** `app.svelte` (lowercase)
- **Styles:** `app.css` (lowercase)
### Directories
- `src/components/` -- PascalCase files, plural directory name
- `src/lib/` -- lowercase files, shared library code
### Variables and Functions
- **camelCase** for all variables and functions: `targetLang`, `translations`, `settingsOpen`, `translateLine()`, `getLangName`
- **SCREAMING_SNAKE_CASE** for constants: `MAX_SUBTITLES`, `SOURCE_LANGUAGES`, `TARGET_LANGUAGES`, `LANG_NAMES`
- **PascalCase** for types and interfaces: `AppStatus`, `SubtitleLine`, `Props`, `Speech`
### CSS Classes
- Tailwind utilities in HTML templates (no custom class names)
- Scoped CSS uses lowercase descriptive names: `.status-bar`, `.status-dot`, `.lang-badge`, `.subtitle-container`, `.slider`
- BEM-like descriptive naming but not strict BEM
## Git Conventions
### Branch Strategy
- `main` branch for stable releases
- Feature branches: `feat/<descriptive-name>` (e.g., `feat/remoteASR`)
### Commit Message Format
- `feat:` -- New features (most common)
- `fix:` -- Bug fixes
- `refactor:` -- Code restructuring
- Scope references milestone/plan: `feat(05-remote-asr):`
### Linting and Formatting
- **Biome** for TS/JS/JSON files (lint + format, configured in `biome.json`)
- **Prettier** for Svelte files (`.prettierrc`)
- **rustfmt** for Rust files (edition 2021)
- **Lefthook** git hooks: pre-commit runs typecheck + biome + prettier + rustfmt; pre-push runs tests + typecheck + audit
- Install hooks: `pnpm lefthook install`
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

## High-Level Architecture
- **Hybrid architecture:** Rust backend handles audio capture, ASR inference, and event streaming; Svelte frontend handles UI, state, settings, and translation orchestration
- Frontend→Rust via `invoke()` (commands); Rust→Frontend via Tauri `emit()` (events)
- Persistent settings via `tauri-plugin-store`; secure secrets via `tauri-plugin-stronghold`
- Single-window desktop app (800x600, resizable)
## Frontend Architecture
### Component Hierarchy
```
app.svelte
├── IdleScreen (status === 'idle')
│   ├── Start button → speech.start()
│   └── Settings button → settingsOpen = true
├── ListeningScreen (status === 'listening')
│   ├── Status bar (listening indicator, lang badge, stop button)
│   └── SubtitleLine[] (scrolling subtitles with translations)
├── ErrorScreen (status === 'error')
│   ├── Error message display
│   └── Retry button → speech.start()
└── Settings (slide-up panel)
    ├── Source Language dropdown
    ├── Target Language dropdown
    ├── Subtitle Position slider
    ├── Translation Engine dropdown
    ├── Overlay Appearance (transparency, font size)
    └── Advanced: ASR Engine, Remote Endpoint, API Key
```
### Data Flow
- `'idle'` -- Initial/waiting state. Shows IdleScreen with Start button.
- `'listening'` -- Active speech recognition. Shows ListeningScreen with live subtitles.
- `'error'` -- Error occurred. Shows ErrorScreen with retry option.
- `idle -> listening`: User clicks Start, calls `speech.start()`
  - **Browser engine:** Starts Rust audio capture + Web Speech API
  - **Vosk engine:** Starts Rust audio capture → loads Vosk model → starts recognition loop
  - **Remote engine:** Starts Rust audio capture → calls `remote_asr_start` with endpoint + API key
- `listening -> idle`: User clicks Stop, calls `speech.stop()` (stops ASR + audio capture)
- `listening -> error`: Speech recognition error fires
- `error -> listening`: User clicks Try Again, calls `speech.start()`
### State Management
- `$state()` -- Reactive state in `Speech` class (`status`, `subtitles`, `language`, `errorMessage`, `engine`, `remoteEndpoint`, `apiKey`) and `app.svelte` (`targetLang`, `translations`, `settingsOpen`, `subtitlePosition`, etc.)
- `$derived()` -- Computed values for language labels (`sourceLabel`, `targetLabel`)
- `$effect()` -- Side-effect that watches `speech.subtitles` and triggers translation for new finalized lines
### Settings Persistence
- Settings stored in `settings.json` via `tauri-plugin-store`
- Loaded on mount in `app.svelte` via `invoke('settings_get')`
- Saved reactively via `speech.saveSetting(key, value)` → `invoke('settings_set', { key, value })`
- API keys stored in Stronghold encrypted vault, loaded separately via `src/lib/stronghold.ts`
### Translation Pipeline
- `translatedIds` Set -- prevents re-translating the same line
- `inFlight` Set -- prevents duplicate in-progress requests for the same ID
- Context window: keeps last 3 source lines for translation context
### Speech Recognition Engines
- **Browser (`browser`):** Web Speech API in webview. Runs in `continuous` mode with `interimResults`. Auto-restarts on unexpected `onend`.
- **Vosk (`vosk`):** Rust-side recognition loop. Audio → mpsc channel → Vosk recognizer → emit events. Partial/final results via `backend://subtitle/update` and `backend://subtitle/final`.
- **Remote (`remote`):** Rust-side audio chunking with silence detection. Buffers audio → encodes WAV → POST to OpenAI-compatible `/v1/audio/transcriptions` endpoint. Results emitted as `subtitle` events.
- Language can be changed mid-session via `setLanguage()` which stops and restarts recognition after 100ms delay
- Caps subtitle history at 12 lines (`MAX_SUBTITLES`)
## Backend Architecture
### Rust Module Structure
- `src-tauri/src/main.rs` -- Entry point. Calls `app_lib::run()`. Includes `windows_subsystem = "windows"` attribute for release builds.
- `src-tauri/src/lib.rs` -- `run()` function that constructs the Tauri app. Registers plugins (log, store, stronghold), manages shared state (AudioState, VoskAsr, RemoteAsrState), registers all commands.
- `src-tauri/src/commands.rs` -- All tauri commands: audio capture, settings CRUD, Stronghold vault helpers, Vosk model management, event emission helpers. Defines `Settings` struct.
- `src-tauri/src/audio.rs` -- Audio capture via `cpal`. Captures at device native rate, downmixes to mono, resamples to 16kHz (linear interpolation), delivers 480-sample (30ms) chunks via `mpsc` channel.
- `src-tauri/src/vosk.rs` -- Vosk ASR wrapper. Manages model loading, recognition loop (tokio::spawn), stop signal via mpsc channel. Emits `backend://subtitle/update` (partial) and `backend://subtitle/final` events.
- `src-tauri/src/remote_asr.rs` -- Remote ASR via OpenAI-compatible API. Audio buffering with silence detection (RMS threshold), two-layer VAD (pre-send RMS gate at 0.04 + post-response logprob filter at -0.8), WAV encoding, retry logic (3 attempts with backoff), cancellation via `AtomicBool` stop flag. Handles DeepInfra `/v1/inference/` and standard `/v1/audio/transcriptions` URL patterns. Multipart form field: `audio`. Emits `subtitle` and `asr-error` events.
### Tauri Configuration
- Single window: 800x600, resizable, not fullscreen
- Identifier: `com.subtitle.realtime`
- CSP policy restricts connections to `https://api.deepinfra.com` only
- Frontend served from `../dist` (production) or `http://localhost:5173` (dev)
- Build commands use `pnpm`
- Bundle targets: all platforms
### Tauri Commands (registered in `lib.rs`)
- `audio_capture_start` / `audio_capture_stop` -- Start/stop cpal audio capture
- `asr_infer` -- Stub ASR inference (placeholder)
- `translate` -- Stub translation (placeholder)
- `settings_get` / `settings_set` -- Read/write persistent settings
- `test_event_emission` -- Debug helper for event streaming
- `api_key_get` / `api_key_set` -- Deprecated (stronghold JS API used instead)
- `stronghold_get_vault_path` / `stronghold_get_password` -- Stronghold vault bootstrap
- `vosk_load_model` / `vosk_get_model_path` / `vosk_start` / `vosk_stop` -- Vosk lifecycle
- `remote_asr_start` / `remote_asr_stop` / `remote_asr_status` -- Remote ASR lifecycle
### Frontend-Backend Communication
- **invoke() calls** from frontend to Rust for all commands above
- **Event streaming** Rust→Frontend:
  - `backend://subtitle/update` -- Partial/interim transcription (Vosk)
  - `backend://subtitle/final` -- Final transcription result (Vosk)
  - `backend://subtitle/translated` -- Translation result (future use)
  - `backend://subtitle/error` -- Error events
  - `subtitle` -- Remote ASR transcription results
  - `asr-error` -- Remote ASR errors (with `retryable` flag)
- **Stronghold** accessed via JS API directly from frontend (no Rust commands for key CRUD)
## Key Design Decisions
### 1. Hybrid Rust+Svelte Architecture
Rust handles audio I/O, ASR inference, and network requests (performance-critical paths). Svelte handles UI, state, and orchestration. Communication via Tauri IPC (invoke + events).
### 2. Three ASR Engine Backends
Browser (Web Speech API), Vosk (on-device), Remote (OpenAI-compatible API). Engine selection persisted in settings. Switching restarts recognition pipeline.
### 3. Event-Driven Streaming
Rust ASR backends emit events for subtitle results. Frontend subscribes via `listen()`. Decouples audio processing from UI rendering.
### 4. Stronghold for Secrets
API keys stored in encrypted Stronghold vault (argon2 KDF). Password auto-generated, stored in tauri-plugin-store. Frontend accesses vault via JS API, not Rust commands.
### 5. Audio Pipeline
cpal captures at native rate → downmix → resample to 16kHz mono → deliver via mpsc channel. ASR engines consume from the channel. Single capture instance shared across engines.
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
