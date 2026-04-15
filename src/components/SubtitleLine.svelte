<script lang="ts">
  interface Props {
    text: string;
    translation1?: string;
    translation2?: string;
    isTranslating1?: boolean;
    isTranslating2?: boolean;
    isLast?: boolean;
    showTranslation2?: boolean;
  }

  let {
    text,
    translation1,
    translation2,
    isTranslating1 = false,
    isTranslating2 = false,
    isLast = false,
    showTranslation2 = true,
  }: Props = $props();
</script>

<div class="line grid gap-8 p-4" class:last={isLast} class:grid-cols-3={showTranslation2} class:grid-cols-2={!showTranslation2}>
  <div class="col flex justify-center text-center">{text}</div>

  <div class="col flex justify-center text-center">
    {#if isTranslating1}
      <div class="secondary-text">
        <span class="dot">·</span>
        <span class="dot">·</span>
        <span class="dot">·</span>
      </div>
    {/if}
    {#if translation1}
      <div class="secondary-text">{translation1}</div>
    {/if}
  </div>

  {#if showTranslation2}
    <div class="col flex justify-center text-center">
      {#if isTranslating2}
        <div class="secondary-text">
          <span class="dot">·</span>
          <span class="dot">·</span>
          <span class="dot">·</span>
        </div>
      {/if}
      {#if translation2}
        <div class="secondary-text">{translation2}</div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .line {
    color: var(--on-bg-color-strong);
    font-size: 1.8rem;
    font-weight: 500;
    animation: slideUp 0.4s ease-out forwards;
    opacity: 1;
  }

  .line.last {
    font-size: 2.4rem;
    font-weight: 600;
  }

  .secondary-text {
    color: var(--on-bg-color);
  }

  .col {
    min-width: 0;
    word-break: break-word;
  }

  .dot {
    animation: dotBounce 1.2s ease-in-out infinite;
  }

  .dot:nth-child(2) { animation-delay: 0.15s; }
  .dot:nth-child(3) { animation-delay: 0.3s; }
</style>
