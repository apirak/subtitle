# Architecture & Behavior Documentation

> Documented on 2026-04-10 — before migration to Svelte + Turi
> Purpose: Reference for validating that the new implementation matches all current behavior

---

## 1. Tech Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| UI Framework | Preact | ^10.29.1 |
| Build Tool | Vite | ^8.0.4 |
| Language | TypeScript | ~6.0.2 |
| Styling | CSS Modules | (built-in via Vite) |
| ML/AI | @huggingface/transformers | ^4.0.1 |
| Translation API | DeepInfra (Qwen3-14B) | External |

---

## 2. File Structure

```
src/
├── main.tsx              # Entry point — renders <App /> into #app
├── app.tsx               # Main component — all state & screen rendering
├── app.module.css        # Main app styles (dark theme, animations)
├── types.ts              # Shared TypeScript interfaces & types
├── Dropdown.tsx          # Reusable <select> dropdown component
├── Dropdown.module.css   # Dropdown styles (custom arrow, dark theme)
├── Settings.tsx          # Settings modal (bottom sheet)
├── Settings.module.css   # Settings styles (glassmorphism, slide-up)
├── useSpeechToText.ts    # Hook: Whisper AI speech recognition
├── useWebSpeechApi.ts    # Hook: Browser Web Speech API
├── useTranslation.ts     # Hook: DeepInfra translation API
├── worker.ts             # Web Worker: Whisper model loading & inference
├── index.css             # Global reset (box-sizing, black bg, font)
├── css-modules.d.ts      # TypeScript declaration for .module.css
└── app.css               # Unused (Preact template default)
public/
├── favicon.svg
├── icons.svg
└── models/
    └── onnx-community/whisper-base/
        ├── config.json
        ├── generation_config.json
        ├── preprocessor_config.json
        ├── tokenizer.json
        ├── tokenizer_config.json
        └── onnx/
            ├── encoder_model.onnx (~78 MB)
            └── decoder_model_merged.onnx (~198 MB)
scripts/
└── download-model.mjs     # Script: downloads Whisper ONNX model
```

---

## 3. Core Types (`types.ts`)

```typescript
AppStatus = 'idle' | 'loading' | 'listening' | 'processing' | 'error'
EngineType = 'whisper' | 'webspeech'

SubtitleLine = {
  id: string           // unique per line (timestamp-random or "interim-...")
  text: string
  timestamp: number
  translation?: string // populated after translation completes
}

// Worker communication types
MainToWorkerMessage = 'init' | 'transcribe' (with audio Float32Array + language)
WorkerToMainMessage = 'loading' | 'ready' | 'processing' | 'result' | 'error'
```

---

## 4. Application State

All state lives in `App` component (no global store):

| State | Type | Default | Purpose |
|-------|------|---------|---------|
| `engine` | `'webspeech' \| 'whisper'` | `'webspeech'` | Active speech recognition engine |
| `targetLang` | `string` | `'th'` | Translation target language code |
| `translations` | `Record<string, string>` | `{}` | Cache: subtitle ID → translated text |
| `settingsOpen` | `boolean` | `false` | Settings panel visibility |
| `subtitlePosition` | `number` | `20` | Subtitle vertical position (5-90% from bottom) |
| `translatedRef` | `Set<string>` | `new Set()` | Tracks which subtitle IDs already sent for translation |

Derived state:
```typescript
current = engine === 'whisper' ? whisper : webspeech
// destructured: status, subtitles, loadProgress, errorMessage, language, setLanguage, startSession, stopSession
```

---

## 5. Screen Rendering Logic

The app renders exactly one screen based on `status`:

### 5.1 Idle Screen (`status === 'idle'`)
- Title: "Real-time Subtitles"
- Subtitle config: `{sourceLabel} → {targetLabel}`
- Start button (mic icon + "Start")
- Settings gear button (bottom right)

### 5.2 Loading Screen (`status === 'loading'`)
- Text: "Loading AI Model..."
- Progress bar (width based on `loadProgress%`)
- Percentage text

### 5.3 Listening/Processing Screen (`status === 'listening' || 'processing'`)
- **Status bar** (fixed top):
  - Green pulsing dot
  - Status text: "Listening..." or "Processing..."
  - Language badge: `{sourceLabel} → {targetLabel}`
  - Stop button (right side)
