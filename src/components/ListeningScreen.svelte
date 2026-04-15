<script lang="ts">
  import SubtitleLine from './SubtitleLine.svelte';
  import type { SubtitleLine as SubtitleLineType } from '$lib/types';

  interface Props {
    subtitles: SubtitleLineType[];
    translations1: Record<string, string>;
    translations2: Record<string, string>;
    sourceLabel: string;
    targetLabel1: string;
    targetLabel2: string;
    translationDebugUrl?: string;
    onStop: () => void;
    isMockMode?: boolean;
  }

  let {
    subtitles,
    translations1,
    translations2,
    sourceLabel,
    targetLabel1,
    targetLabel2,
    translationDebugUrl = '',
    onStop,
    isMockMode = false,
  }: Props = $props();
</script>

<svelte:head>
  <title>Listening… · Real-time Subtitles</title>
</svelte:head>

<div class="status-bar backdrop-blur-md">
  <div class="status-stack">
    <div class="status-indicator">
      <span class="status-dot"></span>
      {isMockMode ? 'Mock Preview' : 'Listening…'}
      <span class="lang-badge">{sourceLabel} | {targetLabel1} | {targetLabel2}</span>
    </div>
    {#if translationDebugUrl}
      <div class="debug-url">Translate endpoint: {translationDebugUrl}</div>
    {/if}
  </div>
  <button class="stop" onclick={onStop}>Stop</button>
</div>

<div class="fixed top-18 bottom-0 left-0 right-0 overflow-y-auto flex flex-col justify-end">
  <div class="flex flex-col gap-2">
    {#each subtitles as line, i}
      <SubtitleLine
        text={line.text}
        translation1={translations1[line.id]}
        translation2={translations2[line.id]}
        isTranslating1={!line.id.startsWith('interim-') && !translations1[line.id]}
        isTranslating2={!line.id.startsWith('interim-') && !translations2[line.id]}
        isLast={i === subtitles.length - 1}
      />
    {/each}

    <div class="h-32 shrink-0" aria-hidden="true"></div>
  </div>
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
    background: linear-gradient(to bottom, color-mix(in srgb, var(--bg-color) 74%, transparent) 0%, transparent 100%);
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 0.8rem;
    color: var(--on-bg-color);
  }

  .status-stack {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }

  .status-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--success-color);
    box-shadow: 0 0 8px var(--success-color-soft);
    animation: pulseDot 2s ease-in-out infinite;
  }

  .lang-badge {
    font-size: 0.7rem;
    font-weight: 600;
    padding: 3px 8px;
    border-radius: 4px;
    background: var(--surface-hover-color);
    color: var(--on-surface-color-strong);
    border: 1px solid var(--border-color-default);
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .debug-url {
    max-width: min(72vw, 920px);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 0.68rem;
    color: var(--on-surface-color);
    opacity: 0.82;
  }

  .stop {
    padding: 8px 20px;
    font-size: 0.85rem;
    font-weight: 500;
    font-family: inherit;
    color: var(--on-surface-color-strong);
    background: var(--field-color);
    border: 1px solid var(--border-color-default);
    border-radius: 20px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .stop:hover {
    color: var(--on-surface-color-strong);
    background: var(--surface-hover-color);
    border-color: var(--border-color-strong);
  }
</style>
