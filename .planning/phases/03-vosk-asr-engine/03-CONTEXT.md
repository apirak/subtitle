# Phase 3: Vosk ASR Engine - Context

**Gathered:** 2026-04-10
**Status:** Ready for planning

<domain>
## Phase Boundary

Integrate Vosk on-device speech recognition into the Rust backend. After this phase, the app transcribes microphone audio to text using a Vosk model running locally, with live partial results and engine switching support.
</domain>

<decisions>
## Implementation Decisions

### Model Loading
- **D-01:** Lazy loading — Vosk model loads in a background thread when user starts captioning with Vosk selected. UI shows a loading indicator during model load. Model is unloaded when captioning stops.

### Model Source
- **D-02:** Bundled small model — Ship a minimal English Vosk model (~50MB) with the app. Works out of the box without user configuration. User can configure a custom model path later via settings (Phase 4, SETT-05).

### Partial Results
- **D-03:** Continuous partial → final results. Interim results shown as user speaks (via `backend://subtitle/update`), replaced by final results (`backend://subtitle/final`) when utterance is complete. Best for live subtitle overlay.

### Audio Integration
- **D-04:** Direct mpsc consumer task — The `vosk.rs` module spawns a background task that continuously reads 30ms audio chunks from Phase 2's mpsc channel and feeds them to the Vosk recognizer. Clean architecture: Phase 2 produces, Vosk consumes.

### Engine Switching
- **D-05:** Clear on engine switch — When switching ASR engines, the subtitle buffer is cleared and a new session starts. Consistent with current `setLanguage()` behavior in `speech.svelte.ts`.

### Integration Architecture
- **D-06:** Vosk recognizer runs in a dedicated Rust module (`src-tauri/src/vosk.rs`). It subscribes to Phase 2's mpsc channel, processes audio chunks, and emits events via `emit_subtitle_update` / `emit_subtitle_final`. No audio round-trips through the frontend.

### Background Thread Requirement
- **D-07:** All Vosk model operations (load, inference, unload) run on background threads via `tokio::spawn`. The UI thread must never block on Vosk. Per PITFALL #5: "Vosk model loading blocks the calling thread -- must use background thread."

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Context
- `.planning/PROJECT.md` — Core value, architecture shift rationale, constraints
- `.planning/REQUIREMENTS.md` §ASR-01, ASR-04, ASR-05 — Vosk-specific requirements
- `.planning/ROADMAP.md` §Phase 3 — Goal and success criteria for this phase
- `.planning/ROADMAP.md` §Phase 2 — Phase 2 delivers audio via mpsc channel

### Codebase Conventions
- `.planning/codebase/ARCHITECTURE.md` — Current state before Phase 3
- `.planning/codebase/CONVENTIONS.md` — Rust naming (snake_case), error handling, logging
- `.planning/codebase/STACK.md` — Rust edition 2021, minimum rust-version 1.77.2

### Prior Phase Context
- `.planning/phases/01-rust-backend-infrastructure/01-CONTEXT.md` — Phase 1 IPC patterns, event streaming
- `.planning/phases/02-microphone-audio-capture/02-CONTEXT.md` — Phase 2 audio capture decisions: cpal, 30ms chunks, mpsc channel

### External
- Vosk crate documentation — https://github.com/alphacep/vosk-api
- Vosk models — https://alphacephei.com/vosk/models
</canonical_refs>

<codebase>
## Existing Code Insights

### Reusable Assets
- `emit_subtitle_update`, `emit_subtitle_final`, `emit_error` helpers in `commands.rs` — reuse for emitting Vosk results
- Event protocol: `backend://subtitle/{update,final,error}` — already defined in Phase 1
- `tauri-plugin-log` for debug logging during ASR development

### Established Patterns
- Phase 1: `invoke()` from frontend to Rust, `listen()` for Tauri events from Rust to frontend
- Phase 2: cpal for audio capture, 30ms chunks (~480 samples at 16kHz), mpsc channel to ASR
- Tokio async runtime (standard with Tauri v2) — all background work via `tokio::spawn`

### Integration Points
- Vosk module consumes from Phase 2's mpsc channel (already established in Phase 2 D-03)
- Vosk emits subtitle events via existing `emit_subtitle_update` / `emit_subtitle_final` helpers
- Settings engine selection (`asr_engine: "vosk"`) triggers Vosk module activation
- Model path from settings (Phase 4 SETT-05) — until then, use bundled default model

</codebase>

<specifics>
## Specific Ideas

### Vosk Model for Phase 3
- Use `vosk-model-small-en-us` (≈50MB) as the bundled default model for Phase 3
- Lightweight enough to ship with the app, fast to load, good enough for live captioning
- Place model files in `src-tauri/vosk-model/` directory for Phase 3

### Vosk Crate
- Use the `vosk` Rust crate for model loading and inference
- Verify the crate compiles on Linux (primary platform) before committing to it

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within Phase 3 scope.

</deferred>

---

*Phase: 03-vosk-asr-engine*
*Context gathered: 2026-04-10*
