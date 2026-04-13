<script lang="ts">
  import SubtitleLine from './SubtitleLine.svelte';
  import type { SubtitleLine as SubtitleLineType } from '$lib/types';

  interface Props {
    subtitles: SubtitleLineType[];
    translations: Record<string, string>;
    sourceLabel: string;
    targetLabel: string;
    subtitlePosition: number;
    onStop: () => void;
    isMockMode?: boolean;
  }

  let { subtitles, translations, sourceLabel, targetLabel, subtitlePosition, onStop, isMockMode = false }: Props = $props();
</script>

<svelte:head>
  <title>Listening… · Real-time Subtitles</title>
</svelte:head>

<div class="status-bar">
  <div class="status-indicator">
    <span class="status-dot"></span>
    {isMockMode ? 'Mock Preview' : 'Listening…'}
    <span class="lang-badge">{sourceLabel} → {targetLabel}</span>
  </div>
  <button class="stop" onclick={onStop}>Stop</button>
</div>

<div class="subtitle-container" style="bottom: {subtitlePosition}%">
  {#each subtitles as line, i}
    <SubtitleLine
      text={line.text}
      translation={translations[line.id]}
      isTranslating={!line.id.startsWith('interim-') && !translations[line.id]}
      isLast={i === subtitles.length - 1}
    />
  {/each}
</div>

<style>
  .status-bar {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 24px;
    z-index: 20;
    background: linear-gradient(to bottom, rgba(0, 0, 0, 0.5) 0%, transparent 100%);
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 0.8rem;
    color: rgba(255, 255, 255, 0.5);
  }

  .status-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: #4ade80;
    box-shadow: 0 0 8px rgba(74, 222, 128, 0.5);
    animation: pulseDot 2s ease-in-out infinite;
  }

  .lang-badge {
    font-size: 0.7rem;
    font-weight: 600;
    padding: 3px 8px;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.08);
    color: rgba(255, 255, 255, 0.45);
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .stop {
    padding: 8px 20px;
    font-size: 0.85rem;
    font-weight: 500;
    font-family: inherit;
    color: rgba(255, 255, 255, 0.7);
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 20px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .stop:hover {
    color: #fff;
    background: rgba(239, 68, 68, 0.15);
    border-color: rgba(239, 68, 68, 0.35);
  }

  .subtitle-container {
    position: fixed;
    left: 0;
    right: 0;
    z-index: 10;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 0 24px;
    pointer-events: auto;
  }
</style>
