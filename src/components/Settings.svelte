<script lang="ts">
  import Dropdown from './Dropdown.svelte';
  import { SOURCE_LANGUAGES, TARGET_LANGUAGES } from '$lib/languages';

  interface Props {
    open: boolean;
    language: string;
    onLanguageChange: (lang: string) => void;
    targetLang: string;
    onTargetLangChange: (lang: string) => void;
    subtitlePosition: number;
    onSubtitlePositionChange: (pos: number) => void;
    onClose: () => void;
  }

  let {
    open,
    language,
    onLanguageChange,
    targetLang,
    onTargetLangChange,
    subtitlePosition,
    onSubtitlePositionChange,
    onClose,
  }: Props = $props();

  const sourceOptions = SOURCE_LANGUAGES.map((l) => ({ value: l.code, label: l.label }));
  const targetOptions = TARGET_LANGUAGES.map((l) => ({ value: l.value, label: l.label }));
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" class:open={open} onclick={onClose}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="panel" class:open={open} onclick={(e) => e.stopPropagation()}>
    <div class="panel-header">
      <span class="panel-title">Settings</span>
      <button class="close" onclick={onClose} aria-label="Close settings">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
    </div>

    <div class="panel-body">
      <div class="field">
        <label class="label" for="source-lang">Source Language</label>
        <Dropdown id="source-lang" value={language} options={sourceOptions} onchange={onLanguageChange} />
      </div>

      <div class="field">
        <label class="label" for="target-lang">Translate To</label>
        <Dropdown id="target-lang" value={targetLang} options={targetOptions} onchange={onTargetLangChange} />
      </div>

      <div class="field">
        <label class="label" for="subtitle-pos">Subtitle Position — {subtitlePosition}%</label>
        <input
          id="subtitle-pos"
          type="range"
          min={5}
          max={90}
          value={subtitlePosition}
          oninput={(e) => onSubtitlePositionChange(Number((e.target as HTMLInputElement).value))}
          class="slider"
        />
        <div class="slider-labels">
          <span>Bottom</span>
          <span>Top</span>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0);
    z-index: 100;
    display: flex;
    align-items: flex-end;
    justify-content: center;
    transition: background 0.3s ease;
    pointer-events: none;
  }

  .overlay.open {
    background: rgba(0, 0, 0, 0.4);
    pointer-events: auto;
  }

  .panel {
    width: 100%;
    max-width: 420px;
    background: rgba(28, 28, 30, 0.95);
    backdrop-filter: blur(40px);
    -webkit-backdrop-filter: blur(40px);
    border-radius: 20px 20px 0 0;
    padding: 0 0 40px;
    transform: translateY(100%);
    transition: transform 0.35s cubic-bezier(0.32, 0.72, 0, 1);
  }

  .panel.open {
    transform: translateY(0);
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px 24px 12px;
  }

  .panel-title {
    font-size: 1.05rem;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.9);
  }

  .close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.08);
    border: none;
    color: rgba(255, 255, 255, 0.5);
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .close:hover {
    background: rgba(255, 255, 255, 0.15);
    color: #fff;
  }

  .panel-body {
    display: flex;
    flex-direction: column;
    gap: 24px;
    padding: 8px 24px 0;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .label {
    font-size: 0.75rem;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.4);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .slider {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    height: 4px;
    border-radius: 2px;
    background: rgba(255, 255, 255, 0.1);
    outline: none;
    cursor: pointer;
  }

  .slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: #fff;
    cursor: pointer;
    box-shadow: 0 0 6px rgba(0, 0, 0, 0.3);
  }

  .slider-labels {
    display: flex;
    justify-content: space-between;
    font-size: 0.65rem;
    color: rgba(255, 255, 255, 0.2);
  }
</style>
