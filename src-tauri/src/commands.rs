use crate::audio;
use crate::vosk::VoskAsr;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_store::StoreExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioCaptureResponse {
    pub sample_rate: u32,
    pub channels: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AsrResponse {
    pub text: String,
    pub is_final: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranslateResponse {
    pub original: String,
    pub translated: String,
}

fn normalize_translation_endpoint(endpoint: &str) -> String {
    let endpoint = endpoint.trim().trim_end_matches('/');

    // Already a full chat completions URL or special inference path — use as-is.
    if endpoint.ends_with("/chat/completions") || endpoint.contains("/v1/inference/") {
        return endpoint.to_string();
    }

    // DashScope base path: .../compatible-mode/v1  →  .../chat/completions
    // If caller already provides a deeper compatible-mode path, keep as-is.
    if endpoint.contains("/compatible-mode/v1") {
        if endpoint.ends_with("/compatible-mode/v1") {
            return format!("{}/chat/completions", endpoint);
        }
        return endpoint.to_string();
    }

    // Gemini: .../v1beta/openai  →  .../v1beta/openai/chat/completions
    // DeepInfra: .../v1/openai  →  .../v1/openai/chat/completions
    if endpoint.ends_with("/openai") {
        return format!("{}/chat/completions", endpoint);
    }

    // Standard /v1 base  →  /v1/chat/completions
    if endpoint.ends_with("/v1") || endpoint.ends_with("/v1beta") {
        return format!("{}/chat/completions", endpoint);
    }

    // Bare host  →  /v1/chat/completions
    format!("{}/v1/chat/completions", endpoint)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub theme: String,
    pub engine: String,
    pub source_lang: String,
    pub target_lang: String,
    pub target_lang_2: String,
    pub overlay_transparency: f32,
    pub overlay_font_size: u32,
    pub subtitle_position: u32,
    pub translation_engine: String,
    pub translation_endpoint: Option<String>,
    pub translation_model: Option<String>,
    pub translation_api_key_name: Option<String>,
    pub remote_endpoint: Option<String>,
    pub remote_api_key_name: Option<String>,
}

#[tauri::command]
pub async fn audio_capture_start(
    state: State<'_, audio::AudioState>,
) -> Result<AudioCaptureResponse, String> {
    audio::start_capture(&state)?;
    Ok(AudioCaptureResponse {
        sample_rate: 16000,
        channels: 1,
    })
}

#[tauri::command]
pub async fn audio_capture_stop(state: State<'_, audio::AudioState>) -> Result<(), String> {
    audio::stop_capture(&state)?;
    Ok(())
}

#[tauri::command]
pub async fn asr_infer(audio_data: Vec<u8>, sample_rate: u32) -> Result<AsrResponse, String> {
    Ok(AsrResponse {
        text: "stub".to_string(),
        is_final: true,
    })
}

#[tauri::command]
pub async fn translate(
    text: String,
    source_lang: String,
    target_lang: String,
    endpoint: String,
    model: String,
    api_key: String,
) -> Result<TranslateResponse, String> {
    if endpoint.trim().is_empty() {
        return Err("Translation endpoint is not configured".to_string());
    }
    if model.trim().is_empty() {
        return Err("Translation model is not configured".to_string());
    }
    if api_key.trim().is_empty() {
        return Err("Translation API key is not configured".to_string());
    }

    let url = normalize_translation_endpoint(&endpoint);

    let client = Client::new();
    let payload = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": format!(
                    "Translate the user's text from {} to {}. Return only the translated text.",
                    source_lang, target_lang
                )
            },
            {
                "role": "user",
                "content": text
            }
        ],
        "temperature": 0.2,
    });

    log::debug!("[Translate] Calling endpoint: {}", endpoint);
    log::debug!("[Translate] Resolved URL: {}", url);
    log::debug!("[Translate] Model: {}", model);
    log::debug!("[Translate] API key length: {}", api_key.len());

    let response = client
        .post(&url)
        .bearer_auth(&api_key)
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            log::error!("[Translate] Request send failed: {}", e);
            e.to_string()
        })?;

    let status = response.status();
    if !status.is_success() {
        let error_body = response.text().await.unwrap_or_default();
        log::error!("[Translate] API returned status {}: {}", status.as_u16(), error_body);
        if status.as_u16() == 401 {
            return Err("Invalid API key (401 Unauthorized)".to_string());
        }
        if status.as_u16() == 429 {
            return Err("Rate limited - try again later (429)".to_string());
        }
        return Err(format!(
            "API error {} at {}: {}",
            status.as_u16(),
            url,
            error_body
        ));
    }

    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    let translated = json
        .get("choices")
        .and_then(|choices| choices.as_array())
        .and_then(|choices| choices.first())
        .and_then(|choice| {
            choice
                .get("message")
                .and_then(|message| message.get("content"))
                .and_then(|content| content.as_str())
                .or_else(|| choice.get("text").and_then(|content| content.as_str()))
        })
        .map(str::trim)
        .filter(|content| !content.is_empty())
        .ok_or_else(|| format!("Empty response from API at {}: {}", url, json))?
        .to_string();

    Ok(TranslateResponse {
        original: text,
        translated,
    })
}