- **Subtitle container** (fixed, positioned at `bottom: {subtitlePosition}%`):
  - Max 12 subtitle lines displayed
  - Last line: 3rem, bold, white
  - Older lines: 1.8rem, 25% opacity
  - Each line can show:
    - Original text
    - "···" bouncing dots (while translating)
    - Translated text (below, 60% size, 60% opacity)

### 5.4 Error Screen (`status === 'error'`)
- Error message in red
- Error detail in muted text
- "Try Again" button

### 5.5 Settings Panel (always mounted, shown/hidden)
- Bottom sheet with backdrop blur
- Fields:
  1. **Speech Engine** dropdown (Built-in / Whisper AI)
  2. **Source Language** dropdown (varies by engine)
  3. **Translate To** dropdown (14 target languages)
  4. **Subtitle Position** slider (5-90%)

---

## 6. Data Flow

```
User clicks Start
        │
        ▼
┌─────────────────── Check Engine ────────────────────┐
│                                                       │
│  WebSpeech                    Whisper                 │
│  ─────────                    ─────────               │
│  Browser SpeechRecognition    Load ONNX model         │
│  .continuous = true           via Web Worker          │
│  .interimResults = true       Show progress bar       │
│  Instant start                Then start listening    │
│                                                       │
└───────────────────────────────────────────────────────┘
        │
        ▼
   Audio Capture (microphone)
        │
        ▼
┌─────────────────── Speech Recognition ───────────────┐
│                                                       │
│  WebSpeech                    Whisper                 │
│  ─────────                    ─────────               │
│  onresult events              Buffer 3s chunks        │
│  - interim: show as draft     Resample to 16kHz       │
│  - final: commit as subtitle  Send to Worker          │
│                               Worker runs inference   │
│                               Returns text result     │
│                                                       │
└───────────────────────────────────────────────────────┘
        │
        ▼
   Subtitle Added (status → 'listening')
        │
        ▼
   Translation Trigger (useEffect on subtitles)
        │
        ├─ Skip if id starts with "interim-"
        ├─ Skip if already in translatedRef
        ├─ Skip if no targetLang
        │
        ▼
   Call useTranslation.translate()
        │
        ├─ Build prompt with context (previous 2 lines)
        ├─ POST to DeepInfra API (Qwen3-14B)
        ├─ Parse response, strip <think/> tags
        └─ Update translations[id] = result
        │
        ▼
   UI re-renders with translation shown
```

---

## 7. Hook Details

### 7.1 `useSpeechToText()` — Whisper AI Engine

**Returns:** `{ status, subtitles, loadProgress, errorMessage, language, setLanguage, detectedLanguage, startSession, stopSession }`

**Key behaviors:**
- Creates a **Web Worker** (`worker.ts`) for model loading and inference
- Uses `ScriptProcessorNode` (deprecated but functional) for audio capture
- Resamples audio from native sample rate to 16kHz using linear interpolation
- Buffers audio chunks, flushes every **3 seconds**
- Model loaded once, **reused across sessions** (worker kept alive)
- Max **12 subtitles** kept in state
- Auto-detect language if `language === null`
- Worker terminated only on component unmount

**Language options (14 + auto):** Auto-detect, English, Thai, Chinese, Japanese, Korean, Spanish, French, German, Portuguese, Russian, Arabic, Hindi, Vietnamese, Indonesian

### 7.2 `useWebSpeechApi()` — Browser Built-in Engine

**Returns:** Same interface as `useSpeechToText()`

**Key behaviors:**
- Creates `SpeechRecognition` / `webkitSpeechRecognition` instance
- `continuous = true`, `interimResults = true`
- Interim results shown with `interim-` prefixed IDs (replaced on final)
- **Auto-restarts** on `onend` if user didn't manually stop and no fatal error
- Ignores `aborted` and `no-speech` errors
- Friendly error messages for: network, not-allowed, audio-capture, service errors
- Changing language **restarts** the recognition session (100ms delay)

**Language options (14):** Uses BCP 47 codes (th-TH, en-US, zh-CN, etc.)

### 7.3 `useTranslation(targetLang)` — DeepInfra Translation

**Returns:** `{ translate }`

**Key behaviors:**
- API: `https://api.deepinfra.com/v1/openai/chat/completions`
- Model: `Qwen/Qwen3-14B`
- API key from `import.meta.env.VITE_DEEPINFRA_API_KEY`
- Tracks in-flight requests via `Set<string>` — **no duplicate requests**
- Keeps **recent 3 lines** for context (last 2 used as context in prompt)
- Prompt uses `/no_think` prefix to disable model reasoning
- Strips `<think/>...</think()` tags from response
- Silent failure on API error (logs to console only)

