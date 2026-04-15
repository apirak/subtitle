<script lang="ts">
	interface AsrEngineOption {
		value: "browser" | "vosk" | "remote";
		label: string;
	}

	interface Props {
		asrEngineOptions: AsrEngineOption[];
		currentEngine: "browser" | "vosk" | "remote";
		currentRemoteEndpoint: string;
		currentRemoteModel: string;
		currentApiKey: string;
		remoteApiKeyHint: string;
		onEngineChange: (engine: "browser" | "vosk" | "remote") => void;
		onRemoteEndpointChange: (value: string) => void;
		onRemoteModelChange: (value: string) => void;
		onApiKeyChange: (value: string) => void;
	}

	let {
		asrEngineOptions,
		currentEngine,
		currentRemoteEndpoint,
		currentRemoteModel,
		currentApiKey,
		remoteApiKeyHint,
		onEngineChange,
		onRemoteEndpointChange,
		onRemoteModelChange,
		onApiKeyChange,
	}: Props = $props();
</script>

<div class="mx-auto w-full max-w-140">
	<div class="settings-section-label">Speech To Text</div>

	<div class="mb-5">
		<div class="settings-field-label">ASR Engine</div>
		<div class="settings-radio-group" role="radiogroup" aria-label="ASR Engine">
			{#each asrEngineOptions as option}
				<label class="settings-radio-option" data-selected={currentEngine === option.value ? "true" : undefined}>
					<input
						type="radio"
						name="split-asr-engine"
						class="settings-radio-input"
						checked={currentEngine === option.value}
						onchange={() => onEngineChange(option.value)}
					/>
					<div>
						<div class="settings-radio-title">{option.label}</div>
						<div class="settings-radio-description">
							{#if option.value === "browser"}
								Use built-in Web Speech API in the webview.
							{:else if option.value === "vosk"}
								Use the local on-device Vosk recognizer.
							{:else}
								Use a remote OpenAI-compatible transcription API.
							{/if}
						</div>
					</div>
				</label>
			{/each}
		</div>
	</div>

	{#if currentEngine === "remote"}
		<div class="mb-5">
			<label class="settings-field-label" for="split-remote-endpoint">API Endpoint</label>
			<input
				id="split-remote-endpoint"
				type="url"
				class="settings-input"
				placeholder="https://api.example.com/v1/audio/transcriptions"
				value={currentRemoteEndpoint}
				oninput={(event) => onRemoteEndpointChange((event.target as HTMLInputElement).value)}
			/>
		</div>

		<div class="mb-5">
			<label class="settings-field-label" for="split-remote-model">Model</label>
			<input
				id="split-remote-model"
				type="text"
				class="settings-input"
				placeholder="whisper-1"
				value={currentRemoteModel}
				oninput={(event) => onRemoteModelChange((event.target as HTMLInputElement).value)}
			/>
		</div>

		<div>
			<label class="settings-field-label" for="split-api-key">API Key</label>
			<input
				id="split-api-key"
				type="password"
				class="settings-input"
				placeholder="sk-..."
				value={currentApiKey}
				onchange={(event) => onApiKeyChange((event.target as HTMLInputElement).value)}
			/>
			{#if remoteApiKeyHint}
				<div class="settings-key-hint">{remoteApiKeyHint}</div>
			{/if}
		</div>
	{/if}

	<p class="settings-helper-text">
		Use Browser/Vosk for local recognition, or Remote for OpenAI-compatible transcription APIs.
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