#[tauri::command]
pub async fn settings_get(app: tauri::AppHandle) -> Result<Settings, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;

    fn opt_str_to_string(opt: Option<&str>) -> String {
        opt.map(String::from)
            .unwrap_or_else(|| "browser".to_string())
    }

    let theme = match store.get("theme") {
        Some(v) => v
            .as_str()
            .map(String::from)
            .unwrap_or_else(|| "night".to_string()),
        None => "night".to_string(),
    };
    let engine = match store.get("engine") {
        Some(v) => opt_str_to_string(v.as_str()),
        None => "browser".to_string(),
    };
    let source_lang = match store.get("source_lang") {
        Some(v) => opt_str_to_string(v.as_str()),
        None => "en-US".to_string(),
    };
    let target_lang = match store.get("target_lang") {
        Some(v) => opt_str_to_string(v.as_str()),
        None => "es".to_string(),
    };
    let target_lang_2 = match store.get("target_lang_2") {
        Some(v) => v
            .as_str()
            .map(String::from)
            .unwrap_or_else(|| "none".to_string()),
        None => "none".to_string(),
    };
    let overlay_transparency = store
        .get("overlay_transparency")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.7) as f32;
    let overlay_font_size = store
        .get("overlay_font_size")
        .and_then(|v| v.as_u64())
        .unwrap_or(24) as u32;
    let subtitle_position = store
        .get("subtitle_position")
        .and_then(|v| v.as_u64())
        .unwrap_or(20) as u32;
    let translation_engine = match store.get("translation_engine") {
        Some(v) => opt_str_to_string(v.as_str()),
        None => "remote".to_string(),
    };
    let translation_endpoint = match store.get("translation_endpoint") {
        Some(v) => v.as_str().map(String::from),
        None => None,
    };
    let translation_model = match store.get("translation_model") {
        Some(v) => v.as_str().map(String::from),
        None => None,
    };
    let translation_api_key_name = match store.get("translation_api_key_name") {
        Some(v) => v.as_str().map(String::from),
        None => None,
    };
    let remote_endpoint = match store.get("remote_endpoint") {
        Some(v) => v.as_str().map(String::from),
        None => None,
    };
    let remote_api_key_name = match store.get("remote_api_key_name") {
        Some(v) => v.as_str().map(String::from),
        None => None,
    };

    Ok(Settings {
        theme,
        engine,
        source_lang,
        target_lang,
        target_lang_2,
        overlay_transparency,
        overlay_font_size,
        subtitle_position,
        translation_engine,
        translation_endpoint,
        translation_model,
        translation_api_key_name,
        remote_endpoint,
        remote_api_key_name,
    })
}

#[tauri::command]
pub async fn settings_set(app: tauri::AppHandle, key: String, value: String) -> Result<(), String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;

    // Determine type and store appropriately
    match key.as_str() {
        "theme"
        | "engine"
        | "source_lang"
        | "target_lang"
        | "target_lang_2"
        | "translation_endpoint"
        | "translation_model"
        | "translation_api_key_name"
        | "remote_endpoint"
        | "remote_api_key_name"
        | "translation_engine"
        | "source_language" => {
            store.set(key, serde_json::Value::String(value));
        }
        "overlay_transparency" => {
            if let Ok(v) = value.parse::<f64>() {
                let num =
                    serde_json::Number::from_f64(v).unwrap_or_else(|| serde_json::Number::from(0));
                store.set(key, serde_json::Value::Number(num));
            }
        }
        "overlay_font_size" | "subtitle_position" => {
            if let Ok(v) = value.parse::<u64>() {
                store.set(key, serde_json::Value::Number(serde_json::Number::from(v)));
            }
        }
        _ => {
            store.set(key, serde_json::Value::String(value));
        }
    }

    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

