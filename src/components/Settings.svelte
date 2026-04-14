<script lang="ts">
	import Dropdown from "./Dropdown.svelte";
	import { SOURCE_LANGUAGES, TARGET_LANGUAGES } from "../lib/languages";

	interface Props {
		open: boolean;
		language: string;
		onLanguageChange: (lang: string) => void;
		targetLang: string;
		onTargetLangChange: (lang: string) => void;
		subtitlePosition: number;
		onSubtitlePositionChange: (pos: number) => void;
		onClose: () => void;
		translationEngine: string;
		onTranslationEngineChange: (engine: string) => void;
		overlayTransparency: number;
		onOverlayTransparencyChange: (value: number) => void;
		fontSize: number;
		onFontSizeChange: (value: number) => void;
		engine: string;
		onEngineChange: (engine: string) => void;
		remoteEndpoint: string;
		onRemoteEndpointChange: (value: string) => void;
		apiKey: string;
		onApiKeyChange: (value: string) => void;
	}

	let {
		open,
		language,
		onLanguageChange,
		targetLang,
		onTargetLangChange,
		subtitlePosition,
		onSubtitlePositionChange,
		onClose,
		translationEngine,
		onTranslationEngineChange,
		overlayTransparency,
		onOverlayTransparencyChange,
		fontSize,
		onFontSizeChange,
		engine,
		onEngineChange,
		remoteEndpoint,
		onRemoteEndpointChange,
		apiKey,
		onApiKeyChange,
	}: Props = $props();

	const sourceOptions = SOURCE_LANGUAGES.map((l) => ({ value: l.code, label: l.label }));
	const targetOptions = TARGET_LANGUAGES.map((l) => ({ value: l.value, label: l.label }));
	const engineOptions = [
		{ value: "browser", label: "Browser (Web Speech API)" },
		{ value: "vosk", label: "Vosk (On-Device)" },
		{ value: "remote", label: "Remote (API)" },
	];
	const translationEngineOptions = [
		{ value: "none", label: "None" },
		{ value: "ollama", label: "Ollama (Local)" },
		{ value: "nllb", label: "NLLB (On-Device)" },
		{ value: "remote", label: "Remote (API)" },
	];
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="fixed inset-0 z-[100] flex items-end justify-center transition-colors duration-[300ms] ease"
	style:background={open ? "rgba(0, 0, 0, 0.4)" : "rgba(0, 0, 0, 0)"}
	style:pointer-events={open ? "auto" : "none"}
	onclick={onClose}
