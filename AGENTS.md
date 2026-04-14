# AGENTS.md

Instructions for AI agents working on the Subtitle codebase.

## Quick Reference

| Layer | Location | Language | Key files |
|-------|----------|----------|-----------|
| Frontend | `src/` | TypeScript + Svelte 5 | `app.svelte`, `lib/speech.svelte.ts`, `lib/stronghold.ts` |
| Backend | `src-tauri/src/` | Rust | `lib.rs`, `commands.rs`, `audio.rs`, `vosk.rs`, `remote_asr.rs` |
| Config | `src-tauri/` | JSON/TOML | `tauri.conf.json`, `Cargo.toml`, `capabilities/default.json` |

## Architecture Summary

Hybrid Tauri v2 app. Rust handles audio capture, ASR inference, and network calls. Svelte 5 handles UI, state, and orchestration. Communication via `invoke()` (frontend→Rust) and Tauri events (Rust→frontend).

### Data Flow: Audio → Subtitle

```
Microphone → cpal (native rate) → downmix + resample (16kHz mono) → mpsc channel
                                                                         ↓
                                                           ASR engine consumes:
                                                           ├─ Vosk: on-device recognition loop
                                                           ├─ Remote: WAV encode → HTTP POST → parse response
                                                           └─ Browser: Web Speech API (separate path)
                                                                         ↓
                                                           Rust emits Tauri event
                                                                         ↓
                                                           Frontend listens → appends subtitle → triggers translation
```

### Three ASR Engines

- **browser** — Web Speech API in webview. No Rust involvement for recognition. Audio capture still via Rust.
- **vosk** — Rust-side Vosk recognizer. Events: `backend://subtitle/update` (partial), `backend://subtitle/final` (final).
- **remote** — Rust-side HTTP client. Buffers audio with silence detection (RMS < 0.005, 300ms window), encodes WAV, POSTs to OpenAI-compatible endpoint. Two-layer VAD: pre-send RMS gate (0.04) discards silent buffers; post-response logprob filter (-0.8) discards Whisper hallucinations. Handles DeepInfra `/v1/inference/` and standard `/v1/audio/transcriptions` URL patterns. Multipart form field: `audio`. Events: `subtitle` (result), `asr-error` (error with `retryable` flag).

### Settings & Secrets

- **Settings:** `tauri-plugin-store` → `settings.json`. Struct: `Settings` in `commands.rs`. Load: `settings_get`, save: `settings_set`.
- **API keys:** `tauri-plugin-stronghold` → encrypted vault. Frontend accesses via JS API (`src/lib/stronghold.ts`). Vault password auto-generated, stored in settings.

## Code Patterns

### Adding a New Rust Command

1. Write `#[tauri::command]` function in `commands.rs` (or relevant module)
2. Register in `lib.rs` → `invoke_handler(tauri::generate_handler![...])`
3. Add permission in `src-tauri/capabilities/default.json` if needed
4. Call from frontend: `invoke('command_name', { arg1, arg2 })`

### Adding a New Frontend Event Listener

1. Rust side: `app.emit("event-name", payload)` using `Emitter` trait
2. Frontend: `listen<PayloadType>('event-name', (event) => { ... })` from `@tauri-apps/api/event`
3. Clean up listener on stop (store `UnlistenFn`, call on teardown)

### Svelte 5 Component Pattern

```svelte
<script lang="ts">
  interface Props {
    propName: type;
    onAction: (value: type) => void;
  }
  let { propName, onAction }: Props = $props();
</script>
```

- No `export let`. Use `$props()` destructuring.
- No `createEventDispatcher`. Use callback props.
- Reactive state: `$state()`, computed: `$derived()`, effects: `$effect()`.

### Rust Shared State Pattern

State structs are registered in `lib.rs` via `app.manage(StateStruct::new())`. Commands access via `State<'_, StateStruct>`.

### Error Handling Convention

- Rust: `Result<T, String>` everywhere. Convert errors with `.map_err(|e| e.to_string())`.
- Frontend: Try/catch around `invoke()`. Set `speech.errorMessage` and `speech.status = 'error'`.

## Key Constraints

- **Audio format:** 16kHz mono f32. 480-sample (30ms) chunks via mpsc channel.
- **Remote ASR:** OpenAI Whisper-compatible API. Multipart form with WAV file. Max 3 retries with exponential backoff.
- **Vosk model path:** `{CARGO_MANIFEST_DIR}/vosk-model`. Must exist before `vosk_start`.
- **Stronghold vault:** Password auto-generated on first run. Vault path: `{CARGO_MANIFEST_DIR}/.local/share/com.subtitle.realtime/vault.hold`.
- **Max subtitles:** 12 lines (`MAX_SUBTITLES` in `speech.svelte.ts`).
- **Translation context:** Last 3 source lines for context window.

## Common Tasks

### Debug Audio Issues
Check `audio.rs` logs for device selection, sample rate, channel count. Verify cpal config: prefers 16kHz mono f32, falls back to native config with resampling.

### Debug Remote ASR Issues
Check endpoint URL in settings, API key in Stronghold, and `remote_asr.rs` logs. Verify endpoint accepts multipart WAV at `/v1/audio/transcriptions`.

### Add a New Language
Edit `src/lib/languages.ts`. Add to `SOURCE_LANGUAGES` (with locale code like `xx-XX`) and/or `TARGET_LANGUAGES` (with short code like `xx`).

### Modify Settings Schema
1. Update `Settings` struct in `commands.rs`
2. Update `settings_get` to read new field with default
3. Update `settings_set` match arm for type coercion
4. Update `Settings.svelte` Props and UI
5. Update `app.svelte` state and onMount loading

## Testing

- **Frontend:** Vitest configured. Tests in `src/lib/__tests__/`. Run: `pnpm t` or `pnpm vitest`.
- **Backend:** `cargo test --lib` from `src-tauri/`. Tests in each module's `#[cfg(test)] mod tests` block.
- **Pre-commit:** lefthook runs typecheck, biome (TS/JS/JSON), prettier (Svelte), rustfmt (Rust).
- **Pre-push:** lefthook runs `pnpm t`, `cargo test --lib`, `pnpm typecheck:native`, `pnpm audit --audit-level critical`.
- Install hooks: `pnpm lefthook install`. Run manually: `pnpm lefthook run pre-commit`.