/// Demonstrates event emission for streaming subtitle results.
/// Actual implementation in Plan 02 (event streaming + key management).
#[tauri::command]
pub async fn test_event_emission(app: tauri::AppHandle) -> Result<String, String> {
    use tauri::Emitter;
    let payload = serde_json::json!({
        "text": "test subtitle",
        "is_final": false,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    });
    app.emit("backend://subtitle/update", payload)
        .map_err(|e| e.to_string())?;
    Ok("event emitted".to_string())
}

#[tauri::command]
pub async fn api_key_get(key_name: String) -> Result<Option<String>, String> {
    log::warn!("api_key_get is deprecated - frontend uses Stronghold JS API directly");
    Ok(None)
}

#[tauri::command]
pub async fn api_key_set(key_name: String, key_value: String) -> Result<(), String> {
    log::warn!("api_key_set is deprecated - frontend uses Stronghold JS API directly");
    Ok(())
}

#[tauri::command]
pub async fn stronghold_get_vault_path(_app: tauri::AppHandle) -> Result<String, String> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let app_data_dir = format!("{}/.local/share/com.subtitle.realtime", manifest_dir);
    let vault_path = format!("{}/vault.hold", app_data_dir);
    Ok(vault_path)
}

#[tauri::command]
pub async fn stronghold_get_password(app: tauri::AppHandle) -> Result<String, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    match store.get("stronghold_password") {
        Some(v) => v
            .as_str()
            .map(String::from)
            .ok_or_else(|| "stronghold_password is not a string".to_string()),
        None => {
            let password = uuid::Uuid::new_v4().to_string();
            store.set(
                "stronghold_password",
                serde_json::Value::String(password.clone()),
            );
            store.save().map_err(|e| e.to_string())?;
            log::info!("Generated new Stronghold password");
            Ok(password)
        }
    }
}

