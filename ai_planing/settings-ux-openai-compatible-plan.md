# Settings UX Review + Implementation Plan

## Objective

Design a centered modal Settings experience (macOS-like) with a left navigation menu and right-side configuration panel, while enabling user-defined OpenAI-compatible providers for both:

- Speech to Text (STT)
- Translation

This plan focuses on UX quality, information architecture, and how each setting is persisted and used in runtime.

## Current State Summary

- Frontend settings UI is a bottom sheet style panel in [src/components/Settings.svelte](src/components/Settings.svelte).
- Current app state wiring is in [src/app.svelte](src/app.svelte).
- STT supports browser, vosk, remote in [src/lib/speech.svelte.ts](src/lib/speech.svelte.ts).
- Remote STT endpoint assembly and request path handling live in [src-tauri/src/remote_asr.rs](src-tauri/src/remote_asr.rs).
- Settings schema and persistence are in [src-tauri/src/commands.rs](src-tauri/src/commands.rs).
- Translation backend is currently a stub in [src-tauri/src/commands.rs](src-tauri/src/commands.rs) (translate command).

## UX Review of Proposed Menu

The menu structure is strong and understandable. It aligns with common desktop mental models.

### Strengths

- Left nav + right detail panel is scalable for future options.
- Grouping by Theme / Language / AI is meaningful.
- Separate STT and Translate sections is correct, because provider/model/credentials may differ.

### UX Risks to Fix Early

- Too many model names in one dropdown can confuse users if they don\'t know provider compatibility.
- Reusing one URL/API key field for both STT and Translate causes configuration mistakes.
- "Online/Offline" and "Model" can conflict unless model options filter based on mode.
- Two translation outputs (First/Second Translate) need explicit display behavior in Listening UI.

### Recommended UX Details

1. Navigation behavior

- Keep selected menu sticky while scrolling.
- Show unsaved indicator per section when edited.
- Keep footer actions fixed: Cancel, Save.

2. Field behavior

- Provider first, model second. Model list must be filtered by provider.
- URL/API key fields appear only when provider requires them.
- Add "Test Connection" for online providers.
- API key fields should support "Reveal" and "Clear".

3. Validation

- URL validation: absolute URL + allowed schema (https preferred).
- API key required only for selected providers that need auth.
- Inline errors near fields, not only toast.

4. Defaults

- Theme: Night
- Base Language: English
- First Translate: Thai
- Second Translate: None
- STT Mode: online
- STT Provider: browser (or openai-compatible if you want API-first behavior)
- Translate Provider: none or openai-compatible (choose one product direction and keep consistent)

## Proposed Information Architecture

### Left Menu

- Theme
- Language
- AI

### Right Panel: Theme

- Color: Day | Night | Toy
- Highlight first language: toggle

### Right Panel: Language

- Base Language
- First Translate
- Second Translate

### Right Panel: AI

#### Speech to Text

- Mode: online | offline
- Provider: openai-compatible | browser | vosk | gemini | deepinfra (optional)
- Model: dynamic list based on provider
- URL (only openai-compatible/custom provider)
- API Key (if required)
- Test Connection

#### Translate

- Provider: openai-compatible | deepinfra | ollama | none
- Model: qwen | custom | provider-specific list
- URL (only openai-compatible/custom provider)
- API Key (if required)
- Test Connection

## Data Model Proposal

Persist a dedicated structured settings object (instead of many unrelated single keys).

```ts
interface AppSettingsV2 {
	theme: {
		color: "day" | "night" | "toy";
		highlightFirstLanguage: boolean;
	};
	language: {
		baseLanguage: string; // e.g. en-US
		firstTranslate: string; // e.g. th
		secondTranslate: string | null; // null == None
	};
	ai: {
		stt: {
			mode: "online" | "offline";
			provider: "openai-compatible" | "browser" | "vosk" | "gemini" | "deepinfra";
			model: string;
			endpoint: string | null;
			apiKeyRef: string | null; // key name in stronghold
		};
		translate: {
			provider: "openai-compatible" | "deepinfra" | "ollama" | "none";
			model: string;
			endpoint: string | null;
			apiKeyRef: string | null;
		};
	};
}
```

## Where Each Setting Is Used

### Theme settings

