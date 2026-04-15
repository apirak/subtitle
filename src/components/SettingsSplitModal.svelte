<script lang="ts">
	import { SOURCE_LANGUAGES, TARGET_LANGUAGES } from "../lib/languages";
	import {
		translateWithOpenAI,
		validateTranslationConfig,
		type TranslationConfig,
	} from "../lib/openai-translator";
	import { resolveOpenAICompatibleEndpoint } from "../lib/api-connection";
	import SettingsSidebar from "./setting/SettingsSidebar.svelte";
	import SettingsPanelHeader from "./setting/SettingsPanelHeader.svelte";

	type SettingsSection = "theme" | "language" | "tts" | "translate";
	type ThemeMode = "night" | "day" | "toy";

	interface Props {
		open: boolean;
		activeSection: SettingsSection;
		theme: ThemeMode;
		language: string;
		targetLang: string;
		targetLang2: string;
		engine: "browser" | "vosk" | "remote";
		remoteEndpoint: string;
		apiKey: string;
		translationEngine: string;
		translationEndpoint: string;
		translationModel: string;
		translationApiKey: string;
		onThemeChange: (theme: ThemeMode) => void;
		onLanguageChange: (language: string) => void;
		onTargetLangChange: (language: string) => void;
		onTargetLang2Change: (language: string) => void;
		onEngineChange: (engine: "browser" | "vosk" | "remote") => void;
		onRemoteEndpointChange: (value: string) => void;
		onApiKeyChange: (value: string) => void;
		onTranslationEngineChange: (engine: string) => void;
		onTranslationEndpointChange: (value: string) => void;
		onTranslationModelChange: (value: string) => void;
		onTranslationApiKeyChange: (value: string) => void;
		onSectionChange: (section: SettingsSection) => void;
		onClose: () => void;
	}

	let {
		open,
		activeSection,
		theme,
		language,
		targetLang,
		targetLang2,
		engine,
		remoteEndpoint,
		apiKey,
		translationEngine,
		translationEndpoint,
		translationModel,
		translationApiKey,
		onThemeChange,
		onLanguageChange,
		onTargetLangChange,
		onTargetLang2Change,
		onEngineChange,
		onRemoteEndpointChange,
		onApiKeyChange,
		onTranslationEngineChange,
		onTranslationEndpointChange,
		onTranslationModelChange,
		onTranslationApiKeyChange,
		onSectionChange,
		onClose,
	}: Props = $props();
	let currentSection = $state<SettingsSection>("theme");
	let currentTheme = $state<ThemeMode>("night");
	let currentLanguage = $state("");
	let currentTargetLang = $state("");
	let currentTargetLang2 = $state("none");
	let currentEngine = $state<"browser" | "vosk" | "remote">("browser");
	let currentRemoteEndpoint = $state("");
	let currentApiKey = $state("");
	let currentTranslationEngine = $state("remote");
	let currentTranslationEndpoint = $state("");
	let currentTranslationModel = $state("");
	let currentTranslationApiKey = $state("");
	let isTestingTranslate = $state(false);
	let testTranslateError = $state("");
	let testTranslateResult = $state("");
	let testTranslateSourceText = $state("");
	let testTranslateResolvedUrl = $state("");

	const menuItems: Array<{ key: SettingsSection; label: string }> = [
		{ key: "theme", label: "Theme" },
		{ key: "language", label: "Language" },
		{ key: "tts", label: "TTS" },
		{ key: "translate", label: "Translate" },
	];

	const sectionTitles: Record<SettingsSection, string> = {
		theme: "Theme",
		language: "Language",
		tts: "TTS",
		translate: "Translate",
	};

	const sourceOptions = SOURCE_LANGUAGES.map((item) => ({ value: item.code, label: item.label }));
	const targetOptions = TARGET_LANGUAGES.map((item) => ({ value: item.value, label: item.label }));
	const targetOptionsWithNone = [{ value: "none", label: "None" }, ...targetOptions];
	const asrEngineOptions: Array<{ value: "browser" | "vosk" | "remote"; label: string }> = [
		{ value: "browser", label: "Browser (Web Speech API)" },
		{ value: "vosk", label: "Vosk (On-device)" },
		{ value: "remote", label: "Remote (API)" },
	];
	const translationEngineOptions = [
		{ value: "none", label: "None" },
		{ value: "remote", label: "Remote (OpenAI-compatible)" },
	];

	$effect(() => {
		currentSection = activeSection;
	});

	$effect(() => {
		currentTheme = theme;
	});

	$effect(() => {
		currentLanguage = language;
	});

	$effect(() => {
		currentTargetLang = targetLang;
	});

	$effect(() => {
		currentTargetLang2 = targetLang2;
	});

	$effect(() => {
		currentEngine = engine;
	});

	$effect(() => {
		currentRemoteEndpoint = remoteEndpoint;
	});

	$effect(() => {
		currentApiKey = apiKey;
	});

	$effect(() => {
		currentTranslationEngine = translationEngine;
	});

	$effect(() => {
		currentTranslationEndpoint = translationEndpoint;
	});

	$effect(() => {
		currentTranslationModel = translationModel;
	});

	$effect(() => {
		currentTranslationApiKey = translationApiKey;
	});

	function handleSectionClick(section: SettingsSection) {
		currentSection = section;
		onSectionChange(section);
	}

	function handleThemeClick(nextTheme: ThemeMode) {
		currentTheme = nextTheme;
		onThemeChange(nextTheme);
	}

	function handleLanguageChange(nextLanguage: string) {
		currentLanguage = nextLanguage;
		onLanguageChange(nextLanguage);
	}

	function handleTargetLangChange(nextLanguage: string) {
		currentTargetLang = nextLanguage;
		onTargetLangChange(nextLanguage);
	}

	function handleTargetLang2Change(nextLanguage: string) {
		currentTargetLang2 = nextLanguage;
		onTargetLang2Change(nextLanguage);
	}

	function handleEngineChange(nextEngine: "browser" | "vosk" | "remote") {
		currentEngine = nextEngine;
		onEngineChange(nextEngine);
	}

	function handleRemoteEndpointChange(nextValue: string) {
		currentRemoteEndpoint = nextValue;
		onRemoteEndpointChange(nextValue);
	}

	function handleApiKeyChange(nextValue: string) {
		currentApiKey = nextValue;
		onApiKeyChange(nextValue);
	}

	function handleTranslationEngineChange(nextValue: string) {
		currentTranslationEngine = nextValue;
		onTranslationEngineChange(nextValue);
	}

	function handleTranslationEndpointChange(nextValue: string) {
		currentTranslationEndpoint = nextValue;
		onTranslationEndpointChange(nextValue);
	}

	function handleTranslationModelChange(nextValue: string) {
		currentTranslationModel = nextValue;
		onTranslationModelChange(nextValue);
	}

	function handleTranslationApiKeyChange(nextValue: string) {
		currentTranslationApiKey = nextValue;
		onTranslationApiKeyChange(nextValue);
	}

	function getTranslationTestSample(languageCode: string): string {
		if (languageCode.startsWith("th")) {
			return "สวัสดี นี่คือการทดสอบการแปล";
		}

		if (languageCode.startsWith("ja")) {
			return "こんにちは、これは翻訳テストです";
		}

		if (languageCode.startsWith("zh")) {
			return "你好，这是翻译测试";
		}

		return "Hello, this is a translation test.";
	}

	async function handleTestTranslate() {
		testTranslateError = "";
		testTranslateResult = "";
		testTranslateSourceText = "";
		testTranslateResolvedUrl = resolveOpenAICompatibleEndpoint(currentTranslationEndpoint);

		const config: TranslationConfig = {
			engine: currentTranslationEngine === "none" ? "none" : "remote",
			model: currentTranslationModel,
			endpoint: currentTranslationEndpoint,
			apiKey: currentTranslationApiKey,
		};

		const validation = validateTranslationConfig(config);
		if (!validation.valid) {
			testTranslateError = validation.error ?? "Translation configuration is invalid";
			return;
		}

		const sourceLanguage = currentLanguage || "en-US";
		const targetLanguage = currentTargetLang || "th";
		const sampleText = getTranslationTestSample(sourceLanguage);

		testTranslateSourceText = sampleText;
		isTestingTranslate = true;

		try {
			const translatedText = await translateWithOpenAI(sampleText, sourceLanguage, targetLanguage, config);
			testTranslateResult = translatedText;
		} catch (error) {
			testTranslateError = error instanceof Error ? error.message : String(error);
		} finally {
			isTestingTranslate = false;
		}
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="settings-modal fixed inset-0 z-120 flex items-center justify-center p-4"
	style:opacity={open ? "1" : "0"}
	style:pointer-events={open ? "auto" : "none"}
	style:transition="opacity 180ms ease"
	onclick={onClose}
>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="settings-shell flex w-full max-w-230 min-h-130 max-h-[calc(100vh-2rem)] overflow-hidden rounded-2xl"
		onclick={(event) => event.stopPropagation()}
	>
		<SettingsSidebar activeSection={currentSection} {menuItems} onSectionChange={handleSectionClick} />

		<section class="flex min-h-0 flex-1 flex-col p-6">
			<SettingsPanelHeader title="{sectionTitles[currentSection]} Settings" {onClose} />

			<div class="settings-panel">
				{#if currentSection === "theme"}
					<div class="mx-auto w-full max-w-140">
						<div class="settings-section-label">Choose Theme</div>
						<div class="grid grid-cols-3 gap-3">
							<button
								type="button"
								onclick={() => handleThemeClick("night")}
								class="theme-card"
								data-selected={currentTheme === "night" ? "true" : undefined}
							>
								<div class="theme-card-title">Night</div>
								<div class="theme-card-description">Dark and focused</div>
							</button>

							<button
								type="button"
								onclick={() => handleThemeClick("day")}
								class="theme-card"
								data-selected={currentTheme === "day" ? "true" : undefined}
							>
								<div class="theme-card-title">Day</div>
								<div class="theme-card-description">Light and clear</div>
							</button>

							<button
								type="button"
								onclick={() => handleThemeClick("toy")}
								class="theme-card"
								data-selected={currentTheme === "toy" ? "true" : undefined}
							>
								<div class="theme-card-title">Toy</div>
								<div class="theme-card-description">Warm and playful</div>
							</button>
						</div>
					</div>
				{:else if currentSection === "language"}
					<div class="mx-auto w-full max-w-140">
						<div class="settings-section-label">Speech And Translation</div>

						<div class="mb-5">
							<label class="settings-field-label" for="split-source-language">Source Language</label>
							<select
								id="split-source-language"
								class="settings-select"
								value={currentLanguage}
								onchange={(event) => handleLanguageChange((event.target as HTMLSelectElement).value)}
							>
								{#each sourceOptions as option}
									<option value={option.value}>{option.label}</option>
								{/each}
							</select>
						</div>

						<div class="mb-5">
							<label class="settings-field-label" for="split-target-language">Translate To</label>
							<select
								id="split-target-language"
								class="settings-select"
								value={currentTargetLang}
								onchange={(event) => handleTargetLangChange((event.target as HTMLSelectElement).value)}
							>
								{#each targetOptions as option}
									<option value={option.value}>{option.label}</option>
								{/each}
							</select>
						</div>

						<div>
							<label class="settings-field-label" for="split-target-language-2">Translation 2</label>
							<select
								id="split-target-language-2"
								class="settings-select"
								value={currentTargetLang2}
								onchange={(event) => handleTargetLang2Change((event.target as HTMLSelectElement).value)}
							>
								{#each targetOptionsWithNone as option}
									<option value={option.value}>{option.label}</option>
								{/each}
							</select>
						</div>

						<p class="settings-helper-text">
							Source language is column 1. Translation 1 and Translation 2 are columns 2 and 3.
						</p>
					</div>
				{:else if currentSection === "tts"}
					<div class="mx-auto w-full max-w-140">
						<div class="settings-section-label">Speech To Text</div>

						<div class="mb-5">
							<div class="settings-field-label">ASR Engine</div>
							<div class="settings-radio-group" role="radiogroup" aria-label="ASR Engine">
								{#each asrEngineOptions as option}
									<label class="settings-radio-option" data-selected={currentEngine === option.value ? "true" : undefined}>
										<input
											type="radio"
											name="split-asr-engine"
											class="settings-radio-input"
											checked={currentEngine === option.value}
											onchange={() => handleEngineChange(option.value)}
										/>
										<div>
											<div class="settings-radio-title">{option.label}</div>
											<div class="settings-radio-description">
												{#if option.value === "browser"}
													Use built-in Web Speech API in the webview.
												{:else if option.value === "vosk"}
													Use the local on-device Vosk recognizer.
												{:else}
													Use a remote OpenAI-compatible transcription API.
												{/if}
											</div>
										</div>
									</label>
								{/each}
							</div>
						</div>

						{#if currentEngine === "remote"}
							<div class="mb-5">
								<label class="settings-field-label" for="split-remote-endpoint">API Endpoint</label>
								<input
									id="split-remote-endpoint"
									type="url"
									class="settings-input"
									placeholder="https://api.example.com/v1/audio/transcriptions"
									value={currentRemoteEndpoint}
									oninput={(event) =>
										handleRemoteEndpointChange((event.target as HTMLInputElement).value)}
								/>
							</div>

							<div>
								<label class="settings-field-label" for="split-api-key">API Key</label>
								<input
									id="split-api-key"
									type="password"
									class="settings-input"
									placeholder="sk-..."
									value={currentApiKey}
									onchange={(event) => handleApiKeyChange((event.target as HTMLInputElement).value)}
								/>
							</div>
						{/if}

						<p class="settings-helper-text">
							Use Browser/Vosk for local recognition, or Remote for OpenAI-compatible transcription APIs.
						</p>
					</div>
				{:else if currentSection === "translate"}
					<div class="mx-auto w-full max-w-140">
						<div class="settings-section-label">Translation</div>

						<div class="mb-5">
							<div class="settings-field-label">Translation Engine</div>
							<div class="settings-radio-group" role="radiogroup" aria-label="Translation Engine">
								{#each translationEngineOptions as option}
									<label class="settings-radio-option" data-selected={currentTranslationEngine === option.value ? "true" : undefined}>
										<input
											type="radio"
											name="split-translation-engine"
											class="settings-radio-input"
											checked={currentTranslationEngine === option.value}
											onchange={() => handleTranslationEngineChange(option.value)}
										/>
										<div>
											<div class="settings-radio-title">{option.label}</div>
											<div class="settings-radio-description">
												{#if option.value === "none"}
													Disable translated columns and show only source text.
												{:else if option.value === "remote"}
													Use an OpenAI-compatible chat completion endpoint.
												{/if}
											</div>
										</div>
									</label>
								{/each}
							</div>
						</div>

						{#if currentTranslationEngine !== "none"}
							<div class="mb-5">
								<label class="settings-field-label" for="split-translation-model">Model</label>
								<input
									id="split-translation-model"
									type="text"
									class="settings-input"
									placeholder="Qwen3-32B"
									value={currentTranslationModel}
									oninput={(event) => handleTranslationModelChange((event.target as HTMLInputElement).value)}
								/>
							</div>

							<div class="mb-5">
								<label class="settings-field-label" for="split-translation-endpoint">Endpoint</label>
								<input
									id="split-translation-endpoint"
									type="url"
									class="settings-input"
									placeholder="https://api.deepinfra.com/v1/openai"
									value={currentTranslationEndpoint}
									oninput={(event) => handleTranslationEndpointChange((event.target as HTMLInputElement).value)}
								/>
							</div>

							<div class="mb-5">
								<label class="settings-field-label" for="split-translation-api-key">API Key</label>
								<input
									id="split-translation-api-key"
									type="password"
									class="settings-input"
									placeholder="sk-..."
									value={currentTranslationApiKey}
									onchange={(event) => handleTranslationApiKeyChange((event.target as HTMLInputElement).value)}
								/>
							</div>

							<div class="mt-5">
								<button
									type="button"
									class="settings-test-button"
									onclick={handleTestTranslate}
									disabled={isTestingTranslate}
								>
									{#if isTestingTranslate}Testing…{:else}Test Translate{/if}
								</button>

								{#if testTranslateResolvedUrl}
									<div class="settings-test-meta">Resolved endpoint: {testTranslateResolvedUrl}</div>
								{/if}

								{#if testTranslateSourceText}
									<div class="settings-test-block">
										<div class="settings-test-label">Source sample</div>
										<div class="settings-test-value">{testTranslateSourceText}</div>
									</div>
								{/if}

								{#if testTranslateResult}
									<div class="settings-test-block">
										<div class="settings-test-label">Translated result</div>
										<div class="settings-test-value">{testTranslateResult}</div>
									</div>
								{/if}

								{#if testTranslateError}
									<div class="settings-test-error">{testTranslateError}</div>
								{/if}
							</div>
						{/if}

						<p class="settings-helper-text">
							Configure the translator used for both Translation 1 and Translation 2 columns.
						</p>
					</div>
				{:else}
					<div class="settings-placeholder">
						{sectionTitles[currentSection]} section is selected. UI elements will be added in the next slices.
					</div>
				{/if}
			</div>
		</section>
	</div>
</div>

<style>
	.settings-modal {
		background: var(--overlay-backdrop);
		backdrop-filter: blur(8px);
	}

	.settings-shell {
		background: var(--surface-color);
		border: 1px solid var(--border-color-default);
		box-shadow: 0 32px 80px rgba(0, 0, 0, 0.35);
		color: var(--on-surface-color);
	}

	.settings-section-label,
	.settings-field-label {
		color: var(--muted-color);
		font-size: 0.75rem;
		font-weight: 600;
		letter-spacing: 0.08em;
		text-transform: uppercase;
	}

	.settings-section-label {
		margin-bottom: 1rem;
	}

	.settings-field-label {
		display: block;
		margin-bottom: 0.5rem;
	}

	.settings-panel {
		min-height: 410px;
		overflow-y: auto;
		border-radius: 1rem;
		border: 1px solid var(--border-color-default);
		background: var(--surface-elevated-color);
		padding: 2rem;
		font-size: 0.875rem;
		color: var(--on-surface-color);
	}

	.theme-card,
	.settings-select,
	.settings-input,
	.settings-radio-option {
		border: 1px solid var(--border-color-default);
		background: var(--field-color);
		color: var(--on-surface-color-strong);
		transition:
			background-color 160ms ease,
			border-color 160ms ease,
			color 160ms ease,
			box-shadow 160ms ease;
	}

	.theme-card {
		cursor: pointer;
		border-radius: 1rem;
		padding: 1rem;
		text-align: left;
	}

	.theme-card:hover,
	.settings-radio-option:hover,
	.settings-select:hover,
	.settings-input:hover {
		background: var(--field-hover-color);
		border-color: var(--border-color-strong);
	}

	.theme-card[data-selected="true"],
	.settings-radio-option[data-selected="true"] {
		background: var(--accent-color);
		border-color: var(--accent-color);
		color: var(--on-accent-color);
	}

	.theme-card-title,
	.settings-radio-title {
		font-size: 0.875rem;
		font-weight: 600;
	}

	.theme-card-description,
	.settings-radio-description,
	.settings-helper-text,
	.settings-placeholder {
		color: var(--on-surface-color);
	}

	.theme-card-description,

	.settings-test-button {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		padding: 0.7rem 1rem;
		border-radius: 0.75rem;
		border: 1px solid var(--border-color-strong);
		background: var(--surface-hover-color);
		color: var(--on-surface-color-strong);
		font-weight: 600;
		cursor: pointer;
		transition:
			background-color 160ms ease,
			border-color 160ms ease,
			opacity 160ms ease;
	}

	.settings-test-button:hover:not(:disabled) {
		background: var(--field-color);
		border-color: var(--on-surface-color);
	}

	.settings-test-button:disabled {
		opacity: 0.7;
		cursor: progress;
	}

	.settings-test-meta,
	.settings-test-error,
	.settings-test-value {
		margin-top: 0.75rem;
		word-break: break-word;
	}

	.settings-test-meta {
		font-size: 0.74rem;
		color: var(--muted-color);
	}

	.settings-test-block {
		margin-top: 0.9rem;
		padding: 0.9rem 1rem;
		border-radius: 0.85rem;
		border: 1px solid var(--border-color-default);
		background: var(--field-color);
	}

	.settings-test-label {
		font-size: 0.72rem;
		font-weight: 700;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--muted-color);
	}

	.settings-test-value {
		font-size: 0.9rem;
		line-height: 1.5;
		color: var(--on-surface-color-strong);
	}

	.settings-endpoint-hint {
		margin-top: 0.45rem;
		font-size: 0.74rem;
		color: var(--muted-color);
		line-height: 1.5;
	}

	.settings-endpoint-hint code {
		font-family: ui-monospace, monospace;
		background: var(--surface-hover-color);
		padding: 0.1em 0.35em;
		border-radius: 0.3em;
	}

	.settings-test-error {
		padding: 0.8rem 0.9rem;
		border-radius: 0.85rem;
		border: 1px solid color-mix(in srgb, var(--danger-color) 45%, transparent);
		background: color-mix(in srgb, var(--danger-color) 10%, transparent);
		color: var(--danger-color);
		font-size: 0.82rem;
	}
	.settings-radio-description {
		margin-top: 0.25rem;
		font-size: 0.75rem;
		line-height: 1.45;
	}

	.theme-card[data-selected="true"] .theme-card-title,
	.theme-card[data-selected="true"] .theme-card-description,
	.settings-radio-option[data-selected="true"] .settings-radio-title,
	.settings-radio-option[data-selected="true"] .settings-radio-description {
		color: var(--on-accent-color);
	}

	.settings-select,
	.settings-input {
		width: 100%;
		border-radius: 1rem;
		padding: 0.875rem 1rem;
		outline: none;
	}

	.settings-select:focus,
	.settings-input:focus {
		border-color: var(--accent-color);
		box-shadow: 0 0 0 3px var(--focus-ring-color);
	}

	.settings-input::placeholder {
		color: var(--placeholder-color);
	}

	.settings-radio-group {
		display: grid;
		gap: 0.75rem;
	}

	.settings-radio-option {
		display: flex;
		align-items: flex-start;
		gap: 0.75rem;
		border-radius: 1rem;
		padding: 0.875rem 1rem;
	}

	.settings-radio-input {
		margin-top: 0.125rem;
		height: 1rem;
		width: 1rem;
		accent-color: var(--accent-color);
	}

	.settings-radio-option[data-selected="true"] .settings-radio-input {
		accent-color: var(--on-accent-color);
	}

	.settings-helper-text {
		margin-top: 1.25rem;
		font-size: 0.875rem;
		line-height: 1.5;
	}

	.settings-placeholder {
		display: grid;
		height: 100%;
		place-items: center;
	}
</style>