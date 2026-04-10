<script lang="ts">
  interface Props {
    message: string;
    onRetry: () => void;
  }

  let { message, onRetry }: Props = $props();
</script>

<div class="screen">
  <span class="error-text">Something went wrong. Please check microphone permissions.</span>
  {#if message}
    <span class="error-detail">{message}</span>
  {/if}
  <button class="retry" onclick={onRetry}>
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
      <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" />
      <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
      <line x1="12" y1="19" x2="12" y2="22" />
    </svg>
    Try Again
  </button>
</div>

<style>
  .screen {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
  }

  .error-text {
    font-size: 1rem;
    color: #ef4444;
  }

  .error-detail {
    font-size: 0.8rem;
    color: rgba(255, 255, 255, 0.4);
    max-width: 500px;
    text-align: center;
    word-break: break-word;
  }

  .retry {
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

  .retry:hover {
    background: rgba(255, 255, 255, 0.15);
    border-color: rgba(255, 255, 255, 0.25);
  }
</style>
