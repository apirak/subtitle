# Plan: Real-time Subtitle Web App (MVP)

## TL;DR
Build a browser-based real-time subtitle app using the existing Vite+Preact+TypeScript scaffold. Whisper-tiny runs entirely client-side via Transformers.js v4 in a Web Worker. Microphone audio is captured with Web Audio API, chunked every ~3s, and sent to the worker for transcription. Results appear as animated subtitles on a dark full-screen UI.

## Architecture Overview

```
User clicks "Start Session"
  → useSpeechToText hook requests mic permission
  → Hook creates Web Worker (worker.ts)
  → Worker loads Xenova/whisper-tiny model (progress sent to UI)
  → Hook starts AudioContext, captures PCM via ScriptProcessorNode
  → Every ~3s of audio → Float32Array chunk sent to Worker via postMessage
  → Worker runs ASR pipeline → sends { text } back
  → Hook updates subtitles state → App.tsx renders with CSS animation
```

## Steps

### Phase 1: Project Setup
1. Install `@huggingface/transformers` dependency (`pnpm add @huggingface/transformers`)
2. Create shared types file `src/types.ts` — defines `WorkerMessage` (main→worker) and `WorkerResponse` (worker→main) interfaces
3. Update `vite.config.ts` if needed for worker bundling (Vite handles `new Worker(new URL(...), { type: 'module' })` natively — no config change needed)

### Phase 2: Web Worker (`src/worker.ts`)
4. Create `src/worker.ts` with singleton `WhisperPipeline` class (pattern from Transformers.js docs):
   - `getInstance(progressCallback)` — lazy-loads `pipeline('automatic-speech-recognition', 'Xenova/whisper-tiny')` once
   - Posts `{ status: 'loading', progress }` messages during model download
   - Posts `{ status: 'ready' }` when pipeline is ready
5. Add `message` event listener that:
   - Receives `{ type: 'init' }` → triggers model warm-up
   - Receives `{ type: 'transcribe', audio: Float32Array }` → runs `transcriber(audio)` → posts `{ status: 'result', text }` back
   - Posts `{ status: 'processing' }` before inference starts

### Phase 3: Custom Hook (`src/useSpeechToText.ts`)
6. Create `useSpeechToText` hook that manages:
   - **Worker lifecycle**: creates Worker via `new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' })`, attaches message handler, cleanup on unmount
   - **App state**: `status` field tracks `'idle' | 'loading' | 'listening' | 'processing' | 'error'`
   - **Subtitles array**: `{ id: string, text: string, timestamp: number }[]`
