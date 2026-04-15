<script lang="ts">
	type SettingsSection = "theme" | "language" | "stt" | "translate";

	interface Props {
		activeSection: SettingsSection;
		menuItems: Array<{ key: SettingsSection; label: string }>;
		onSectionChange: (section: SettingsSection) => void;
	}

	let { activeSection, menuItems, onSectionChange }: Props = $props();
</script>

<aside class="settings-sidebar w-60 overflow-y-auto p-5">
	<div class="settings-eyebrow">Settings</div>
	<nav class="flex flex-col gap-2" aria-label="Settings menu">
		{#each menuItems as item}
			<button
				type="button"
				onclick={() => onSectionChange(item.key)}
				class="settings-menu-button"
				data-selected={activeSection === item.key ? "true" : undefined}
			>
				{item.label}
			</button>
		{/each}
	</nav>
</aside>

<style>
	.settings-sidebar {
		background: var(--surface-alt-color);
		border-right: 1px solid var(--border-color-subtle);
	}

	.settings-eyebrow {
		color: var(--muted-color);
		font-size: 0.75rem;
		font-weight: 600;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		margin-bottom: 1rem;
	}

	.settings-menu-button {
		width: 100%;
		cursor: pointer;
		border-radius: 0.75rem;
		border: 1px solid transparent;
		padding: 0.625rem 0.75rem;
		text-align: left;
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--on-surface-color);
		transition:
			background-color 160ms ease,
			border-color 160ms ease,
			color 160ms ease;
	}

	.settings-menu-button:hover {
		background: var(--surface-hover-color);
		border-color: var(--border-color-default);
	}

	.settings-menu-button[data-selected="true"] {
		background: var(--surface-active-color);
		border-color: var(--border-color-strong);
		color: var(--on-surface-color-strong);
	}
</style>
