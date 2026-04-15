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
	import ThemeSettingsPanel from "./setting/panels/ThemeSettingsPanel.svelte";
	import LanguageSettingsPanel from "./setting/panels/LanguageSettingsPanel.svelte";
	import SttSettingsPanel from "./setting/panels/SttSettingsPanel.svelte";
	import TranslationSettingsPanel from "./setting/panels/TranslationSettingsPanel.svelte";

	type SettingsSection = "theme" | "language" | "stt" | "translate";
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
		{ key: "stt", label: "Speech To Text" },
		{ key: "translate", label: "Translate" },
	];

	const sectionTitles: Record<SettingsSection, string> = {
		theme: "Theme",
		language: "Language",
		stt: "Speech To Text",
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

	function formatMaskedKeyHint(value: string): string {
		const normalized = value.trim();
		if (!normalized) return "";
		const last4 = normalized.slice(-4);
		return `Saved key: ••••${last4}`;
	}

	let remoteApiKeyHint = $derived(formatMaskedKeyHint(currentApiKey));
	let translationApiKeyHint = $derived(formatMaskedKeyHint(currentTranslationApiKey));

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
					<ThemeSettingsPanel currentTheme={currentTheme} onThemeChange={handleThemeClick} />
				{:else if currentSection === "language"}
					<LanguageSettingsPanel
						sourceOptions={sourceOptions}
						targetOptions={targetOptions}
						targetOptionsWithNone={targetOptionsWithNone}
						currentLanguage={currentLanguage}
						currentTargetLang={currentTargetLang}
						currentTargetLang2={currentTargetLang2}
						onLanguageChange={handleLanguageChange}
						onTargetLangChange={handleTargetLangChange}
						onTargetLang2Change={handleTargetLang2Change}
					/>
				{:else if currentSection === "stt"}
					<SttSettingsPanel
						asrEngineOptions={asrEngineOptions}
						currentEngine={currentEngine}
						currentRemoteEndpoint={currentRemoteEndpoint}
						currentApiKey={currentApiKey}
						remoteApiKeyHint={remoteApiKeyHint}
						onEngineChange={handleEngineChange}
						onRemoteEndpointChange={handleRemoteEndpointChange}
						onApiKeyChange={handleApiKeyChange}
					/>
				{:else if currentSection === "translate"}
					<TranslationSettingsPanel
						translationEngineOptions={translationEngineOptions}
						currentTranslationEngine={currentTranslationEngine}
						currentTranslationModel={currentTranslationModel}
						currentTranslationEndpoint={currentTranslationEndpoint}
						currentTranslationApiKey={currentTranslationApiKey}
						translationApiKeyHint={translationApiKeyHint}
						isTestingTranslate={isTestingTranslate}
						testTranslateResolvedUrl={testTranslateResolvedUrl}
						testTranslateSourceText={testTranslateSourceText}
						testTranslateResult={testTranslateResult}
						testTranslateError={testTranslateError}
						onTranslationEngineChange={handleTranslationEngineChange}
						onTranslationModelChange={handleTranslationModelChange}
						onTranslationEndpointChange={handleTranslationEndpointChange}
						onTranslationApiKeyChange={handleTranslationApiKeyChange}
						onTestTranslate={handleTestTranslate}
					/>
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

	.settings-placeholder {
		display: grid;
		height: 100%;
		place-items: center;
		color: var(--on-surface-color);
	}
</style>