# Migration Plan: Preact → Svelte 5 + Tauri v2

## Goal
Migrate Real-time Subtitles app from **Preact + Vite** to **Svelte 5 + Tauri v2** with focus on:
- Simplicity & readability
- Svelte 5 Runes best practices
- Tauri v2 best practices
- **Remove Whisper AI entirely** — use only WebSpeech API

---

## Current Stack → New Stack

| Layer | Current | New |
|-------|---------|-----|
| UI Framework | Preact | Svelte 5 (Runes) |
| Desktop Shell | None (web only) | Tauri v2 |
| Build Tool | Vite | Vite (via Tauri) |
| Language | TypeScript | TypeScript |
| Styling | CSS Modules | Scoped `<style>` in `.svelte` |
| Speech Engine | WebSpeech + Whisper | WebSpeech only |
| Translation | DeepInfra API | DeepInfra API (unchanged) |
| State Management | useState/useRef hooks | Svelte 5 Runes ($state, $derived, $effect) |

---

## Key Decisions

### 1. Svelte Only (not SvelteKit)
This app is a single-page app with no routing. SvelteKit adds unnecessary complexity.
Using **plain Svelte 5 + Vite** via Tauri's built-in frontend setup.

### 2. Svelte 5 Runes
Use the new reactive primitives:
- `$state()` — reactive state (replaces useState)
- `$derived()` — computed values (replaces useMemo/computed)
- `$effect()` — side effects (replaces useEffect)
- `$bindable()` — two-way binding in components
- No more stores — runes are simpler and more readable

### 3. Scoped Styles in `.svelte`
Each component has its own `<style>` block. No CSS Modules needed.
Global styles in `src/app.css`.

### 4. Tauri v2 Benefits
- Desktop app with system tray support
- No browser needed
- Better microphone access (native permissions)
- Smaller bundle than Electron (~10MB vs ~100MB+)
- Rust backend for future features (file export, etc.)

### 5. Remove Whisper
Simplifies the app significantly:
- No Web Worker needed
- No model downloading/loading
- No audio resampling
- No progress bar UI
- Smaller bundle (~300MB less)
- App goes from `idle → loading → listening` to just `idle → listening`

---

## Migration Phases

See [02-project-structure.md](./02-project-structure.md) for new file layout.
See [03-implementation-details.md](./03-implementation-details.md) for each component's design.
See [04-tauri-setup.md](./04-tauri-setup.md) for Tauri-specific configuration.