7. Implement `startSession()`:
   - Posts `{ type: 'init' }` to worker (triggers model loading)
   - Calls `navigator.mediaDevices.getUserMedia({ audio: true })`
   - Creates `AudioContext` (sample rate 16000 via constructor option, or resample if browser doesn't support)
   - Connects mic stream → `ScriptProcessorNode` (bufferSize 4096)
   - Accumulates PCM samples in a buffer; every ~3 seconds, clones buffer as Float32Array and posts `{ type: 'transcribe', audio }` to worker
   - Resampling logic: if AudioContext sample rate ≠ 16000, downsample via linear interpolation
8. Implement `stopSession()`:
   - Disconnects audio nodes, stops mic tracks, terminates worker
   - Resets status to `'idle'`

### Phase 4: UI (`src/app.tsx` + `src/app.module.css`)
9. Replace `src/app.tsx` with new subtitle UI:
   - **Idle state**: centered "Start Session" button with mic icon (Unicode/SVG), minimal text
   - **Loading state**: pulsing "Loading AI Model..." with progress percentage
   - **Listening state**: subtitle container at bottom-center of screen, status indicator "● Listening", "Stop" button
   - **Subtitle rendering**: map over subtitles array, newest at bottom, container scrolls/pushes up smoothly
   - Each subtitle line fades in + slides up via CSS animation
10. Replace `src/app.css` with `src/app.module.css`:
    - Dark theme: `background: #000`, white text
    - Full viewport height layout
    - `.subtitle` class with `@keyframes slideUp` (translateY + opacity)
    - `.subtitleContainer` fixed at bottom ~30% of screen
    - Status indicator with pulsing dot animation
    - Button styling consistent with dark theme
11. Update `src/index.css`: set `body { margin: 0; background: #000; color: #fff; font-family: system-ui }`, remove existing variable-based theming

### Phase 5: Cleanup
12. Remove unused assets (`src/assets/preact.svg`, `src/assets/vite.svg`, `src/assets/hero.png`) — *ask user before deleting*
13. Update `index.html` title to "Real-time Subtitles"

## Relevant Files

| File | Action | Purpose |
|---|---|---|
| `src/types.ts` | **NEW** | Shared TS interfaces for worker messages |
| `src/worker.ts` | **NEW** | Transformers.js pipeline in Web Worker |
| `src/useSpeechToText.ts` | **NEW** | Custom hook: mic capture + worker comms |
| `src/app.tsx` | **REPLACE** | Subtitle UI (replaces template counter) |
| `src/app.module.css` | **NEW** | CSS module with dark theme + animations |
| `src/app.css` | **DELETE** | Replaced by app.module.css |
| `src/index.css` | **MODIFY** | Minimal dark base styles |
| `index.html` | **MODIFY** | Update title |
| `package.json` | **MODIFY** | Add `@huggingface/transformers` |

## Key Technical Details

### Worker Message Protocol (types.ts)
```
MainToWorker:
  | { type: 'init' }
  | { type: 'transcribe', audio: Float32Array }

WorkerToMain:
  | { status: 'loading', progress: number }
  | { status: 'ready' }
  | { status: 'processing' }
  | { status: 'result', text: string }
  | { status: 'error', message: string }
```

### Audio Pipeline
- `getUserMedia({ audio: true })` → `AudioContext` → `createMediaStreamSource()` → `ScriptProcessorNode(4096, 1, 1)`
- Buffer accumulation: collect samples in array, flush to worker every ~3 seconds (48000 samples at 16kHz)
- Resampling: if native rate ≠ 16kHz, use linear interpolation to downsample
- ScriptProcessorNode is deprecated but universally supported and sufficient for MVP. AudioWorklet would be the upgrade path.

### Whisper Pipeline
- Model: `Xenova/whisper-tiny` (~39MB ONNX, first load downloads from HuggingFace CDN, cached in browser)
- Input: `Float32Array` at 16kHz
- Output: `{ text: string }`
- Package: `@huggingface/transformers` v4.0.1

### Animation Approach
- Each subtitle gets a unique `id` (timestamp-based)
- New subtitles enter with `animation: slideUp 0.4s ease-out`
- Container uses `display: flex; flex-direction: column; justify-content: flex-end` so items stack from bottom
- Older subtitles naturally push up as new ones appear
- Max ~8 visible subtitles, older ones fade out via overflow hidden

## Verification
1. `pnpm install` succeeds without errors
2. `pnpm run dev` starts without build errors
3. Open in Chrome → click "Start Session" → grant mic permission → model loads with progress indicator
4. Speak into mic → subtitle text appears within ~3-5 seconds
5. Subtitles animate in smoothly, push older ones up
6. Click "Stop" → mic stops, UI returns to idle state
7. TypeScript compilation: `pnpm run build` succeeds with no type errors

## Decisions
- **ScriptProcessorNode over AudioWorklet**: simpler for MVP, widely supported, AudioWorklet would add extra file + complexity
- **Xenova/whisper-tiny**: smallest model for fastest loading + inference in browser. Trade-off: lower accuracy vs larger models
- **Fixed 3-second chunks**: simple timer-based chunking vs. silence detection (VAD). Keeps MVP simple. Silence detection could be a future enhancement
- **CSS Modules**: chosen over Tailwind since no Tailwind is configured in the project
- **useState over Signals**: simpler, no additional imports needed for a small state surface

## Further Considerations
1. **Language support**: Currently defaults to English. Could add a language selector prop for multilingual whisper models. Recommend: keep English-only for MVP.
2. **VAD (Voice Activity Detection)**: A simple energy-based silence detection could skip sending silent chunks to the worker, saving compute. Recommend: defer to post-MVP.
3. **Model caching indicator**: After first load, model is cached in browser. Could show "Cached ✓" on subsequent visits. Recommend: defer to post-MVP.
