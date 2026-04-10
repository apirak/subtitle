# Implementation Details

## 1. Types (`src/lib/types.ts`)

Keep it minimal — only what's needed without Whisper:

```typescript
export type AppStatus = 'idle' | 'listening' | 'error';

export interface SubtitleLine {
  id: string;
  text: string;
  timestamp: number;
}
```

**Removed from old types:**
- `MainToWorkerMessage` / `WorkerToMainMessage` — no Worker
- `EngineType` — only WebSpeech
- `'loading'` / `'processing'` statuses — no model loading
- `SpeechRecognitionInstance` and related — use native browser types

---

## 2. Speech Module (`src/lib/speech.svelte.ts`)

This is the core logic. Uses Svelte 5 runes for reactive state.

```typescript
// Conceptual structure — not full implementation

const MAX_SUBTITLES = 12;

class SpeechRecognition {
  status = $state<AppStatus>('idle');
  subtitles = $state<SubtitleLine[]>([]);
  language = $state('en-US');
  errorMessage = $state('');

  private recognition: SpeechRecognition | null = null;
  private stopping = false;

  start() { /* create recognition, set handlers, start */ }
  stop() { /* set stopping=true, stop recognition */ }
  setLanguage(lang: string) { /* restart with new language */ }
}

export const speech = new SpeechRecognition();
```

**Key behaviors to preserve from current `useWebSpeechApi.ts`:**
- `continuous = true`, `interimResults = true`
- Interim results with `interim-` prefix IDs
- Auto-restart on `onend` unless manually stopped or errored
- Ignore `aborted` and `no-speech` errors
- Friendly error messages for network, not-allowed, audio-capture, service
- Language change restarts session (100ms delay)
- Max 12 subtitles

**Svelte 5 advantage:** No more `useRef` for mutable refs. Just use class properties.

---

## 3. App Component (`src/app.svelte`)

Root component — owns the state that ties everything together.

```svelte
<script lang="ts">
  import { speech } from './lib/speech.svelte';
  // ... other imports

  let targetLang = $state('th');
  let translations = $state<Record<string, string>>({});
  let settingsOpen = $state(false);
  let subtitlePosition = $state(20);

  const translatedIds = new Set<string>();

  // Derived labels
  let sourceLabel = $derived(
    LANGUAGES.find(l => l.code === speech.language)?.label ?? 'Auto'
  );
  let targetLabel = $derived(
    TARGET_LANGUAGES.find(l => l.value === targetLang)?.label ?? 'English'
  );

  // Translation trigger
  $effect(() => {
    for (const line of speech.subtitles) {
      if (line.id.startsWith('interim-')) continue;
      if (translatedIds.has(line.id)) continue;
      translatedIds.add(line.id);
      if (targetLang) translate(line.id, line.text);
    }
  });
</script>
```

**Simplified from current `app.tsx`:**
- No `engine` state — WebSpeech only
- No `loadProgress` — no model loading
- No `useCallback` — Svelte reactivity handles this
- `translatedRef` is a plain `Set` (not a ref)

---

## 4. Components

### IdleScreen.svelte
- Title "Real-time Subtitles"
- Language config display (`{sourceLabel} → {targetLabel}`)
- Start button (mic icon)
- Settings gear button

### ListeningScreen.svelte
Props: `subtitles`, `translations`, `sourceLabel`, `targetLabel`, `subtitlePosition`, `onStop`

- Fixed status bar at top
- Subtitle container at configurable bottom position
- Maps subtitles → SubtitleLine components

### SubtitleLine.svelte
Props: `line`, `translation`, `isTranslating`

- Shows text
- Shows "···" bouncing dots while translating
- Shows translation text when ready
- CSS: last line = 3rem bold white, older = 1.8rem 25% opacity

### ErrorScreen.svelte
Props: `message`, `onRetry`

- Error message in red
- Try Again button

### Settings.svelte
Props: `open`, `onClose`, `language`, `onLanguageChange`, `targetLang`, `onTargetLangChange`, `subtitlePosition`, `onSubtitlePositionChange`

- Bottom sheet with backdrop blur
- Dropdowns: Source Language, Translate To
- Slider: Subtitle Position
- **Removed:** Engine dropdown (no more Whisper choice)

### Dropdown.svelte
Props: `value`, `options`, `onChange`

- Reusable `<select>` with custom styling

---

## 5. Translation Logic

Stays in `app.svelte` as a function (not a separate module):

```typescript
async function translate(id: string, text: string) {
  // Same logic as current useTranslation.ts
  // - Build prompt with context (previous 2 lines)
  // - POST to DeepInfra API
  // - Strip <think/> tags
  // - Update translations[id]
}
```

**Simplified:** No hook wrapper needed. Just a function with closure over `targetLang` and `translations`.

---

## 6. Global Styles (`src/app.css`)

```css
:root {
  --bg: #000;
  --text: white;
  --font: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  --glass-bg: rgba(255, 255, 255, 0.08);
  --glass-border: rgba(255, 255, 255, 0.12);
}

*, *::before, *::after {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

body {
  background: var(--bg);
  color: var(--text);
  font-family: var(--font);
  -webkit-font-smoothing: antialiased;
}
```

---

## 7. Preserved Behaviors Checklist

- [x] WebSpeech API with continuous + interimResults
- [x] Interim + final subtitle distinction
- [x] Max 12 subtitles in view
- [x] Auto-restart Web Speech on disconnection
- [x] Translation via DeepInfra Qwen3-14B with context
- [x] Translation deduplication
- [x] `/no_think` prompt prefix + `<think/>` tag stripping
- [x] Translation "···" indicator while in progress
- [x] Adjustable subtitle position (5-90%)
- [x] Settings bottom sheet with glassmorphism
- [x] Dark theme with pure black background
- [x] 4 animations: slideUp, pulse, pulseDot, dotBounce
- [x] Graceful error handling with friendly messages
- [x] `VITE_DEEPINFRA_API_KEY` environment variable

## Removed
- [x] Whisper engine (worker, model loading, audio processing)
- [x] Engine selection in settings
- [x] Loading/processing screens
- [x] Model download script
- [x] `/public/models/` directory (~276MB)
- [x] `@huggingface/transformers` dependency
