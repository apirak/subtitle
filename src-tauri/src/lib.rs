mod commands;

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
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
