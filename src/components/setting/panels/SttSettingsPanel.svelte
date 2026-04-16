<script lang="ts">
	interface AsrEngineOption {
		value: "browser" | "vosk" | "remote" | "gemini";
		label: string;
	}

	interface Props {
		asrEngineOptions: AsrEngineOption[];
		currentEngine: "browser" | "vosk" | "remote" | "gemini";
		currentRemoteEndpoint: string;
		currentRemoteModel: string;
		currentRemoteMinSpeechRms: number;
		currentApiKey: string;
		remoteApiKeyHint: string;
		onEngineChange: (engine: "browser" | "vosk" | "remote" | "gemini") => void;
		onRemoteEndpointChange: (value: string) => void;
		onRemoteModelChange: (value: string) => void;
		onRemoteMinSpeechRmsChange: (value: number) => void;
		onApiKeyChange: (value: string) => void;
		isTestingStt: boolean;
		testSttResolvedUrl: string;
		testSttResult: string;
		testSttError: string;
		onTestStt: () => void;
	}

	let {
		asrEngineOptions,
		currentEngine,
		currentRemoteEndpoint,
		currentRemoteModel,
		currentRemoteMinSpeechRms,
		currentApiKey,
		remoteApiKeyHint,
		onEngineChange,
		onRemoteEndpointChange,
		onRemoteModelChange,
		onRemoteMinSpeechRmsChange,
		onApiKeyChange,
		isTestingStt,
		testSttResolvedUrl,
		testSttResult,
		testSttError,
		onTestStt,
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
								Use the local on-device Vosk recognizer.						{:else if option.value === "gemini"}
							Use Gemini transcription API.							{:else}
								Use a remote OpenAI-compatible transcription API.
							{/if}
						</div>
					</div>
				</label>
			{/each}
		</div>
	</div>

	{#if currentEngine === "remote" || currentEngine === "gemini"}
		<div class="mb-5">
			<label class="settings-field-label" for="split-remote-endpoint">API Endpoint</label>
			<input
				id="split-remote-endpoint"
				type="url"
				class="settings-input"
				placeholder={currentEngine === "gemini"
					? "https://generativelanguage.googleapis.com/v1beta"
					: "https://api.example.com/v1/audio/transcriptions"}
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
				placeholder={currentEngine === "gemini" ? "gemini-2.0-flash" : "whisper-1"}
				value={currentRemoteModel}
				oninput={(event) => onRemoteModelChange((event.target as HTMLInputElement).value)}
			/>
		</div>

		<div class="mb-5">
			<label class="settings-field-label" for="split-remote-min-rms">Speech RMS Threshold</label>
			<input
				id="split-remote-min-rms"
				type="number"
				class="settings-input"
				min="0"
				max="1"
				step="0.001"
				value={currentRemoteMinSpeechRms}
				oninput={(event) => {
					const raw = (event.target as HTMLInputElement).value;
					const parsed = Number(raw);
					if (Number.isFinite(parsed) && parsed > 0 && parsed < 1) {
						onRemoteMinSpeechRmsChange(parsed);
					}
				}}
			/>
			<div class="settings-key-hint">
				Lower value is more sensitive (captures quieter speech). Suggested: 0.006-0.02.
			</div>
		</div>

		<div>
			<label class="settings-field-label" for="split-api-key">API Key</label>
			<input
				id="split-api-key"
				type="password"
				class="settings-input"
				placeholder={currentEngine === "gemini" ? "AIza..." : "sk-..."}
				value={currentApiKey}
				onchange={(event) => onApiKeyChange((event.target as HTMLInputElement).value)}
			/>
			{#if remoteApiKeyHint}
				<div class="settings-key-hint">{remoteApiKeyHint}</div>
			{/if}
		</div>

		<div class="mt-5">
			<button type="button" class="settings-test-button" onclick={onTestStt} disabled={isTestingStt}>
				{#if isTestingStt}Testing…{:else}Test STT{/if}
			</button>

			{#if testSttResolvedUrl}
				<div class="settings-test-meta">Resolved endpoint: {testSttResolvedUrl}</div>
			{/if}

			{#if testSttResult}
				<div class="settings-test-block">
					<div class="settings-test-label">Transcription result</div>
					<div class="settings-test-value">{testSttResult}</div>
				</div>
			{/if}

			{#if testSttError}
				<div class="settings-test-error">{testSttError}</div>
			{/if}
		</div>
	{/if}

	<p class="settings-helper-text">
		Use Browser/Vosk for local recognition, Remote for OpenAI-compatible APIs, or Gemini for Google's transcription API.
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
		margin-top: 0.4rem;
		font-size: 0.9rem;
		line-height: 1.5;
		color: var(--on-surface-color-strong);
	}

	.settings-test-error {
		font-size: 0.85rem;
		color: var(--error-color, #f87171);
	}

	.settings-helper-text {
		margin-top: 1.25rem;
		font-size: 0.875rem;
		line-height: 1.5;
		color: var(--on-surface-color);
	}
</style>
