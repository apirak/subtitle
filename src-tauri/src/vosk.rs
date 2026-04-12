use crate::commands::{emit_error, emit_subtitle_final, emit_subtitle_update};
use crate::audio::AudioState;
use log::{error, info};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State};
use tokio::sync::mpsc;
use vosk::{DecodingState, Model, Recognizer};

pub struct VoskAsr {
    pub(crate) model: Arc<Mutex<Option<Arc<Model>>>>,
    task_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
    stop_tx: Mutex<Option<mpsc::Sender<()>>>,
}

fn f32_to_i16(samples: &[f32]) -> Vec<i16> {
    samples
        .iter()
        .map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
        .collect()
}

impl VoskAsr {
    pub fn new() -> Self {
        Self {
            model: Arc::new(Mutex::new(None)),
            task_handle: Mutex::new(None),
            stop_tx: Mutex::new(None),
        }
    }

    pub fn load_model(&self, model_path: &str) -> Result<(), String> {
        let path = std::path::Path::new(model_path);
        if !path.exists() {
            return Err(format!("Model path does not exist: {}", model_path));
        }
        let model =
            Model::new(model_path).ok_or_else(|| format!("Failed to load model from {}", model_path))?;
        *self.model.lock().map_err(|e| e.to_string())? = Some(Arc::new(model));
        info!("Vosk model loaded from {}", model_path);
        Ok(())
    }

    pub fn start(
        &self,
        app: AppHandle,
        audio_state: State<'_, AudioState>,
    ) -> Result<(), String> {
        let model = self.model.lock().map_err(|e| e.to_string())?.clone();
        let model = model.ok_or("Model not loaded")?;

        let receiver = audio_state
            .take_receiver()
            .ok_or("Audio receiver not available")?;

        let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);
        *self.stop_tx.lock().map_err(|e| e.to_string())? = Some(stop_tx);

        let handle = tokio::spawn(async move {
            Self::recognition_loop(app, model, receiver, stop_rx).await;
        });

        *self.task_handle.lock().map_err(|e| e.to_string())? = Some(handle);
        info!("Vosk ASR started");
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), String> {
        if let Some(tx) = self.stop_tx.lock().map_err(|e| e.to_string())?.take() {
            let _ = tx.try_send(());
        }
        let handle = {
            let mut guard = self.task_handle.lock().map_err(|e| e.to_string())?;
            guard.take()
        };
        if let Some(h) = handle {
            let _ = h.await;
        }
        info!("Vosk ASR stopped");
        Ok(())
    }

    async fn recognition_loop(
        app: AppHandle,
        model: Arc<Model>,
        mut rx: mpsc::Receiver<Vec<f32>>,
        mut stop_rx: mpsc::Receiver<()>,
    ) {
        let mut recognizer = match Recognizer::new(&model, 16000.0) {
            Some(r) => r,
            None => {
                let _ = emit_error(&app, "VOSK_INIT", "Failed to create recognizer");
                return;
            }
        };

        let mut current_id: Option<String> = None;

        loop {
            tokio::select! {
                Some(f32_chunk) = rx.recv() => {
                    let i16_chunk = f32_to_i16(&f32_chunk);

                    match recognizer.accept_waveform(&i16_chunk) {
                        Ok(state) => {
                            let partial = recognizer.partial_result();
                            if !partial.partial.is_empty() {
                                let id =
                                    current_id.get_or_insert_with(|| format!("vosk-{}", uuid::Uuid::new_v4()));
                                let _ = emit_subtitle_update(&app, &id, &partial.partial, false);
                            }

                            if state == DecodingState::Finalized {
                                let final_result = recognizer.final_result();
                                if let vosk::CompleteResult::Single(result) = final_result {
                                    if !result.text.is_empty() {
                                        let id = current_id.take().unwrap_or_else(|| {
                                            format!("vosk-{}", uuid::Uuid::new_v4())
                                        });
                                        let _ = emit_subtitle_final(&app, &id, &result.text);
                                    }
                                }
                                recognizer.reset();
                            }
                        }
                        Err(e) => {
                            error!("Vosk accept_waveform error: {:?}", e);
                        }
                    }
                }
                _ = stop_rx.recv() => {
                    info!("Vosk recognition loop received stop signal");
                    break;
                }
            }
        }
    }
}

impl Default for VoskAsr {
    fn default() -> Self {
        Self::new()
    }
}
