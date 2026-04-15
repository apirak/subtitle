/**
 * STT (Speech-to-Text) configuration types and helpers.
 * Team: STT develops independently here.
 */

export type STTEngine = "browser" | "vosk" | "remote" | "gemini";

export interface STTConfig {
	engine: STTEngine;
	language: string;
	remoteEndpoint?: string;
	apiKey?: string;
}

/**
 * Validate STT configuration before starting recognition.
 */
export function validateSTTConfig(config: STTConfig): { valid: boolean; error?: string } {
	if (!config.engine) {
		return { valid: false, error: "STT engine not selected" };
	}

	if (config.engine === "remote" || config.engine === "gemini") {
		if (!config.remoteEndpoint?.trim()) {
			return { valid: false, error: "Remote endpoint not configured" };
		}
		if (!config.apiKey?.trim()) {
			return { valid: false, error: "API key not configured" };
		}
	}

	return { valid: true };
}

/**
 * Get UI label for STT engine.
 */
export function getSTTEngineLabel(engine: STTEngine): string {
	const labels: Record<STTEngine, string> = {
		browser: "Browser (Web Speech API)",
		vosk: "Vosk (On-device)",
		remote: "Remote (OpenAI-compatible)",
		gemini: "Gemini (Batch API)",
	};
	return labels[engine];
}
