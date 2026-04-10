<script lang="ts">
  import { speech } from './lib/speech.svelte';
  import { SOURCE_LANGUAGES, TARGET_LANGUAGES, getLangName } from './lib/languages';
  import IdleScreen from './components/IdleScreen.svelte';
  import ListeningScreen from './components/ListeningScreen.svelte';
  import ErrorScreen from './components/ErrorScreen.svelte';
  import Settings from './components/Settings.svelte';
  import './app.css';

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

    const apiKey = import.meta.env.VITE_DEEPINFRA_API_KEY;
    if (!apiKey) return;

    const langName = getLangName(targetLang);
    const context = recentLines.slice(-2);

    let prompt: string;
    if (context.length > 0) {
      prompt = `/no_think
Translate the last sentence to ${langName}. The previous sentences are for context only — do not translate them.

Context:
${context.map((c, i) => `${i + 1}. ${c}`).join('\n')}

Translate this:
${text}

Return only the translation.`;
    } else {
      prompt = `/no_think\nTranslate to ${langName}. Return only the translation.\n\n${text}`;
    }

    recentLines.push(text);
    if (recentLines.length > 3) recentLines.shift();

    inFlight.add(id);

    try {
      const res = await fetch('https://api.deepinfra.com/v1/openai/chat/completions', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${apiKey}`,
        },
        body: JSON.stringify({
          model: 'Qwen/Qwen3-14B',
          messages: [{ role: 'user', content: prompt }],
        }),
      });

      if (!res.ok) return;

      const data = await res.json();
      const msg = data.choices?.[0]?.message;
      const raw = String(msg?.content || msg?.reasoning_content || '').trim();
      const result = raw.replace(/<think[\s\S]*?<\/think>/g, '').trim();

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

<div class="container">
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

<style>
  .container {
    width: 100%;
    height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    position: relative;
    overflow: hidden;
  }
</style>
