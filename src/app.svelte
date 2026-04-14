<script lang="ts">
	import { onDestroy, onMount } from "svelte";
	import { invoke } from "@tauri-apps/api/core";
	import { speech } from "./lib/speech.svelte";
	import { SOURCE_LANGUAGES, TARGET_LANGUAGES } from "./lib/languages";
	import { MOCK_SUBTITLES, MOCK_TRANSLATIONS } from "./lib/mockData";
	import { getApiKey } from "./lib/stronghold";
	import IdleScreen from "./components/IdleScreen.svelte";
	import ListeningScreen from "./components/ListeningScreen.svelte";
	import ErrorScreen from "./components/ErrorScreen.svelte";
	import Settings from "./components/Settings.svelte";
	import "./app.css";

	onDestroy(() => {
		speech.destroy();
	});

	let targetLang = $state("th");
	let translations = $state<Record<string, string>>({});
	let settingsOpen = $state(false);
	let selectedEngine = $state<"browser" | "vosk" | "remote">("browser");
	let remoteEndpoint = $state("");
	let modelPath = $state("");
	let apiKey = $state("");
	let overlayTransparency = $state(80);
	let fontSize = $state(24);
	let translationEngine = $state("none");
	let isMockMode = $state(false);
	let altPressed = $state(false);

	const translatedIds = new Set<string>();
	const inFlight = new Set<string>();
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
			try {
				const settings = await invoke<{
					engine: string;
					source_lang: string;
					target_lang: string;
					overlay_transparency: number;
					overlay_font_size: number;
					translation_engine: string;
					remote_endpoint: string | null;
					remote_api_key_name: string | null;
				}>("settings_get");
				selectedEngine = settings.engine as "browser" | "vosk" | "remote";
				speech.engine = settings.engine as "browser" | "vosk" | "remote";
				speech.language = settings.source_lang;
				targetLang = settings.target_lang;
				remoteEndpoint = settings.remote_endpoint ?? "";
				overlayTransparency = Math.round(settings.overlay_transparency * 100);
				fontSize = settings.overlay_font_size;
				translationEngine = settings.translation_engine;

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
			} catch (err) {
				console.error("Failed to load settings:", err);
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
	let targetLabel = $derived(TARGET_LANGUAGES.find((l) => l.value === targetLang)?.label ?? "English");

	function resetTranslationState() {
		translatedIds.clear();
		inFlight.clear();
		recentLines.length = 0;
		translations = {};
	}

	function enterMockMode() {
		resetTranslationState();
		isMockMode = true;
		speech.subtitles = [...MOCK_SUBTITLES];
		translations = { ...MOCK_TRANSLATIONS };
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

	$effect(() => {
		if (isMockMode) return;

		const subs = speech.subtitles;
		for (const line of subs) {
			if (line.id.startsWith("interim-")) continue;
			if (translatedIds.has(line.id)) continue;
			translatedIds.add(line.id);
			if (targetLang) translateLine(line.id, line.text);
		}
	});

	async function translateLine(id: string, text: string) {
		if (!targetLang || inFlight.has(id)) return;

		recentLines.push(text);
		if (recentLines.length > 3) recentLines.shift();

		inFlight.add(id);

		try {
			const result = await speech.translate(text, speech.language, targetLang);
			if (result) {
				translations[id] = result;
			}
		} catch {
			/* silent */
		} finally {
			inFlight.delete(id);
		}
	}
</script>

<svelte:head>
	<title>Real-time Subtitles</title>
</svelte:head>

<div class="w-full h-screen flex flex-col items-center justify-center relative overflow-hidden">
	{#if speech.status === "idle"}
		<IdleScreen {sourceLabel} {targetLabel} onStart={handleStart} onSettings={() => (settingsOpen = true)} />
	{:else if speech.status === "listening"}
		<ListeningScreen
			subtitles={speech.subtitles}
			{translations}
			{sourceLabel}
			{targetLabel}
			onStop={isMockMode ? exitMockMode : speech.stop}
			{isMockMode}
		/>
	{:else if speech.status === "error"}
		<ErrorScreen message={speech.errorMessage} onRetry={speech.start} />
	{/if}

	<Settings
		open={settingsOpen}
		language={speech.language}
		onLanguageChange={speech.setLanguage}
		{targetLang}
		onTargetLangChange={(v) => (targetLang = v)}
		onClose={() => (settingsOpen = false)}
		{overlayTransparency}
		onOverlayTransparencyChange={(v) => {
			overlayTransparency = v;
			speech.saveSetting?.("overlay_transparency", v).catch(console.error);
		}}
		{fontSize}
		onFontSizeChange={(v) => {
			fontSize = v;
			speech.saveSetting?.("font_size", v).catch(console.error);
		}}
		{translationEngine}
		onTranslationEngineChange={(v) => {
			translationEngine = v;
			speech.saveSetting?.("translation_engine", v).catch(console.error);
		}}
		engine={speech.engine}
		onEngineChange={(v) => {
			speech.setEngine(v as "browser" | "vosk" | "remote");
			speech.saveSetting("engine", v).catch(console.error);
		}}
		{remoteEndpoint}
		onRemoteEndpointChange={(v) => {
			remoteEndpoint = v;
			speech.saveSetting("remote_endpoint", v).catch(console.error);
		}}
		{apiKey}
		onApiKeyChange={(v) => {
			apiKey = v;
			speech.apiKey = v;
			speech.saveApiKey("remote", v).catch(console.error);
			speech.saveSetting("remote_api_key_name", "remote").catch(console.error);
		}}
	/>
</div>
