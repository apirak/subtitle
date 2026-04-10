<script lang="ts">
  interface Props {
    text: string;
    translation?: string;
    isTranslating?: boolean;
    isLast?: boolean;
  }

  let { text, translation, isTranslating = false, isLast = false }: Props = $props();
</script>

<div class="line" class:last={isLast}>
  <span class="text">{text}</span>
  {#if isTranslating}
    <span class="translating">
      <span class="dot">·</span>
      <span class="dot">·</span>
      <span class="dot">·</span>
    </span>
  {/if}
  {#if translation}
    <span class="translation">{translation}</span>
  {/if}
</div>

<style>
  .line {
    font-size: 1.8rem;
    font-weight: 500;
    color: #fff;
    text-align: center;
    text-shadow: 0 2px 20px rgba(0, 0, 0, 0.8);
    max-width: min(90%, 42rem);
    line-height: 1.4;
    animation: slideUp 0.4s ease-out forwards;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    opacity: 0.25;
  }

  .line.last {
    opacity: 1;
    font-size: 3rem;
    font-weight: 600;
  }

  .translation {
    font-size: 0.6em;
    font-weight: 400;
    color: rgba(255, 255, 255, 0.6);
    text-shadow: 0 1px 10px rgba(0, 0, 0, 0.6);
  }

  .translating {
    display: inline-flex;
    gap: 2px;
    font-size: 0.6em;
    font-weight: 400;
    color: rgba(255, 255, 255, 0.4);
  }

  .dot {
    animation: dotBounce 1.2s ease-in-out infinite;
  }

  .dot:nth-child(2) { animation-delay: 0.15s; }
  .dot:nth-child(3) { animation-delay: 0.3s; }
</style>
