/**
 * Translation engine implementation for OpenAI-compatible APIs.
 * Team: Translation develops independently here.
 */

import { callOpenAICompatibleAPI, composeOpenAIRequest } from "./api-connection";

export type TranslationEngine = "none" | "remote";

export interface TranslationConfig {
	engine: TranslationEngine;
	model: string;
	endpoint: string;
	apiKey: string;
}

/**
 * Validate translation configuration before translating.
 */
export function validateTranslationConfig(config: TranslationConfig): { valid: boolean; error?: string } {
	if (config.engine === "none") {
		return { valid: true };
	}

	if (config.engine === "remote") {
		if (!config.model?.trim()) {
			return { valid: false, error: "Translation model not configured" };
		}
		if (!config.endpoint?.trim()) {
			return { valid: false, error: "Translation endpoint not configured" };
		}
		if (!config.apiKey?.trim()) {
			return { valid: false, error: "Translation API key not configured" };
		}
	}

	return { valid: true };
}

/**
 * Translate text using OpenAI-compatible API (DashScope, OpenAI, etc.).
 * @throws Error if endpoint/key missing or API call fails
 */
export async function translateWithOpenAI(
	text: string,
	sourceLang: string,
	targetLang: string,
	config: TranslationConfig
): Promise<string> {
	if (config.engine === "none") {
		return "";
	}

	const request = composeOpenAIRequest(config.model, sourceLang, targetLang, text);

	try {
		const result = await callOpenAICompatibleAPI(config.endpoint, config.apiKey, request);
		return result;
	} catch (error) {
		const message = error instanceof Error ? error.message : String(error);
		console.error("[Translation] API call failed:", message);
		throw error;
	}
}

/**
 * Get UI label for translation engine.
 */
export function getTranslationEngineLabel(engine: TranslationEngine): string {
	const labels: Record<TranslationEngine, string> = {
		none: "None (disabled)",
		remote: "Remote (OpenAI-compatible)",
	};
	return labels[engine];
}