>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="w-full max-w-[420px] max-h-[540px] bg-[rgba(28,28,30,0.95)] backdrop-blur-[40px] rounded-t-[20px] transition-transform duration-[350ms] overflow-y-auto"
		style="padding: 0 0 40px;"
		style:transform={open ? "translateY(0)" : "translateY(100%)"}
		style:transition-timing-function="cubic-bezier(0.32, 0.72, 0, 1)"
		onclick={(e) => e.stopPropagation()}
	>
		<div class="flex items-center justify-between" style="padding: 20px 24px 12px;">
			<span class="text-[1.05rem] font-semibold text-white/90">Settings</span>
			<button
				class="flex items-center justify-center w-8 h-8 rounded-full bg-[rgba(255,255,255,0.08)] border-0 text-white/50 cursor-pointer transition-all duration-200 ease hover:bg-[rgba(255,255,255,0.15)] hover:text-white"
				onclick={onClose}
				aria-label="Close settings"
			>
				<svg
					width="20"
					height="20"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<line x1="18" y1="6" x2="6" y2="18" />
					<line x1="6" y1="6" x2="18" y2="18" />
				</svg>
			</button>
		</div>

		<div class="flex flex-col gap-6" style="padding: 8px 24px 0;">
			<div class="flex flex-col gap-1.5">
				<label class="text-xs font-semibold text-white/40 uppercase tracking-[0.06em]" for="source-lang"
					>Source Language</label
				>
				<Dropdown id="source-lang" value={language} options={sourceOptions} onchange={onLanguageChange} />
			</div>

			<div class="flex flex-col gap-1.5">
				<label class="text-xs font-semibold text-white/40 uppercase tracking-[0.06em]" for="target-lang"
					>Translate To</label
				>
				<Dropdown id="target-lang" value={targetLang} options={targetOptions} onchange={onTargetLangChange} />
			</div>

			<div class="flex flex-col gap-1.5">
				<label class="text-xs font-semibold text-white/40 uppercase tracking-[0.06em]" for="subtitle-pos"
					>Subtitle Position — {subtitlePosition}%</label
				>
				<input
					id="subtitle-pos"
					type="range"
					min={5}
					max={90}
					value={subtitlePosition}
					oninput={(e) => onSubtitlePositionChange(Number((e.target as HTMLInputElement).value))}
					class="slider"
				/>
				<div class="flex justify-between text-[0.65rem] text-white/20">
					<span>Bottom</span>
					<span>Top</span>
				</div>
			</div>

			<!-- TRANSLATION SECTION -->
			<div class="text-xs font-semibold text-white/40 uppercase tracking-[0.06em]">Translation</div>

			<div class="flex flex-col gap-1.5">
				<label class="text-xs font-semibold text-white/40 uppercase tracking-[0.06em]" for="translation-engine"
					>Translation Engine</label
				>
				<Dropdown
					id="translation-engine"
					value={translationEngine}
					options={translationEngineOptions}
					onchange={onTranslationEngineChange}
				/>
			</div>

			<!-- OVERLAY APPEARANCE SECTION -->
			<div class="text-xs font-semibold text-white/40 uppercase tracking-[0.06em]">Overlay Appearance</div>

			<div class="flex flex-col gap-1.5">
				<label class="text-xs font-semibold text-white/40 uppercase tracking-[0.06em]" for="overlay-transparency"
					>Overlay Transparency — {overlayTransparency}%</label
				>
				<input
					id="overlay-transparency"
					type="range"
					min={0}
					max={100}
					value={overlayTransparency}
					oninput={(e) => onOverlayTransparencyChange(Number((e.target as HTMLInputElement).value))}
					class="slider"
				/>
				<div class="flex justify-between text-[0.65rem] text-white/20">
					<span>Invisible</span>
					<span>Opaque</span>
				</div>
			</div>

			<div class="flex flex-col gap-1.5">
				<label class="text-xs font-semibold text-white/40 uppercase tracking-[0.06em]" for="font-size"
					>Font Size — {fontSize}px</label
				>
				<input
					id="font-size"
					type="range"
					min={12}
					max={48}
					value={fontSize}
					oninput={(e) => onFontSizeChange(Number((e.target as HTMLInputElement).value))}
					class="slider"
				/>
				<div class="flex justify-between text-[0.65rem] text-white/20">
					<span>12px</span>
					<span>48px</span>
				</div>
			</div>

			<!-- ADVANCED SECTION -->
			<div class="text-xs font-semibold text-white/40 uppercase tracking-[0.06em]" style="margin-top: 8px;">
				Advanced
			</div>

			<div class="flex flex-col gap-1.5">
				<label class="text-xs font-semibold text-white/40 uppercase tracking-[0.06em]" for="asr-engine"
					>ASR Engine</label
				>
				<Dropdown id="asr-engine" value={engine} options={engineOptions} onchange={onEngineChange} />
			</div>

			{#if engine === "remote"}
				<div class="flex flex-col gap-1.5">
					<label class="text-xs font-semibold text-white/40 uppercase tracking-[0.06em]" for="remote-endpoint">
						API Endpoint
					</label>
					<input
						id="remote-endpoint"
						type="url"
						placeholder="https://api.example.com/v1/audio/transcriptions"
						value={remoteEndpoint}
						oninput={(e) => onRemoteEndpointChange((e.target as HTMLInputElement).value)}
						class="w-full px-3 py-2 bg-[rgba(255,255,255,0.06)] border border-white/10 rounded-lg text-white/80 text-sm placeholder:text-white/20 focus:outline-none focus:border-white/30"
					/>
					<p class="text-[0.65rem] text-white/30">OpenAI-compatible transcription endpoint URL</p>
				</div>

				<div class="flex flex-col gap-1.5">
					<label class="text-xs font-semibold text-white/40 uppercase tracking-[0.06em]" for="api-key"> API Key </label>
					<input
						id="api-key"
						type="password"
						placeholder="sk-..."
						value={apiKey}
						onchange={(e) => onApiKeyChange((e.target as HTMLInputElement).value)}
						class="w-full px-3 py-2 bg-[rgba(255,255,255,0.06)] border border-white/10 rounded-lg text-white/80 text-sm placeholder:text-white/20 focus:outline-none focus:border-white/30"
					/>
					<p class="text-[0.65rem] text-white/30">Stored securely in OS keychain</p>
				</div>
			{/if}
		</div>
	</div>
</div>

<style>
	.slider {
		-webkit-appearance: none;
		appearance: none;
		width: 100%;
		height: 4px;
		border-radius: 2px;
		background: rgba(255, 255, 255, 0.1);
		outline: none;
		cursor: pointer;
	}

	.slider::-webkit-slider-thumb {
		-webkit-appearance: none;
		width: 18px;
		height: 18px;
		border-radius: 50%;
		background: #fff;
		cursor: pointer;
		box-shadow: 0 0 6px rgba(0, 0, 0, 0.3);
	}
</style>
