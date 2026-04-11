<script lang="ts">
  import { onDestroy } from 'svelte';
  import { speech } from './lib/speech.svelte';
  import { SOURCE_LANGUAGES, TARGET_LANGUAGES } from './lib/languages';
  import IdleScreen from './components/IdleScreen.svelte';
  import ListeningScreen from './components/ListeningScreen.svelte';
  import ErrorScreen from './components/ErrorScreen.svelte';
  import Settings from './components/Settings.svelte';
  import './app.css';

  onDestroy(() => {
    speech.destroy();
  });

  let targetLang = $state('th');
  let translations = $state<Record<string, string>>({});
  let settingsOpen = $state(false);
  let subtitlePosition = $state(20);

  const translatedIds = new Set<string>();
  const inFlight = new Set<string>();
  const recentLines: string[] = [];

  let sourceLabel = $derived(
    SOURCE_LANGUAGES.find((l) => l.code === speech.language)?.label ?? 'Auto'
  );
  let targetLabel = $derived(
    TARGET_LANGUAGES.find((l) => l.value === targetLang)?.label ?? 'English'
  );

  $effect(() => {
    const subs = speech.subtitles;
    for (const line of subs) {
      if (line.id.startsWith('interim-')) continue;
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
  {#if speech.status === 'idle'}
    <IdleScreen {sourceLabel} {targetLabel} onStart={speech.start} onSettings={() => settingsOpen = true} />
  {:else if speech.status === 'listening'}
    <ListeningScreen
      subtitles={speech.subtitles}
      {translations}
      {sourceLabel}
      {targetLabel}
      {subtitlePosition}
      onStop={speech.stop}
    />
  {:else if speech.status === 'error'}
    <ErrorScreen message={speech.errorMessage} onRetry={speech.start} />
  {/if}

  <Settings
    open={settingsOpen}
    language={speech.language}
    onLanguageChange={speech.setLanguage}
    {targetLang}
    onTargetLangChange={(v) => targetLang = v}
    {subtitlePosition}
    onSubtitlePositionChange={(v) => subtitlePosition = v}
    onClose={() => settingsOpen = false}
  />
</div>