#[tauri::command]
pub async fn vosk_load_model(path: String, state: State<'_, VoskAsr>) -> Result<(), String> {
    let model = state.inner().model.clone();
    let path = path.clone();
    tokio::task::spawn_blocking(move || {
        let model_guard = model.lock().map_err(|e| e.to_string())?;
        if model_guard.is_some() {
            return Err("Model already loaded".to_string());
        }
        drop(model_guard);

        let vosk_model =
            vosk::Model::new(&path).ok_or_else(|| format!("Failed to load model from {}", path))?;
        let mut model_guard = model.lock().map_err(|e| e.to_string())?;
        *model_guard = Some(std::sync::Arc::new(vosk_model));
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn vosk_get_model_path() -> Result<String, String> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let model_path = format!("{}/vosk-model", manifest_dir);
    let path = std::path::Path::new(&model_path);
    if !path.exists() {
        return Err(format!("Model path does not exist: {}", model_path));
    }
    Ok(model_path)
}

#[tauri::command]
pub async fn vosk_start(
    app: tauri::AppHandle,
    audio_state: State<'_, audio::AudioState>,
    vosk_state: State<'_, VoskAsr>,
) -> Result<(), String> {
    vosk_state.start(app, audio_state)
}

#[tauri::command]
pub async fn vosk_stop(vosk_state: State<'_, VoskAsr>) -> Result<(), String> {
    vosk_state.stop().await
}

// =============================================================================
// Event Streaming Protocol
// =============================================================================
//
// Event channel: "backend://subtitle/{event_type}"
//
// Event types:
//   - "update"     → Partial/interim transcription result
//   - "final"      → Final transcription result
//   - "translated" → Translation result for a subtitle line
//   - "error"      → Error occurred during processing
//
// Payload structure for "update" / "final":
//   { "id": "uuid-string", "text": "transcribed text", "is_final": bool, "timestamp": ms }
//
// Payload structure for "translated":
//   { "id": "uuid-string", "original": "source text", "translated": "target text", "timestamp": ms }
//
// Payload structure for "error":
//   { "code": "ERROR_CODE", "message": "error description" }
// =============================================================================

/// Emit a subtitle update event (interim results)
pub fn emit_subtitle_update(
    app: &AppHandle,
    id: &str,
    text: &str,
    is_final: bool,
) -> Result<(), String> {
    let payload = serde_json::json!({
        "id": id,
        "text": text,
        "is_final": is_final,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    });
    app.emit("backend://subtitle/update", payload)
        .map_err(|e| e.to_string())
}

/// Emit a final subtitle result event
pub fn emit_subtitle_final(app: &AppHandle, id: &str, text: &str) -> Result<(), String> {
    let payload = serde_json::json!({
        "id": id,
        "text": text,
        "is_final": true,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    });
    app.emit("backend://subtitle/final", payload)
        .map_err(|e| e.to_string())
}

/// Emit a translation result event
pub fn emit_translation(
    app: &AppHandle,
    id: &str,
    original: &str,
    translated: &str,
) -> Result<(), String> {
    let payload = serde_json::json!({
        "id": id,
        "original": original,
        "translated": translated,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    });
    app.emit("backend://subtitle/translated", payload)
        .map_err(|e| e.to_string())
}

/// Emit an error event
pub fn emit_error(app: &AppHandle, code: &str, message: &str) -> Result<(), String> {
    let payload = serde_json::json!({
        "code": code,
        "message": message
    });
    app.emit("backend://subtitle/error", payload)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenvy::dotenv;
    use std::env;
    use tokio::runtime::Runtime;

    #[test]
    fn test_normalize_full_chat_completions_preserved() {
        for url in [
            "https://dashscope-intl.aliyuncs.com/compatible-mode/v1/chat/completions",
            "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions",
            "https://api.deepinfra.com/v1/openai/chat/completions",
            "https://api.openai.com/v1/chat/completions",
        ] {
            assert_eq!(normalize_translation_endpoint(url), url, "should preserve: {}", url);
        }
    }

    #[test]
    fn test_normalize_translation_endpoint_from_v1() {
        let normalized = normalize_translation_endpoint("https://example.com/v1");
        assert_eq!(normalized, "https://example.com/v1/chat/completions");
    }

    #[test]
    fn test_normalize_translation_endpoint_from_base_url() {
        let normalized = normalize_translation_endpoint("https://example.com");
        assert_eq!(normalized, "https://example.com/v1/chat/completions");
    }

    #[test]
    fn test_normalize_translation_endpoint_dashscope_base_preserved() {
        let endpoint = "https://dashscope-intl.aliyuncs.com/compatible-mode/v1";
        let normalized = normalize_translation_endpoint(endpoint);
        assert_eq!(
            normalized,
            "https://dashscope-intl.aliyuncs.com/compatible-mode/v1/chat/completions"
        );
    }

    #[test]
    fn test_normalize_translation_endpoint_deepinfra_inference_preserved() {
        let endpoint = "https://api.deepinfra.com/v1/inference/Qwen/Qwen3-32B";
        let normalized = normalize_translation_endpoint(endpoint);
        assert_eq!(normalized, endpoint);
    }

    #[test]
    fn test_normalize_gemini_base_url() {
        // Gemini base: .../v1beta/openai
        let normalized = normalize_translation_endpoint(
            "https://generativelanguage.googleapis.com/v1beta/openai",
        );
        assert_eq!(
            normalized,
            "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions"
        );
    }

    #[test]
    fn test_normalize_deepinfra_openai_base_url() {
        // DeepInfra OpenAI-compat base: .../v1/openai
        let normalized =
            normalize_translation_endpoint("https://api.deepinfra.com/v1/openai");
        assert_eq!(
            normalized,
            "https://api.deepinfra.com/v1/openai/chat/completions"
        );
    }

    #[test]
    fn test_settings_serialization_roundtrip() {
        let settings = Settings {
            theme: "night".to_string(),
            engine: "browser".to_string(),
            source_lang: "en-US".to_string(),
            target_lang: "th".to_string(),
            target_lang_2: "ja".to_string(),
            overlay_transparency: 0.8,
            overlay_font_size: 32,
            subtitle_position: 25,
            translation_engine: "none".to_string(),
            translation_endpoint: Some("https://api.example.com/v1/chat/completions".to_string()),
            translation_model: Some("gpt-4o-mini".to_string()),
            translation_api_key_name: Some("translation".to_string()),
            remote_endpoint: Some("https://api.openai.com".to_string()),
            remote_api_key_name: Some("openai".to_string()),
        };

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: Settings = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.engine, settings.engine);
        assert_eq!(deserialized.source_lang, settings.source_lang);
        assert_eq!(deserialized.target_lang, settings.target_lang);
        assert_eq!(deserialized.target_lang_2, settings.target_lang_2);
        assert_eq!(
            deserialized.overlay_transparency,
            settings.overlay_transparency
        );
        assert_eq!(deserialized.overlay_font_size, settings.overlay_font_size);
        assert_eq!(deserialized.subtitle_position, settings.subtitle_position);
        assert_eq!(deserialized.translation_engine, settings.translation_engine);
        assert_eq!(deserialized.translation_endpoint, settings.translation_endpoint);
        assert_eq!(deserialized.translation_model, settings.translation_model);
        assert_eq!(
            deserialized.translation_api_key_name,
            settings.translation_api_key_name
        );
        assert_eq!(deserialized.remote_endpoint, settings.remote_endpoint);
        assert_eq!(
            deserialized.remote_api_key_name,
            settings.remote_api_key_name
        );
    }

    #[test]
    fn test_settings_default_values() {
        let settings = Settings {
            theme: "night".to_string(),
            engine: "browser".to_string(),
            source_lang: "en-US".to_string(),
            target_lang: "es".to_string(),
            target_lang_2: "none".to_string(),
            overlay_transparency: 0.7,
            overlay_font_size: 24,
            subtitle_position: 20,
            translation_engine: "remote".to_string(),
            translation_endpoint: None,
            translation_model: None,
            translation_api_key_name: None,
            remote_endpoint: None,
            remote_api_key_name: None,
        };

        assert_eq!(settings.overlay_transparency, 0.7);
        assert_eq!(settings.overlay_font_size, 24);
        assert!(settings.remote_endpoint.is_none());
    }

    #[test]
    fn test_asr_response_serialization() {
        let response = AsrResponse {
            text: "Hello world".to_string(),
            is_final: true,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("Hello world"));
        assert!(json.contains("true"));

        let deserialized: AsrResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.text, "Hello world");
        assert!(deserialized.is_final);
    }

    #[test]
    fn test_translate_response_serialization() {
        let response = TranslateResponse {
            original: "Hello".to_string(),
            translated: "สวัสดี".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: TranslateResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.original, "Hello");
        assert_eq!(deserialized.translated, "สวัสดี");
    }

    #[test]
    fn test_audio_capture_response_serialization() {
        let response = AudioCaptureResponse {
            sample_rate: 16000,
            channels: 1,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: AudioCaptureResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.sample_rate, 16000);
        assert_eq!(deserialized.channels, 1);
    }

    fn run_ai_translation_test(
        provider: &str,
        endpoint: &str,
        model: &str,
        env_key_name: &str,
        source_lang: &str,
        target_lang: &str,
    ) {
        let _ = dotenv();
        let api_key = env::var(env_key_name)
            .unwrap_or_else(|_| panic!("Missing env var {} for {} test", env_key_name, provider));

        let rt = Runtime::new().expect("failed to create tokio runtime");
        let result = rt.block_on(translate(
            "Hello, this is a test.".to_string(),
            source_lang.to_string(),
            target_lang.to_string(),
            endpoint.to_string(),
            model.to_string(),
            api_key,
        ));

        match result {
            Ok(resp) => {
                assert!(
                    !resp.translated.trim().is_empty(),
                    "{} translation should not be empty",
                    provider
                );
                println!("[AI TEST] {} translated: {}", provider, resp.translated);
            }
            Err(err) => panic!("{} translation failed: {}", provider, err),
        }
    }

    #[test]
    #[ignore = "live API test"]
    fn ai_dashscope_translate_live() {
        run_ai_translation_test(
            "dashscope",
            "https://dashscope-intl.aliyuncs.com/compatible-mode/v1",
            "qwen-plus",
            "DASHSCOPE_API_KEY",
            "en-US",
            "th",
        );
    }

    #[test]
    #[ignore = "live API test"]
    fn ai_deepinfra_translate_live() {
        run_ai_translation_test(
            "deepinfra",
            "https://api.deepinfra.com/v1/openai",
            "Qwen/Qwen3.5-2B",
            "VITE_DEEPINFRA_API_KEY",
            "en-US",
            "th",
        );
    }

    #[test]
    #[ignore = "live API test"]
    fn ai_gemini_translate_live() {
        run_ai_translation_test(
            "gemini",
            "https://generativelanguage.googleapis.com/v1beta/openai",
            "gemini-3-flash-preview",
            "YOUR_GOOGLE_AI_STUDIO_KEY",
            "en-US",
            "th",
        );
    }
}
