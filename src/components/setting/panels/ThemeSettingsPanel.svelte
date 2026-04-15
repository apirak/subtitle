<script lang="ts">
	type ThemeMode = "night" | "day" | "toy";

	interface Props {
		currentTheme: ThemeMode;
		onThemeChange: (theme: ThemeMode) => void;
	}

	let { currentTheme, onThemeChange }: Props = $props();
</script>

<div class="mx-auto w-full max-w-140">
	<div class="settings-section-label">Choose Theme</div>
	<div class="grid grid-cols-3 gap-3">
		<button
			type="button"
			onclick={() => onThemeChange("night")}
			class="theme-card"
			data-selected={currentTheme === "night" ? "true" : undefined}
		>
			<div class="theme-card-title">Night</div>
			<div class="theme-card-description">Dark and focused</div>
		</button>

		<button
			type="button"
			onclick={() => onThemeChange("day")}
			class="theme-card"
			data-selected={currentTheme === "day" ? "true" : undefined}
		>
			<div class="theme-card-title">Day</div>
			<div class="theme-card-description">Light and clear</div>
		</button>

		<button
			type="button"
			onclick={() => onThemeChange("toy")}
			class="theme-card"
			data-selected={currentTheme === "toy" ? "true" : undefined}
		>
			<div class="theme-card-title">Toy</div>
			<div class="theme-card-description">Warm and playful</div>
		</button>
	</div>
</div>

<style>
	.settings-section-label {
		color: var(--muted-color);
		font-size: 0.75rem;
		font-weight: 600;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		margin-bottom: 1rem;
	}

	.theme-card {
		cursor: pointer;
		border-radius: 1rem;
		padding: 1rem;
		text-align: left;
		border: 1px solid var(--border-color-default);
		background: var(--field-color);
		color: var(--on-surface-color-strong);
		transition:
			background-color 160ms ease,
			border-color 160ms ease,
			color 160ms ease,
			box-shadow 160ms ease;
	}

	.theme-card:hover {
		background: var(--field-hover-color);
		border-color: var(--border-color-strong);
	}

	.theme-card[data-selected="true"] {
		background: var(--accent-color);
		border-color: var(--accent-color);
		color: var(--on-accent-color);
	}

	.theme-card-title {
		font-size: 0.875rem;
		font-weight: 600;
	}

	.theme-card-description {
		margin-top: 0.25rem;
		font-size: 0.75rem;
		line-height: 1.45;
		color: var(--on-surface-color);
	}

	.theme-card[data-selected="true"] .theme-card-title,
	.theme-card[data-selected="true"] .theme-card-description {
		color: var(--on-accent-color);
	}
</style>
