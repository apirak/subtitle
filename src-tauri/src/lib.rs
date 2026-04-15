mod audio;
mod commands;
mod remote_asr;
mod vosk;

use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_stronghold::Builder;

#[tauri::command]
async fn remote_asr_start(
    app: tauri::AppHandle,
    audio_state: tauri::State<'_, audio::AudioState>,
    remote_state: tauri::State<'_, Arc<remote_asr::RemoteAsrState>>,
    api_key: String,
) -> Result<(), String> {
    log::info!("remote_asr_start called");
    let settings = commands::settings_get(app.clone()).await?;
    log::info!(
        "settings loaded: engine={}, remote_endpoint={:?}",
        settings.engine,
        settings.remote_endpoint
    );

    let endpoint = match settings.remote_endpoint {
        Some(ep) if !ep.is_empty() => {
            log::info!("using endpoint: {}", ep);
            ep
        }
        _ => {
            log::warn!(
                "No remote endpoint configured - remote_endpoint is: {:?}",
                settings.remote_endpoint
            );
            return Err("No remote endpoint configured".to_string());
        }
    };

    if api_key.is_empty() {
        log::error!("No API key provided");
        return Err("No API key configured".to_string());
    }

    log::info!("API key provided, length: {}", api_key.len());

    let receiver = audio_state
        .take_receiver()
        .ok_or("Audio capture not started or receiver already taken".to_string())?;

    remote_asr::remote_asr_start(
        app,
        remote_state.inner().clone(),
        receiver,
        endpoint,
        api_key,
        settings.source_lang,
        settings.remote_model.unwrap_or_default(),
        settings.engine,
    )
    .await
}

#[tauri::command]
async fn remote_asr_stop(
    remote_state: tauri::State<'_, Arc<remote_asr::RemoteAsrState>>,
) -> Result<(), String> {
    remote_asr::remote_asr_stop(&remote_state).await
}

#[tauri::command]
async fn remote_asr_status(
    remote_state: tauri::State<'_, Arc<remote_asr::RemoteAsrState>>,
) -> Result<remote_asr::RemoteAsrStatus, String> {
    Ok(remote_asr::remote_asr_status(&remote_state).await)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Debug)
                        .build(),
                )?;
            }
            app.handle()
                .plugin(tauri_plugin_store::Builder::default().build())?;

            let salt_path = app
                .path()
                .app_local_data_dir()
                .unwrap()
                .join("stronghold.salt");
            app.handle()
                .plugin(Builder::with_argon2(&salt_path).build())?;

            app.manage(audio::AudioState::new());
            app.manage(vosk::VoskAsr::new());
            app.manage(Arc::new(remote_asr::RemoteAsrState::new()));
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
            commands::stronghold_get_vault_path,
            commands::stronghold_get_password,
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
