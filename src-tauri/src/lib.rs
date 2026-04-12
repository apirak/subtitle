mod audio;
mod commands;
mod remote_asr;
mod vosk;

use tauri::{Manager, State};

struct RemoteAsrState {
    is_running: bool,
}

#[tauri::command]
async fn remote_asr_start(
    app: tauri::AppHandle,
    audio_state: State<'_, audio::AudioState>,
) -> Result<(), String> {
    let settings = commands::settings_get(app.clone()).await?;

    let endpoint = match settings.remote_endpoint {
        Some(ep) if !ep.is_empty() => ep,
        _ => return Err("No remote endpoint configured".to_string()),
    };

    let api_key = commands::api_key_get("remote".to_string()).await?
        .ok_or("No API key configured".to_string())?;

    let receiver = audio_state.take_receiver()
        .ok_or("Audio capture not started or receiver already taken".to_string())?;

    remote_asr::remote_asr_start(
        app,
        receiver,
        endpoint,
        api_key,
        settings.source_lang,
    ).await
}

#[tauri::command]
async fn remote_asr_stop() -> Result<(), String> {
    remote_asr::remote_asr_stop().await
}

#[tauri::command]
fn remote_asr_status() -> remote_asr::RemoteAsrStatus {
    remote_asr::remote_asr_status()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      app.handle().plugin(tauri_plugin_store::Builder::default().build())?;
      app.manage(audio::AudioState::new());
      app.manage(vosk::VoskAsr::new());
      app.manage(RemoteAsrState { is_running: false });
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      commands::audio_capture_start,
      commands::audio_capture_stop,
      commands::asr_infer,
      commands::translate,
      commands::settings_get,
      commands::settings_set,
      commands::test_event_emission,
      commands::api_key_get,
      commands::api_key_set,
      commands::vosk_load_model,
      commands::vosk_get_model_path,
      commands::vosk_start,
      commands::vosk_stop,
      remote_asr_start,
      remote_asr_stop,
      remote_asr_status,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
