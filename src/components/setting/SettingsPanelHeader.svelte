<script lang="ts">
	interface Props {
		title: string;
		onClose: () => void;
	}

	let { title, onClose }: Props = $props();
</script>

<div class="flex items-center justify-between mb-6">
	<h2 class="settings-title">{title}</h2>
	<button type="button" class="settings-close" onclick={onClose} aria-label="Close settings">
		×
	</button>
</div>

<style>
	.settings-title {
		color: var(--on-surface-color-strong);
		font-size: 1.125rem;
		font-weight: 600;
	}

	.settings-close {
		display: grid;
		height: 2rem;
		width: 2rem;
		place-items: center;
		cursor: pointer;
		border-radius: 9999px;
		border: 1px solid var(--border-color-default);
		background: transparent;
		color: var(--on-surface-color);
		transition:
			background-color 160ms ease,
			border-color 160ms ease,
			color 160ms ease;
	}

	.settings-close:hover {
		background: var(--surface-hover-color);
		border-color: var(--border-color-strong);
		color: var(--on-surface-color-strong);
	}
</style>