**Prompt structure (with context):**
```
/no_think
Translate the last sentence to {Language}. The previous sentences are for context only — do not translate them.

Context:
1. {previous line 1}
2. {previous line 2}

Translate this:
{current text}

Return only the translation.
```

**Target language options (14):** en, th, zh, ja, ko, es, fr, de, pt, ru, ar, hi, vi, id

---

## 8. Worker Details (`worker.ts`)

**Lifecycle:**
1. Main thread sends `{ type: 'init' }` → Worker loads Whisper pipeline
2. Reports loading progress back to main thread
3. Sends `{ status: 'ready' }` when loaded
4. Main thread sends `{ type: 'transcribe', audio, language }` → Worker runs inference
5. Sends `{ status: 'result', text, language }` back

**Configuration:**
- Model: `onnx-community/whisper-base` (loaded from `/public/models/`)
- Local-only: `allowRemoteModels = false`
- Data type: `fp32`
- Singleton pattern: model loaded once, reused

---

## 9. Translation Deduplication

Two-layer protection against duplicate translations:

1. **`translatedRef` (Set)** in `App` — prevents re-sending a subtitle ID
2. **`inFlightRef` (Set)** in `useTranslation` — prevents concurrent requests for same ID

Flow:
```
subtitle added → check translatedRef → if not present, add to set & call translate()
translate() → check inFlightRef → if not present, add to set & make API call → remove from set on completion
```

---

## 10. Styling & UI Behavior

### Design System
- **Background:** Pure black (#000)
- **Text:** White with varying opacity (0.2 - 1.0)
- **Glass effect:** `backdrop-filter: blur()` on buttons and settings
- **Font:** System font stack (-apple-system preferred)
- **Border radius:** Rounded buttons (60px), dropdowns (12px), settings panel (20px top)

### Animations
| Name | Duration | Used By |
|------|----------|---------|
| `slideUp` | 0.4s ease-out | Subtitle line appearance |
| `pulse` | 1.5s ease-in-out | Loading text opacity |
| `pulseDot` | 2s ease-in-out | Green status dot scale/opacity |
| `dotBounce` | 1.2s ease-in-out | Translation "···" indicator |

### Responsive Behavior
- Full viewport height (`100vh`)
- Subtitle max-width: `min(90%, 42rem)`
- Settings panel max-width: `420px`
- Subtitle position adjustable: 5-90% from bottom

---

## 11. Environment Variables

| Variable | Required | Purpose |
|----------|----------|---------|
| `VITE_DEEPINFRA_API_KEY` | Yes | DeepInfra API authentication for translation |

---

## 12. Build & Scripts

| Script | Command | Purpose |
|--------|---------|---------|
| `dev` | `vite` | Start dev server |
| `build` | `tsc -b && vite build` | Type check + production build |
| `typecheck` | `tsc -b` | Type check only |
| `model` | `node scripts/download-model.mjs` | Download Whisper ONNX model |
| `preview` | `vite preview` | Preview production build |

---

## 13. Migration Checklist

When migrating to Svelte + Turi, ensure these behaviors are preserved:

- [ ] Two speech engines (WebSpeech API + Whisper via Web Worker)
- [ ] Whisper model loaded from local `/public/models/` with progress tracking
- [ ] Audio resampling to 16kHz for Whisper
- [ ] 3-second chunked audio processing
- [ ] Interim + final subtitle distinction
- [ ] Max 12 subtitles in view
- [ ] Auto-restart Web Speech on disconnection
- [ ] Translation via DeepInfra Qwen3-14B with context
- [ ] Translation deduplication (double-layer)
- [ ] `/no_think` prompt prefix + `<think/>` tag stripping
- [ ] Translation "···" indicator while in progress
- [ ] Adjustable subtitle position (5-90%)
- [ ] Settings bottom sheet with glassmorphism
- [ ] Dark theme with pure black background
- [ ] 4 animations: slideUp, pulse, pulseDot, dotBounce
- [ ] Graceful error handling with friendly messages
- [ ] Language-specific error messages for WebSpeech errors
- [ ] Worker kept alive across sessions (only terminated on unmount)
- [ ] `VITE_DEEPINFRA_API_KEY` environment variable
