# Project Structure

```
subtitle/
├── src-tauri/                    # Tauri v2 Rust backend
│   ├── Cargo.toml                # Rust dependencies
│   ├── tauri.conf.json           # Tauri configuration
│   ├── capabilities/
│   │   └── default.json          # Permission capabilities
│   ├── icons/                    # App icons for each platform
│   └── src/
│       ├── main.rs               # Entry point
│       └── lib.rs                # Tauri commands (if any)
│
├── src/                          # Svelte frontend
│   ├── main.ts                   # Entry point — mounts App
│   ├── app.html                  # HTML template
│   ├── app.css                   # Global styles (reset, fonts, variables)
│   ├── app.svelte                # Root component — screen router
│   │
│   ├── lib/                      # Shared modules
│   │   ├── types.ts              # TypeScript interfaces
│   │   ├── languages.ts          # Language lists (source + target)
│   │   └── speech.svelte.ts      # WebSpeech API logic (as Svelte 5 module)
│   │
│   └── components/               # UI components
│       ├── IdleScreen.svelte     # Start screen
│       ├── ListeningScreen.svelte # Active subtitle display
│       ├── ErrorScreen.svelte    # Error display
│       ├── Settings.svelte       # Settings bottom sheet
│       ├── SubtitleLine.svelte   # Single subtitle line
│       └── Dropdown.svelte       # Reusable select dropdown
│
├── static/                       # Static assets
│   └── favicon.png
│
├── package.json
├── svelte.config.js
├── vite.config.ts
├── tsconfig.json
└── .env                          # VITE_DEEPINFRA_API_KEY
```

## Why This Structure

1. **`src/lib/speech.svelte.ts`** — Svelte 5 allows reactive state in `.svelte.ts` files using runes. This replaces the Preact hook with a cleaner module.

2. **No `src/routes/`** — We're not using SvelteKit, so no file-based routing. One page app.

3. **`src-tauri/`** — Standard Tauri v2 convention. Houses the Rust backend.

4. **Components are flat** — Only 6 components. No deep nesting. Easy to find.

5. **Translation stays inline** — The translation logic is simple enough to live in `app.svelte`'s `$effect()`. No separate module needed unless it grows.

## Component Tree

```
App (app.svelte)
├── IdleScreen        ← shows when status === 'idle'
├── ListeningScreen   ← shows when status === 'listening'
│   ├── StatusBar
│   └── SubtitleLine (× N)
├── ErrorScreen       ← shows when status === 'error'
└── Settings          ← overlay, always mounted
    └── Dropdown (× 3)
```

## Data Flow

```
App manages:
  - status: $state<'idle' | 'listening' | 'error'>('idle')
  - targetLang: $state('th')
  - subtitles: $state<SubtitleLine[]>([])
  - translations: $state<Record<string, string>>({})
  - settingsOpen: $state(false)
  - subtitlePosition: $state(20)

Speech module provides:
  - start(), stop(), setLanguage()
  - feeds subtitles back to App via callback

$effect on subtitles:
  → triggers translation for new final lines
  → deduplicated via translatedRef (Set)
```
