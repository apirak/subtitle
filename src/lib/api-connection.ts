/**
 * Shared HTTP utilities for OpenAI-compatible API calls.
 * Used by both STT (remote transcription) and Translation engines.
 */

import { invoke } from "@tauri-apps/api/core";

interface TranslateResponse {
	original: string;
	translated: string;
}

export function resolveOpenAICompatibleEndpoint(endpoint: string): string {
	const e = endpoint.trim().replace(/\/+$/, "");

	if (!e) return "";

	// Already a full chat completions URL or special inference path — use as-is.
	if (e.endsWith("/chat/completions") || e.includes("/v1/inference/")) {
		return e;
	}

	// DashScope base path: .../compatible-mode/v1 → .../chat/completions
	if (e.includes("/compatible-mode/v1")) {
		if (e.endsWith("/compatible-mode/v1")) {
			return `${e}/chat/completions`;
		}
		return e;
	}

	// Gemini: .../v1beta/openai  →  .../v1beta/openai/chat/completions
	// DeepInfra: .../v1/openai  →  .../v1/openai/chat/completions
	if (e.endsWith("/openai")) {
		return `${e}/chat/completions`;
	}

	// Standard /v1 or /v1beta base  →  .../chat/completions
	if (e.endsWith("/v1") || e.endsWith("/v1beta")) {
		return `${e}/chat/completions`;
	}

	// Bare host  →  /v1/chat/completions
	return `${e}/v1/chat/completions`;
}

/**
 * Call OpenAI-compatible API endpoint.
 * @param endpoint Endpoint from settings (e.g., https://dashscope-intl.aliyuncs.com/compatible-mode/v1)
 * @param apiKey Bearer token for Authorization header
 * @param payload OpenAI-style request body
 * @returns Extracted string from choices[0].message.content
 */
export async function callOpenAICompatibleAPI(
	endpoint: string,
	apiKey: string,
	payload: Record<string, unknown>
): Promise<string> {
	if (!endpoint.trim()) {
		throw new Error("API endpoint is not configured");
	}
	if (!apiKey.trim()) {
		throw new Error("API key is not configured");
	}

	const resolvedEndpoint = resolveOpenAICompatibleEndpoint(endpoint);

	console.log("A - Calling OpenAI-compatible endpoint:", endpoint);
	console.log("A - Resolved OpenAI-compatible endpoint:", resolvedEndpoint);
	console.log("A - Calling OpenAI-compatible payload:", payload);
	const model = typeof payload.model === "string" ? payload.model : "";
	const messages = Array.isArray(payload.messages)
		? (payload.messages as Array<{ role?: string; content?: string }>)
		: [];
	const userMessage = messages.find((message) => message.role === "user")?.content;
	const systemMessage = messages.find((message) => message.role === "system")?.content;

	if (!model.trim()) {
		throw new Error("Model is not configured");
	}
	if (typeof userMessage !== "string" || !userMessage.trim()) {
		throw new Error("User message is missing from payload");
	}
	if (typeof systemMessage !== "string" || !systemMessage.trim()) {
		throw new Error("System message is missing from payload");
	}

	const match = systemMessage.match(
		/^Translate the user's text from\s+(.+?)\s+to\s+(.+?)\. Return only the translated text\.$/
	);

	if (!match) {
		throw new Error("Unable to extract source/target languages from translation prompt");
	}

	const [, sourceLang, targetLang] = match;
	const response = await invoke<TranslateResponse>("translate", {
		text: userMessage,
		sourceLang,
		targetLang,
		endpoint,
		model,
		apiKey,
	});
	console.log("B");

	const content = response.translated?.trim();
	if (!content) {
		throw new Error("Empty response from API");
	}

	return content;
}

/**
 * Compose OpenAI-style chat completion request body.
 */
export function composeOpenAIRequest(
	model: string,
	sourceLang: string,
	targetLang: string,
	text: string,
	temperature: number = 0.2
): Record<string, unknown> {
	return {
		model,
		messages: [
			{
				role: "system",
				content: `Translate the user's text from ${sourceLang} to ${targetLang}. Return only the translated text.`,
			},
			{ role: "user", content: text },
		],
		temperature,
	};
}
