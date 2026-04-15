<script lang="ts">
  interface Props {
    sourceLabel: string;
    targetLabel: string;
    onStart: () => void;
    onSettings: () => void;
  }

  let { sourceLabel, targetLabel, onStart, onSettings }: Props = $props();
</script>

<div class="screen-shell flex flex-col items-center gap-8">
  <div class="flex flex-col items-center gap-2">
    <span class="screen-title text-2xl font-bold tracking-tight">Real-time Subtitles</span>
    <span class="screen-subtitle text-sm">{sourceLabel} → {targetLabel}</span>
  </div>

  <button
    class="start-button flex items-center justify-center gap-2 text-base font-semibold rounded-[60px] cursor-pointer transition-all duration-250 ease backdrop-blur-md hover:scale-[1.03] active:scale-[0.97]"
    style="padding: 14px 32px;"
    onclick={onStart}
  >
    <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
      <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" />
      <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
      <line x1="12" y1="19" x2="12" y2="22" />
    </svg>
    Start
  </button>

  <button
    class="settings-button flex items-center justify-center w-8 h-8 rounded-full bg-transparent border-0 cursor-pointer transition-all duration-200 ease hover:rotate-30"
    onclick={onSettings}
    aria-label="Settings"
  >
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1Z" />
    </svg>
  </button>
</div>

<style>
  .screen-shell {
    color: var(--on-bg-color-strong);
  }

  .screen-title {
    color: var(--on-bg-color-strong);
  }

  .screen-subtitle {
    color: var(--on-bg-color);
  }

  .start-button {
    color: var(--on-accent-color);
    background: var(--accent-color);
    border: 1px solid var(--accent-color);
    box-shadow: 0 12px 32px color-mix(in srgb, var(--accent-color) 28%, transparent);
  }

  .start-button:hover {
    background: var(--accent-color-hover);
    border-color: var(--accent-color-hover);
  }

  .settings-button {
    color: var(--muted-color);
  }

  .settings-button:hover {
    color: var(--on-bg-color-strong);
  }
</style>