- Used in root app class/attributes in [src/app.svelte](src/app.svelte).
- Affects CSS variables in [src/app.css](src/app.css).
- highlightFirstLanguage affects first translated line style in subtitle row component (likely [src/components/SubtitleLine.svelte](src/components/SubtitleLine.svelte)).

### Language settings

- baseLanguage maps to speech recognition language in [src/lib/speech.svelte.ts](src/lib/speech.svelte.ts).
- firstTranslate + secondTranslate drive translation pipeline from [src/app.svelte](src/app.svelte).
- secondTranslate requires updating subtitle rendering to show up to 2 translated outputs.

### AI STT settings

- mode/provider/model drive start path in [src/lib/speech.svelte.ts](src/lib/speech.svelte.ts):
  - browser -> Web Speech API
  - vosk -> local model flow
  - openai-compatible/deepinfra/gemini -> remote STT flow with endpoint+key
- endpoint/key are consumed by backend remote ASR command path in [src-tauri/src/remote_asr.rs](src-tauri/src/remote_asr.rs).

### AI Translate settings

- provider/model/endpoint/key are used by translate pipeline call site in [src/app.svelte](src/app.svelte) and translate command in [src-tauri/src/commands.rs](src-tauri/src/commands.rs).
- translate command must be upgraded from stub to real provider routing.

## Security and Secrets

- Store only key references in settings JSON.
- Store API key material in Stronghold (frontend wrapper in [src/lib/stronghold.ts](src/lib/stronghold.ts)).
- Use separate key refs:
  - ai.stt.apiKeyRef
  - ai.translate.apiKeyRef
- Never log raw keys.

## Implementation Plan (Phased)

### Phase 1: UX shell and navigation

1. Replace current bottom sheet with centered modal layout in [src/components/Settings.svelte](src/components/Settings.svelte).
2. Add left navigation state and right panel section rendering.
3. Add Save/Cancel footer with dirty-state handling.

Deliverable: macOS-like settings modal structure with static fields.

### Phase 2: Settings schema migration

1. Extend backend settings schema in [src-tauri/src/commands.rs](src-tauri/src/commands.rs) to V2 structure.
2. Add migration from old flat keys to V2 defaults.
3. Keep backward compatibility for one release.

Deliverable: stable persistence for new fields.

### Phase 3: STT provider matrix

1. Add provider/model option maps in frontend.
2. Wire STT mode/provider/model/endpoint/key into [src/lib/speech.svelte.ts](src/lib/speech.svelte.ts).
3. Extend remote start command contract if needed to pass provider/model/endpoint explicitly.
4. Add Test Connection action.

Deliverable: user-selectable OpenAI-compatible STT + existing offline/browser options.

### Phase 4: Translation provider implementation

1. Replace translate stub in [src-tauri/src/commands.rs](src-tauri/src/commands.rs) with provider routing.
2. Implement OpenAI-compatible translation request path + model + endpoint.
3. Implement deepinfra/ollama adapters (or mark unsupported providers disabled until ready).
4. Add optional second translate output in UI.

Deliverable: configurable translation providers with endpoint/key/model.

### Phase 5: Polish and safety

1. Field-level validation and inline error messages.
2. Connection tests and timeout handling.
3. UX details: loading states, disabled Save, per-section reset.
4. Regression tests (frontend and Rust unit tests).

Deliverable: production-ready settings UX.

## Acceptance Criteria

- User can configure OpenAI-compatible STT with custom URL + API key.
- User can configure OpenAI-compatible Translate with custom URL + API key.
- STT and Translate credentials are separated.
- Settings modal uses left menu/right content layout and opens centered.
- Defaults apply correctly on first launch.
- Invalid settings are blocked with clear inline feedback.
- No plaintext secrets in logs or settings JSON.

## Recommended Product Decisions (Need Confirmation)

1. Theme "Toy" visual definition

- Decide exact palette, contrast constraints, and whether it applies globally or only subtitles.

2. Gemini support scope

- Decide if Gemini is STT only, Translate only, or both in first release.

3. Second Translate display

- Decide if second translation is always visible or optional toggle in Listening view.

4. Provider availability policy

- Decide whether unsupported providers appear disabled (with tooltip) or hidden.

## Suggested Next Implementation Slice

Start with Phase 1 + Phase 2 only. This gives a clear UI foundation and safe data model before touching network logic.
