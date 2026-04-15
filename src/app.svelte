<script lang="ts">
	import { onDestroy, onMount } from "svelte";
	import { invoke } from "@tauri-apps/api/core";
	import { speech } from "./lib/speech.svelte";
	import { SOURCE_LANGUAGES, TARGET_LANGUAGES } from "./lib/languages";
	import { MOCK_SUBTITLES, MOCK_TRANSLATIONS, MOCK_TRANSLATIONS_2 } from "./lib/mockData";
	import { getApiKey } from "./lib/stronghold";
	import { resolveOpenAICompatibleEndpoint } from "./lib/api-connection";
	import IdleScreen from "./components/IdleScreen.svelte";
	import ListeningScreen from "./components/ListeningScreen.svelte";
	import ErrorScreen from "./components/ErrorScreen.svelte";
	import SettingsSplitModal from "./components/SettingsSplitModal.svelte";
	import "./app.css";

	type SettingsSection = "theme" | "language" | "stt" | "translate";

	onDestroy(() => {
		speech.destroy();
	});

	let targetLang = $state("th");
	let targetLang2 = $state("none");
	let theme = $state<"night" | "day" | "toy">("night");
	let translations1 = $state<Record<string, string>>({});
	let translations2 = $state<Record<string, string>>({});
	let settingsSplitOpen = $state(false);
	let settingsSection = $state<SettingsSection>("theme");
	let selectedEngine = $state<"browser" | "vosk" | "remote">("browser");
	let remoteEndpoint = $state("");
	let remoteModel = $state("");
	let modelPath = $state("");
	let apiKey = $state("");
	let translationEngine = $state("remote");
	let translationEndpoint = $state("");
	let translationModel = $state("");
	let translationApiKey = $state("");
	let isMockMode = $state(false);
	let isLoadingApiKeys = $state(true); // Block Start button until keys load
	let altPressed = $state(false);

	const translatedIds1 = new Set<string>();
	const translatedIds2 = new Set<string>();
	const inFlight1 = new Set<string>();
	const inFlight2 = new Set<string>();
	const recentLines: string[] = [];

	onMount(() => {
		const handleKeyDown = (event: KeyboardEvent) => {
			if (event.altKey) altPressed = true;
		};

		const handleKeyUp = (event: KeyboardEvent) => {
			if (!event.altKey || event.key === "Alt") altPressed = false;
		};

		const handleBlur = () => {
			altPressed = false;
		};

		window.addEventListener("keydown", handleKeyDown);
		window.addEventListener("keyup", handleKeyUp);
		window.addEventListener("blur", handleBlur);

		const loadSettings = async () => {
			isLoadingApiKeys = true;
			try {
				const settings = await invoke<{
					theme: string;
					engine: string;
					source_lang: string;
					target_lang: string;
					target_lang_2: string;
					overlay_transparency: number;
					overlay_font_size: number;
					translation_engine: string;
					translation_endpoint: string | null;
					translation_model: string | null;
					translation_api_key_name: string | null;
					remote_endpoint: string | null;
					remote_model: string | null;
					remote_api_key_name: string | null;
				}>("settings_get");
				selectedEngine = settings.engine as "browser" | "vosk" | "remote";
				theme = settings.theme === "day" || settings.theme === "toy" ? settings.theme : "night";
				speech.engine = settings.engine as "browser" | "vosk" | "remote";
				speech.language = settings.source_lang;
				targetLang = settings.target_lang;
				targetLang2 = settings.target_lang_2 || "none";
				remoteEndpoint = settings.remote_endpoint ?? "";
				remoteModel = settings.remote_model ?? "";
				translationEngine = settings.translation_engine;
				translationEndpoint = settings.translation_endpoint ?? "";
				translationModel = settings.translation_model ?? "";
				speech.translationEngine = settings.translation_engine;
				speech.translationEndpoint = settings.translation_endpoint ?? "";
				speech.translationModel = settings.translation_model ?? "";

				const keyName = settings.remote_api_key_name;
				if (keyName) {
					try {
						const result = await getApiKey(keyName);
						if (result) {
							apiKey = result;
							speech.apiKey = result;
						}
					} catch (err) {
						console.error("Failed to load API key from Stronghold:", err);
					}
				}

				isLoadingApiKeys = false; // Mark loading complete

				// Load translation API key in background (non-blocking)
				const translationKeyName = settings.translation_api_key_name;
				if (translationKeyName) {
					getApiKey(translationKeyName)
						.then((result) => {
							if (result) {
								translationApiKey = result;
								speech.translationApiKey = result;
						}
					})
					.catch((err) => {
						console.error("Failed to load translation API key from Stronghold:", err);
					});
				}
			} catch (err) {
				console.error("Failed to load settings:", err);
				isLoadingApiKeys = false; // Mark loading complete even on error
			}
		};
		void loadSettings();

		return () => {
			window.removeEventListener("keydown", handleKeyDown);
			window.removeEventListener("keyup", handleKeyUp);
			window.removeEventListener("blur", handleBlur);
		};
	});

	let sourceLabel = $derived(SOURCE_LANGUAGES.find((l) => l.code === speech.language)?.label ?? "Auto");
	let targetLabel1 = $derived(TARGET_LANGUAGES.find((l) => l.value === targetLang)?.label ?? "Translation 1");
	let targetLabel2 = $derived(
		targetLang2 === "none"
			? "Translation 2"
			: (TARGET_LANGUAGES.find((l) => l.value === targetLang2)?.label ?? "Translation 2")
	);
	let translationDebugUrl = $derived(
		translationEngine === "remote" ? resolveOpenAICompatibleEndpoint(translationEndpoint) : ""
	);

	function resetTranslationState() {
		translatedIds1.clear();
		translatedIds2.clear();
		inFlight1.clear();
		inFlight2.clear();
		recentLines.length = 0;
		translations1 = {};
		translations2 = {};
	}

	function enterMockMode() {
		resetTranslationState();
		isMockMode = true;
		speech.subtitles = [...MOCK_SUBTITLES];
		translations1 = { ...MOCK_TRANSLATIONS };
		translations2 = { ...MOCK_TRANSLATIONS_2 };
		speech.status = "listening";
	}

	function exitMockMode() {
		isMockMode = false;
		resetTranslationState();
		speech.subtitles = [];
		speech.errorMessage = "";
		speech.status = "idle";
	}

	function handleStart() {
		if (altPressed) {
			enterMockMode();
			return;
		}

		isMockMode = false;
		resetTranslationState();
		speech.start();
	}

	function handleSourceLanguageChange(nextLanguage: string) {
		resetTranslationState();
		speech.setLanguage(nextLanguage);
		speech.saveSetting("source_lang", nextLanguage).catch(console.error);
	}

	function handleTargetLanguageChange(nextLanguage: string) {
		resetTranslationState();
		targetLang = nextLanguage;
		speech.saveSetting("target_lang", nextLanguage).catch(console.error);
	}

	function handleTargetLanguage2Change(nextLanguage: string) {
		resetTranslationState();
		targetLang2 = nextLanguage;
		speech.saveSetting("target_lang_2", nextLanguage).catch(console.error);
	}

	function handleEngineChange(nextEngine: "browser" | "vosk" | "remote") {
		selectedEngine = nextEngine;
		speech.setEngine(nextEngine);
		speech.saveSetting("engine", nextEngine).catch(console.error);
	}

	function handleRemoteEndpointChange(nextEndpoint: string) {
		remoteEndpoint = nextEndpoint;
		speech.saveSetting("remote_endpoint", nextEndpoint).catch(console.error);
	}

	function handleRemoteModelChange(nextModel: string) {
		remoteModel = nextModel;
		speech.saveSetting("remote_model", nextModel).catch(console.error);
	}

	function handleApiKeyChange(nextApiKey: string) {
		apiKey = nextApiKey;
		speech.apiKey = nextApiKey;
		speech.saveApiKey("remote", nextApiKey).catch(console.error);
		speech.saveSetting("remote_api_key_name", "remote").catch(console.error);
	}

	function handleTranslationEngineChange(nextEngine: string) {
		translationEngine = nextEngine;
		speech.translationEngine = nextEngine;
		speech.saveSetting("translation_engine", nextEngine).catch(console.error);
	}

	function handleTranslationEndpointChange(nextEndpoint: string) {
		translationEndpoint = nextEndpoint;
		speech.translationEndpoint = nextEndpoint;
		speech.saveSetting("translation_endpoint", nextEndpoint).catch(console.error);
	}

	function handleTranslationModelChange(nextModel: string) {
		translationModel = nextModel;
		speech.translationModel = nextModel;
		speech.saveSetting("translation_model", nextModel).catch(console.error);
	}

	function handleTranslationApiKeyChange(nextApiKey: string) {
		translationApiKey = nextApiKey;
		speech.translationApiKey = nextApiKey;
		speech.saveApiKey("translation", nextApiKey).catch(console.error);
		speech.saveSetting("translation_api_key_name", "translation").catch(console.error);
	}

	$effect(() => {
		if (isMockMode) return;

		const subs = speech.subtitles;
		for (const line of subs) {
			if (line.id.startsWith("interim-")) continue;

			if (targetLang && !translatedIds1.has(line.id)) {
				translatedIds1.add(line.id);
				translateLine(line.id, line.text, targetLang, translations1, inFlight1);
			}

			if (targetLang2 !== "none" && !translatedIds2.has(line.id)) {
				translatedIds2.add(line.id);
				translateLine(line.id, line.text, targetLang2, translations2, inFlight2);
			}
		}
	});

	async function translateLine(
		id: string,
		text: string,
		targetLanguage: string,
		translationMap: Record<string, string>,
		inFlightMap: Set<string>
	) {
		if (!targetLanguage || targetLanguage === "none" || inFlightMap.has(id)) return;

		recentLines.push(text);
		if (recentLines.length > 3) recentLines.shift();

		inFlightMap.add(id);

		try {
			const result = await speech.translate(text, speech.language, targetLanguage);
			if (result) {
				translationMap[id] = result;
			}
		} catch {
			/* silent */
		} finally {
			inFlightMap.delete(id);
		}
	}
