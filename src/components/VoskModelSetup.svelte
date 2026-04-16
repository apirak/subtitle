<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import { open } from "@tauri-apps/plugin-dialog";
	import { listen, type UnlistenFn } from "@tauri-apps/api/event";
	import { onMount, onDestroy } from "svelte";

	interface Props {
		onModelReady: () => void;
		onCancel: () => void;
	}

	let { onModelReady, onCancel }: Props = $props();

	let browseStatus = $state<"idle" | "copying" | "done" | "error">("idle");
	let downloadStatus = $state<"idle" | "downloading" | "extracting" | "done" | "error">("idle");
	let errorMessage = $state("");
	let progress = $state({ downloaded: 0, total: 0, percentage: 0 });

	let unlistenProgress: UnlistenFn | null = null;
	let unlistenComplete: UnlistenFn | null = null;
	let unlistenError: UnlistenFn | null = null;

	onMount(async () => {
		unlistenProgress = await listen<{ downloaded: number; total: number; percentage: number }>(
			"vosk-download-progress",
			(event) => {
				progress = event.payload;
			}
		);

		unlistenComplete = await listen<{ model_path: string }>("vosk-download-complete", () => {
			downloadStatus = "done";
			onModelReady();
		});

		unlistenError = await listen<{ message: string }>("vosk-download-error", (event) => {
			downloadStatus = "error";
			errorMessage = event.payload.message;
		});
	});

	onDestroy(() => {
		unlistenProgress?.();
		unlistenComplete?.();
		unlistenError?.();
	});

	async function handleBrowse() {
		const selected = await open({ multiple: false, directory: true });
		if (!selected) return;

		browseStatus = "copying";
		errorMessage = "";

		try {
			const path = typeof selected === "string" ? selected : (selected.path ?? "");
			if (!path) {
				errorMessage = "No directory selected";
				browseStatus = "error";
				return;
			}
			await invoke("vosk_model_set_from_directory", { path });
			browseStatus = "done";
			onModelReady();
		} catch (err) {
			browseStatus = "error";
			errorMessage = err instanceof Error ? err.message : String(err);
		}
	}

	async function handleDownload() {
		downloadStatus = "downloading";
		errorMessage = "";
		progress = { downloaded: 0, total: 0, percentage: 0 };

		try {
			await invoke("vosk_model_download", {
				url: "https://alphacephei.com/vosk/models/vosk-model-small-en-us-0.15.zip",
			});
		} catch (err) {
			downloadStatus = "error";
			errorMessage = err instanceof Error ? err.message : String(err);
		}
	}

	function formatBytes(bytes: number): string {
		if (bytes === 0) return "0 B";
		const k = 1024;
		const sizes = ["B", "KB", "MB", "GB"];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
	}
</script>

<div class="screen">
	<div class="icon-wrapper">
		<svg
			width="48"
			height="48"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="1.2"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
			<polyline points="7 10 12 15 17 10" />
			<line x1="12" y1="15" x2="12" y2="3" />
		</svg>
	</div>

	<span class="title">Vosk Model Required</span>
	<span class="description">
		On-device speech recognition requires a Vosk model. Download one or browse for an existing model directory.
	</span>

	<div class="actions">
		<button class="action-btn primary" onclick={handleBrowse} disabled={browseStatus === "copying"}>
			{#if browseStatus === "copying"}
				<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="spin">
					<circle cx="12" cy="12" r="10" fill="none" stroke-dasharray="15.7 47.1" />
				</svg>
				Copying...
			{:else if browseStatus === "done"}
				Done
			{:else}
				Browse for Model
			{/if}
		</button>

		<button
			class="action-btn secondary"
			onclick={handleDownload}
			disabled={downloadStatus === "downloading" || downloadStatus === "extracting"}
		>
			{#if downloadStatus === "downloading"}
				<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="spin">
					<circle cx="12" cy="12" r="10" fill="none" stroke-dasharray="15.7 47.1" />
				</svg>
				{progress.percentage}% ({formatBytes(progress.downloaded)} / {formatBytes(progress.total)})
			{:else if downloadStatus === "extracting"}
				<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="spin">
					<circle cx="12" cy="12" r="10" fill="none" stroke-dasharray="15.7 47.1" />
				</svg>
				Extracting...
			{:else if downloadStatus === "done"}
				Done
			{:else}
				Download Small Model (~40MB)
			{/if}
		</button>
	</div>

	{#if downloadStatus === "downloading" && progress.total > 0}
		<div class="progress-bar-wrapper">
			<div class="progress-bar" style="width: {progress.percentage}%"></div>
		</div>
	{/if}

	{#if errorMessage}
		<span class="error-detail">{errorMessage}</span>
	{/if}

	<a class="models-link" href="https://alphacephei.com/vosk/models" target="_blank" rel="noopener noreferrer">
		Browse all Vosk models
	</a>

	<button class="cancel-btn" onclick={onCancel}>Cancel</button>
</div>

<style>
	.screen {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 16px;
		max-width: 400px;
	}

	.icon-wrapper {
		color: var(--on-bg-color);
		opacity: 0.6;
		margin-bottom: 4px;
	}

	.title {
		font-size: 1.25rem;
		font-weight: 700;
		color: var(--on-bg-color-strong);
	}

	.description {
		font-size: 0.85rem;
		color: var(--on-bg-color);
		text-align: center;
		line-height: 1.5;
	}

	.actions {
		display: flex;
		flex-direction: column;
		gap: 10px;
		width: 100%;
		margin-top: 8px;
	}

	.action-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		width: 100%;
		padding: 12px 24px;
		font-size: 0.9rem;
		font-weight: 600;
		font-family: inherit;
		border-radius: 12px;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.action-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.action-btn.primary {
		color: var(--on-accent-color);
		background: var(--accent-color);
		border: 1px solid var(--accent-color);
	}

	.action-btn.primary:hover:not(:disabled) {
		background: var(--accent-color-hover);
		border-color: var(--accent-color-hover);
	}

	.action-btn.secondary {
		color: var(--on-bg-color-strong);
		background: rgba(255, 255, 255, 0.08);
		border: 1px solid rgba(255, 255, 255, 0.15);
	}

	.action-btn.secondary:hover:not(:disabled) {
		background: rgba(255, 255, 255, 0.12);
	}

	.progress-bar-wrapper {
		width: 100%;
		height: 4px;
		background: rgba(255, 255, 255, 0.1);
		border-radius: 2px;
		overflow: hidden;
	}

	.progress-bar {
		height: 100%;
		background: var(--accent-color);
		border-radius: 2px;
		transition: width 0.3s ease;
	}

	.error-detail {
		font-size: 0.8rem;
		color: var(--danger-color);
		text-align: center;
		max-width: 380px;
		word-break: break-word;
	}

	.models-link {
		font-size: 0.8rem;
		color: var(--accent-color);
		text-decoration: none;
		opacity: 0.8;
		transition: opacity 0.2s ease;
	}

	.models-link:hover {
		opacity: 1;
	}

	.cancel-btn {
		font-size: 0.85rem;
		font-family: inherit;
		color: var(--muted-color);
		background: none;
		border: none;
		cursor: pointer;
		padding: 8px 16px;
		transition: color 0.2s ease;
	}

	.cancel-btn:hover {
		color: var(--on-bg-color-strong);
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	:global(.spin) {
		animation: spin 1s linear infinite;
	}
</style>
