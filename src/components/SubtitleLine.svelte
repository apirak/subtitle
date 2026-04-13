<script lang="ts">
  interface Props {
    text: string;
    translation?: string;
    isTranslating?: boolean;
    isLast?: boolean;
  }

  let { text, translation, isTranslating = false, isLast = false }: Props = $props();
</script>

<div class="line flex text-3xl text-white gap-16 p-4" class:last={isLast}>
  <div class="flex-1 flex justify-center text-center">{text}</div>
  <div class="flex-1 flex justify-center text-center">
    {#if isTranslating}
      <div class="text-white/80">
        <span class="dot">·</span>
        <span class="dot">·</span>
        <span class="dot">·</span>
      </div>
    {/if}
    {#if translation}
      <div class="text-white/80">{translation}</div>
    {/if}
  </div>
</div>

<style>
  .line {
    font-size: 1.8rem;
    font-weight: 500;
    animation: slideUp 0.4s ease-out forwards;
  }

  .line.last {
    opacity: 1;
    font-size: 3rem;
    font-weight: 600;
  }

  .dot {
    animation: dotBounce 1.2s ease-in-out infinite;
  }

  .dot:nth-child(2) { animation-delay: 0.15s; }
  .dot:nth-child(3) { animation-delay: 0.3s; }
</style>