</script>

<svelte:head>
	<title>Real-time Subtitles</title>
</svelte:head>

<div
	class="w-full h-screen flex flex-col items-center justify-center relative overflow-hidden"
	data-theme={theme}
>
	{#if speech.status === "idle"}
		<IdleScreen
			{sourceLabel}
			targetLabel={targetLabel1}
			onStart={handleStart}
			onSettings={() => (settingsSplitOpen = true)}
			{isLoadingApiKeys}
			requiresRemoteApiKey={selectedEngine === "remote" && !apiKey}
		/>
	{:else if speech.status === "listening"}
		<ListeningScreen
			subtitles={speech.subtitles}
			{translations1}
			{translations2}
			{sourceLabel}
			{targetLabel1}
			{targetLabel2}
			showTranslation2={targetLang2 !== "none"}
			onStop={isMockMode ? exitMockMode : speech.stop}
			{isMockMode}
			{translationDebugUrl}
		/>
	{:else if speech.status === "error"}
		<ErrorScreen message={speech.errorMessage} onRetry={speech.start} debugUrl={translationDebugUrl} />
	{/if}

	<SettingsSplitModal
		open={settingsSplitOpen}
		activeSection={settingsSection}
		theme={theme}
		language={speech.language}
		{targetLang}
		{targetLang2}
		engine={speech.engine}
		{remoteEndpoint}
		remoteModel={remoteModel}
		{apiKey}
		{translationEngine}
		{translationEndpoint}
		{translationModel}
		translationApiKey={translationApiKey}
		onThemeChange={(nextTheme) => {
			theme = nextTheme;
			speech.saveSetting("theme", nextTheme).catch(console.error);
		}}
		onLanguageChange={handleSourceLanguageChange}
		onTargetLangChange={handleTargetLanguageChange}
		onTargetLang2Change={handleTargetLanguage2Change}
		onEngineChange={handleEngineChange}
		onRemoteEndpointChange={handleRemoteEndpointChange}
		onRemoteModelChange={handleRemoteModelChange}
		onApiKeyChange={handleApiKeyChange}
		onTranslationEngineChange={handleTranslationEngineChange}
		onTranslationEndpointChange={handleTranslationEndpointChange}
		onTranslationModelChange={handleTranslationModelChange}
		onTranslationApiKeyChange={handleTranslationApiKeyChange}
		onSectionChange={(section) => (settingsSection = section)}
		onClose={() => (settingsSplitOpen = false)}
	/>
</div>
