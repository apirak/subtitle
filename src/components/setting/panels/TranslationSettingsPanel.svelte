<script lang="ts">
	interface TranslationEngineOption {
		value: string;
		label: string;
	}

	interface Props {
		translationEngineOptions: TranslationEngineOption[];
		currentTranslationEngine: string;
		currentTranslationModel: string;
		currentTranslationEndpoint: string;
		currentTranslationApiKey: string;
		translationApiKeyHint: string;
		isTestingTranslate: boolean;
		testTranslateResolvedUrl: string;
		testTranslateSourceText: string;
		testTranslateResult: string;
		testTranslateError: string;
		onTranslationEngineChange: (value: string) => void;
		onTranslationModelChange: (value: string) => void;
		onTranslationEndpointChange: (value: string) => void;
		onTranslationApiKeyChange: (value: string) => void;
		onTestTranslate: () => void;
	}

	let {
		translationEngineOptions,
		currentTranslationEngine,
		currentTranslationModel,
		currentTranslationEndpoint,
		currentTranslationApiKey,
		translationApiKeyHint,
		isTestingTranslate,
		testTranslateResolvedUrl,
		testTranslateSourceText,
		testTranslateResult,
		testTranslateError,
		onTranslationEngineChange,
		onTranslationModelChange,
		onTranslationEndpointChange,
		onTranslationApiKeyChange,
		onTestTranslate,
	}: Props = $props();
</script>

