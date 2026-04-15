<script lang="ts">
	interface Option {
		value: string;
		label: string;
	}

	interface Props {
		sourceOptions: Option[];
		targetOptions: Option[];
		targetOptionsWithNone: Option[];
		currentLanguage: string;
		currentTargetLang: string;
		currentTargetLang2: string;
		onLanguageChange: (language: string) => void;
		onTargetLangChange: (language: string) => void;
		onTargetLang2Change: (language: string) => void;
	}

	let {
		sourceOptions,
		targetOptions,
		targetOptionsWithNone,
		currentLanguage,
		currentTargetLang,
		currentTargetLang2,
		onLanguageChange,
		onTargetLangChange,
		onTargetLang2Change,
	}: Props = $props();
</script>

<div class="mx-auto w-full max-w-140">
	<div class="settings-section-label">Speech And Translation</div>

	<div class="mb-5">
		<label class="settings-field-label" for="split-source-language">Source Column</label>
		<select
			id="split-source-language"
			class="settings-select"
			value={currentLanguage}
			onchange={(event) => onLanguageChange((event.target as HTMLSelectElement).value)}
		>
			{#each sourceOptions as option}
				<option value={option.value}>{option.label}</option>
			{/each}
		</select>
	</div>

	<div class="mb-5">
		<label class="settings-field-label" for="split-target-language">Translate Column 1</label>
		<select
			id="split-target-language"
			class="settings-select"
			value={currentTargetLang}
			onchange={(event) => onTargetLangChange((event.target as HTMLSelectElement).value)}
		>
			{#each targetOptions as option}
				<option value={option.value}>{option.label}</option>
			{/each}
		</select>
	</div>

	<div>
		<label class="settings-field-label" for="split-target-language-2">Translate Column 2</label>
		<select
			id="split-target-language-2"
			class="settings-select"
			value={currentTargetLang2}
			onchange={(event) => onTargetLang2Change((event.target as HTMLSelectElement).value)}
		>
			{#each targetOptionsWithNone as option}
				<option value={option.value}>{option.label}</option>
			{/each}
		</select>
	</div>

	<p class="settings-helper-text">
		Source language is column 1. Translation 1 and Translation 2 are columns 2 and 3.
	</p>
</div>

<style>
	.settings-section-label,
	.settings-field-label {
		color: var(--muted-color);
		font-size: 0.75rem;
		font-weight: 600;
		letter-spacing: 0.08em;
		text-transform: uppercase;
	}

	.settings-section-label {
		margin-bottom: 1rem;
	}

	.settings-field-label {
		display: block;
		margin-bottom: 0.5rem;
	}

	.settings-select {
		width: 100%;
		border-radius: 1rem;
		padding: 0.875rem 1rem;
		outline: none;
		border: 1px solid var(--border-color-default);
		background: var(--field-color);
		color: var(--on-surface-color-strong);
		transition:
			background-color 160ms ease,
			border-color 160ms ease,
			color 160ms ease,
			box-shadow 160ms ease;
	}

	.settings-select:hover {
		background: var(--field-hover-color);
		border-color: var(--border-color-strong);
	}

	.settings-select:focus {
		border-color: var(--accent-color);
		box-shadow: 0 0 0 3px var(--focus-ring-color);
	}

	.settings-helper-text {
		margin-top: 1.25rem;
		font-size: 0.875rem;
		line-height: 1.5;
		color: var(--on-surface-color);
	}
</style>
