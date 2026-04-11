# Phase 3: Vosk ASR Engine - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-10
**Phase:** 03-vosk-asr-engine
**Areas discussed:** Model loading strategy, Model path configuration, Recognition configuration, Audio channel integration, Engine state preservation

---

## Model Loading Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Lazy on first captioning (Recommended) | Load when user starts captioning with Vosk selected. Show spinner during load. Simpler, saves RAM when not using Vosk. | ✓ |
| Background preload after app starts | Load in background thread while app is already shown. App instantly interactive, Vosk ready by the time user selects it. | |
| Eager on startup | Load before app window opens. Slowest startup, but all engines ready immediately. | |

**User's choice:** Lazy on first captioning (Recommended)

---

## Model Path Configuration

| Option | Description | Selected |
|--------|-------------|----------|
| Bundled small model (Recommended) | Ship a minimal English Vosk model (~50MB) bundled with the app. Works out of the box. User can configure a custom path in settings later. | ✓ |
| Settings-configured path only | No bundled model. User must set model path in settings before using Vosk. Phase 3 won't work until Phase 4 (settings) is done. | |
| First-run download prompt | Prompt user to download model from Vosk website when they first select Vosk. Adds friction but keeps app bundle small. | |

**User's choice:** Bundled small model (Recommended)

---

## Recognition Configuration

| Option | Description | Selected |
|--------|-------------|----------|
| Continuous partial → final (Recommended) | Show interim results as user speaks, replace with final. Best for live subtitle overlay. Frontend already handles this pattern. | ✓ |
| Final results only | Only show text after a complete utterance is recognized. Lower latency, less distracting, but no live feedback while speaking. | |

**User's choice:** Continuous partial → final (Recommended)

---

## Audio Channel Integration

| Option | Description | Selected |
|--------|-------------|----------|
| Direct mpsc consumer task (Recommended) | vosk.rs spawns a background task reading from mpsc channel and feeding Vosk recognizer. Cleanest — Vosk owns its consumption loop, Phase 2 just produces chunks. | ✓ |
| Ring buffer with controlled read | Ring buffer shared between capture and ASR threads. Vosk reads at its own pace. More control over timing but added complexity. | |

**User's choice:** Direct mpsc consumer task (Recommended)

---

## Engine State Preservation

| Option | Description | Selected |
|--------|-------------|----------|
| Clear and start fresh (Recommended) | Engine switch starts a new session, subtitle buffer cleared. Consistent with current setLanguage() behavior in speech.svelte.ts. | ✓ |
| Keep last 3 subtitles | Preserve brief context — last 3 subtitles remain visible during transition. Helps user orient between engine switches. | |
| Preserve all history | Full subtitle buffer preserved across engine switches. Maximum continuity but may confuse which engine produced what. | |

**User's choice:** Clear and start fresh (Recommended)

---

## Claude's Discretion

No areas deferred to Claude — all decisions made by user.

## Deferred Ideas

None — discussion stayed within phase scope.