<div class="mx-auto w-full max-w-140">
	<div class="settings-section-label">Translation</div>

	<div class="mb-5">
		<div class="settings-field-label">Translation Engine</div>
		<div class="settings-radio-group" role="radiogroup" aria-label="Translation Engine">
			{#each translationEngineOptions as option}
				<label
					class="settings-radio-option"
					data-selected={currentTranslationEngine === option.value ? "true" : undefined}
				>
					<input
						type="radio"
						name="split-translation-engine"
						class="settings-radio-input"
						checked={currentTranslationEngine === option.value}
						onchange={() => onTranslationEngineChange(option.value)}
					/>
					<div>
						<div class="settings-radio-title">{option.label}</div>
						<div class="settings-radio-description">
							{#if option.value === "none"}
								Disable translated columns and show only source text.
							{:else if option.value === "remote"}
								Use an OpenAI-compatible chat completion endpoint.
							{/if}
						</div>
					</div>
				</label>
			{/each}
		</div>
	</div>

	{#if currentTranslationEngine !== "none"}
		<div class="mb-5">
			<label class="settings-field-label" for="split-translation-model">Model</label>
			<input
				id="split-translation-model"
				type="text"
				class="settings-input"
				placeholder="Qwen3-32B"
				value={currentTranslationModel}
				oninput={(event) => onTranslationModelChange((event.target as HTMLInputElement).value)}
			/>
		</div>

		<div class="mb-5">
			<label class="settings-field-label" for="split-translation-endpoint">Endpoint</label>
			<input
				id="split-translation-endpoint"
				type="url"
				class="settings-input"
				placeholder="https://api.deepinfra.com/v1/openai"
				value={currentTranslationEndpoint}
				oninput={(event) => onTranslationEndpointChange((event.target as HTMLInputElement).value)}
			/>
		</div>

		<div class="mb-5">
			<label class="settings-field-label" for="split-translation-api-key">API Key</label>
			<input
				id="split-translation-api-key"
				type="password"
				class="settings-input"
				placeholder="sk-..."
				value={currentTranslationApiKey}
				onchange={(event) => onTranslationApiKeyChange((event.target as HTMLInputElement).value)}
			/>
			{#if translationApiKeyHint}
				<div class="settings-key-hint">{translationApiKeyHint}</div>
			{/if}
		</div>

		<div class="mt-5">
			<button type="button" class="settings-test-button" onclick={onTestTranslate} disabled={isTestingTranslate}>
				{#if isTestingTranslate}Testing…{:else}Test Translate{/if}
			</button>

			{#if testTranslateResolvedUrl}
				<div class="settings-test-meta">Resolved endpoint: {testTranslateResolvedUrl}</div>
			{/if}

			{#if testTranslateSourceText}
				<div class="settings-test-block">
					<div class="settings-test-label">Source sample</div>
					<div class="settings-test-value">{testTranslateSourceText}</div>
				</div>
			{/if}

			{#if testTranslateResult}
				<div class="settings-test-block">
					<div class="settings-test-label">Translated result</div>
					<div class="settings-test-value">{testTranslateResult}</div>
				</div>
			{/if}

			{#if testTranslateError}
				<div class="settings-test-error">{testTranslateError}</div>
			{/if}
		</div>
	{/if}

	<p class="settings-helper-text">
		Configure the translator used for both Translation 1 and Translation 2 columns.
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

	.settings-input,
	.settings-radio-option {
		border: 1px solid var(--border-color-default);
		background: var(--field-color);
		color: var(--on-surface-color-strong);
		transition:
			background-color 160ms ease,
			border-color 160ms ease,
			color 160ms ease,
			box-shadow 160ms ease;
	}

	.settings-input {
		width: 100%;
		border-radius: 1rem;
		padding: 0.875rem 1rem;
		outline: none;
	}

	.settings-input:hover,
	.settings-radio-option:hover {
		background: var(--field-hover-color);
		border-color: var(--border-color-strong);
	}

	.settings-input:focus {
		border-color: var(--accent-color);
		box-shadow: 0 0 0 3px var(--focus-ring-color);
	}

	.settings-input::placeholder {
		color: var(--placeholder-color);
	}

	.settings-radio-group {
		display: grid;
		gap: 0.75rem;
	}

	.settings-radio-option {
		display: flex;
		align-items: flex-start;
		gap: 0.75rem;
		border-radius: 1rem;
		padding: 0.875rem 1rem;
	}

	.settings-radio-option[data-selected="true"] {
		background: var(--accent-color);
		border-color: var(--accent-color);
		color: var(--on-accent-color);
	}

	.settings-radio-input {
		margin-top: 0.125rem;
		height: 1rem;
		width: 1rem;
		accent-color: var(--accent-color);
	}

	.settings-radio-option[data-selected="true"] .settings-radio-input {
		accent-color: var(--on-accent-color);
	}

	.settings-radio-title {
		font-size: 0.875rem;
		font-weight: 600;
	}

	.settings-radio-description {
		margin-top: 0.25rem;
		font-size: 0.75rem;
		line-height: 1.45;
		color: var(--on-surface-color);
	}

	.settings-radio-option[data-selected="true"] .settings-radio-title,
	.settings-radio-option[data-selected="true"] .settings-radio-description {
		color: var(--on-accent-color);
	}

	.settings-test-button {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		padding: 0.7rem 1rem;
		border-radius: 0.75rem;
		border: 1px solid var(--border-color-strong);
		background: var(--surface-hover-color);
		color: var(--on-surface-color-strong);
		font-weight: 600;
		cursor: pointer;
		transition:
			background-color 160ms ease,
			border-color 160ms ease,
			opacity 160ms ease;
	}

	.settings-test-button:hover:not(:disabled) {
		background: var(--field-color);
		border-color: var(--on-surface-color);
	}

	.settings-test-button:disabled {
		opacity: 0.7;
		cursor: progress;
	}

	.settings-test-meta,
	.settings-test-error,
	.settings-test-value {
		margin-top: 0.75rem;
		word-break: break-word;
	}

	.settings-test-meta {
		font-size: 0.74rem;
		color: var(--muted-color);
	}

	.settings-test-block {
		margin-top: 0.9rem;
		padding: 0.9rem 1rem;
		border-radius: 0.85rem;
		border: 1px solid var(--border-color-default);
		background: var(--field-color);
	}

	.settings-test-label {
		font-size: 0.72rem;
		font-weight: 700;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--muted-color);
	}

	.settings-test-value {
		font-size: 0.9rem;
		line-height: 1.5;
		color: var(--on-surface-color-strong);
	}

	.settings-test-error {
		padding: 0.8rem 0.9rem;
		border-radius: 0.85rem;
		border: 1px solid color-mix(in srgb, var(--danger-color) 45%, transparent);
		background: color-mix(in srgb, var(--danger-color) 10%, transparent);
		color: var(--danger-color);
		font-size: 0.82rem;
	}

	.settings-key-hint {
		margin-top: 0.45rem;
		font-size: 0.74rem;
		color: var(--muted-color);
		line-height: 1.5;
	}

	.settings-helper-text {
		margin-top: 1.25rem;
		font-size: 0.875rem;
		line-height: 1.5;
		color: var(--on-surface-color);
	}
</style>
