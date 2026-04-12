use serde::{Deserialize, Serialize};
use keyring::Entry;
use crate::audio;
use crate::vosk::VoskAsr;
use tauri::{Emitter, State, AppHandle};

/// Service name used for keyring entries
const SERVICE_NAME: &str = "subtitle-app";

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub asr_engine: String,
    pub source_lang: String,
    pub target_lang: String,
    pub overlay_transparency: f32,
    pub overlay_font_size: u32,
    pub remote_endpoint: Option<String>,
    pub remote_api_key_name: Option<String>,
}

#[tauri::command]
pub async fn audio_capture_start(
    state: State<'_, audio::AudioState>,
) -> Result<AudioCaptureResponse, String> {
    audio::start_capture(&state)?;
    Ok(AudioCaptureResponse { sample_rate: 16000, channels: 1 })
}

#[tauri::command]
pub async fn audio_capture_stop(
    state: State<'_, audio::AudioState>,
) -> Result<(), String> {
    audio::stop_capture(&state)?;
    Ok(())
}

#[tauri::command]
pub async fn asr_infer(audio_data: Vec<u8>, sample_rate: u32) -> Result<AsrResponse, String> {
    Ok(AsrResponse { text: "stub".to_string(), is_final: true })
}

#[tauri::command]
pub async fn translate(text: String, source_lang: String, target_lang: String) -> Result<TranslateResponse, String> {
    let translated = format!("[translated: {}]", text);
    Ok(TranslateResponse { original: text, translated })
}

#[tauri::command]
pub async fn settings_get() -> Result<Settings, String> {
    Ok(Settings {
        asr_engine: "none".to_string(),
        source_lang: "en".to_string(),
        target_lang: "es".to_string(),
        overlay_transparency: 0.7,
        overlay_font_size: 24,
        remote_endpoint: None,
        remote_api_key_name: None,
    })
}

#[tauri::command]
pub async fn settings_set(settings: Settings) -> Result<(), String> {
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
    let entry = Entry::new(SERVICE_NAME, &key_name)
        .map_err(|e| format!("keyring error: {}", e))?;
    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("keyring error: {}", e)),
    }
}

#[tauri::command]
pub async fn api_key_set(key_name: String, key_value: String) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, &key_name)
        .map_err(|e| format!("keyring error: {}", e))?;
    entry.set_password(&key_value)
        .map_err(|e| format!("keyring error: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn vosk_load_model(
    path: String,
    state: State<'_, VoskAsr>,
) -> Result<(), String> {
    let model = state.inner().model.clone();
    let path = path.clone();
    tokio::task::spawn_blocking(move || {
        let model_guard = model.lock().map_err(|e| e.to_string())?;
        if model_guard.is_some() {
            return Err("Model already loaded".to_string());
        }
        drop(model_guard);
        
        let vosk_model = vosk::Model::new(&path)
            .ok_or_else(|| format!("Failed to load model from {}", path))?;
        let mut model_guard = model.lock().map_err(|e| e.to_string())?;
        *model_guard = Some(std::sync::Arc::new(vosk_model));
        Ok(())
    }).await.map_err(|e| e.to_string())?
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
pub async fn vosk_stop(
    vosk_state: State<'_, VoskAsr>,
) -> Result<(), String> {
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
pub fn emit_subtitle_update(app: &AppHandle, id: &str, text: &str, is_final: bool) -> Result<(), String> {
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
pub fn emit_translation(app: &AppHandle, id: &str, original: &str, translated: &str) -> Result<(), String> {
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
