<script lang="ts">
  interface Props {
    sourceLabel: string;
    targetLabel: string;
    onStart: () => void;
    onSettings: () => void;
  }

  let { sourceLabel, targetLabel, onStart, onSettings }: Props = $props();
</script>

<div class="screen">
  <div class="title-group">
    <span class="title">Real-time Subtitles</span>
    <span class="config">{sourceLabel} → {targetLabel}</span>
  </div>

  <button class="start" onclick={onStart}>
    <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
      <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" />
      <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
      <line x1="12" y1="19" x2="12" y2="22" />
    </svg>
    Start
  </button>

  <button class="settings" onclick={onSettings} aria-label="Settings">
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1Z" />
    </svg>
  </button>
</div>

<style>
  .screen {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 32px;
  }

  .title-group {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }

  .title {
    font-size: 1.5rem;
    font-weight: 700;
    color: rgba(255, 255, 255, 0.9);
    letter-spacing: -0.02em;
  }

  .config {
    font-size: 0.85rem;
    color: rgba(255, 255, 255, 0.35);
  }

  .start {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 14px 32px;
    font-size: 1rem;
    font-weight: 600;
    font-family: inherit;
    color: #fff;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 60px;
    cursor: pointer;
    transition: all 0.25s ease;
    backdrop-filter: blur(10px);
  }

  .start:hover {
    background: rgba(255, 255, 255, 0.15);
    border-color: rgba(255, 255, 255, 0.25);
    transform: scale(1.03);
    box-shadow: 0 0 30px rgba(255, 255, 255, 0.06);
  }

  .start:active {
    transform: scale(0.97);
  }

  .settings {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: none;
    border: none;
    color: rgba(255, 255, 255, 0.2);
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .settings:hover {
    color: rgba(255, 255, 255, 0.45);
    transform: rotate(30deg);
  }
</style>
