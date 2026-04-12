use tokio::sync::mpsc;
use tauri::{AppHandle, Emitter};
use serde::Deserialize;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

// Shared cancellation flag — set to true to signal the async loop to exit
static STOP_FLAG: AtomicBool = AtomicBool::new(false);

const MAX_RETRIES: u32 = 3;
const REQUEST_TIMEOUT_SECS: u64 = 5;
const SILENCE_THRESHOLD: f32 = 0.005;
const MAX_CHUNK_SAMPLES: usize = 48_000;
const SILENCE_WINDOW_SAMPLES: usize = 1024;
const SILENCE_DURATION_MS: u64 = 300;

#[derive(Clone, serde::Serialize)]
pub struct RemoteAsrStatus {
    pub is_running: bool,
    pub chunks_accumulated: u32,
    pub last_transcript: Option<String>,
    pub error: Option<String>,
}

#[derive(Default)]
pub struct RemoteAsrState {
    pub is_running: bool,
    pub chunks_accumulated: u32,
    pub last_transcript: Option<String>,
    pub error: Option<String>,
}

impl RemoteAsrState {
    pub fn set_running(&mut self, running: bool) {
        self.is_running = running;
        if !running {
            self.chunks_accumulated = 0;
        }
    }

    pub fn add_chunk(&mut self, text: String) {
        self.chunks_accumulated += 1;
        self.last_transcript = Some(text);
    }

    pub fn set_error(&mut self, err: String) {
        self.error = Some(err);
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }
}

fn compute_rms(samples: &[f32]) -> f32 {
    let sum: f32 = samples.iter().map(|&s| s * s).sum();
    (sum / samples.len() as f32).sqrt()
}

fn is_silence(samples: &[f32]) -> bool {
    compute_rms(samples) < SILENCE_THRESHOLD
}

fn f32_to_i16(samples: &[f32]) -> Vec<i16> {
    samples.iter().map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16).collect()
}

fn encode_wav(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    let num_channels = 1u16;
    let bits_per_sample = 16u16;
    let byte_rate = sample_rate * num_channels as u32 * bits_per_sample as u32 / 8;
    let block_align = num_channels * bits_per_sample / 8;
    let data_size = samples.len() * 2;

    let mut wav = Vec::with_capacity(44 + data_size);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_size as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&num_channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(data_size as u32).to_le_bytes());
    for &sample in samples {
        wav.extend_from_slice(&sample.to_le_bytes());
    }
    wav
}

async fn send_transcription(
    client: &reqwest::Client,
    endpoint: &str,
    api_key: &str,
    audio_bytes: &[u8],
    language: Option<&str>,
) -> Result<String, String> {
    let audio_part = reqwest::multipart::Part::bytes(audio_bytes.to_vec())
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|e| e.to_string())?;

    let mut form = reqwest::multipart::Form::new()
        .part("file", audio_part)
        .text("model", "whisper-1");

    if let Some(lang) = language {
        let whisper_lang = lang.split('-').next().unwrap_or(lang).to_string();
        form = form.text("language", whisper_lang);
    }

    let resp = client
        .post(&format!("{}/v1/audio/transcriptions", endpoint.trim_end_matches('/')))
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .timeout(tokio::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.status() == 401 {
        return Err("Invalid API key".to_string());
    }
    if resp.status() == 429 {
        if let Some(retry_after) = resp.headers().get("Retry-After") {
            if let Ok(delay_str) = retry_after.to_str() {
                if let Ok(delay_secs) = delay_str.parse::<u64>() {
                    tokio::time::sleep(tokio::time::Duration::from_secs(delay_secs)).await;
                }
            }
        }
    }
    if !resp.status().is_success() {
        return Err(format!("API error {}: {}", resp.status().as_u16(), resp.text().await.unwrap_or_default()));
    }

    #[derive(Deserialize)]
    struct TranscriptionResponse { text: String }
    let transcription = resp.json::<TranscriptionResponse>()
        .await
        .map_err(|e| e.to_string())?;
    Ok(transcription.text)
}

async fn transcribe_with_retry(
    client: &reqwest::Client,
    endpoint: &str,
    api_key: &str,
    audio_bytes: &[u8],
    language: Option<&str>,
) -> Result<String, String> {
    for attempt in 0..MAX_RETRIES {
        match send_transcription(client, endpoint, api_key, audio_bytes, language).await {
            Ok(text) => return Ok(text),
            Err(e) if attempt < MAX_RETRIES - 1 => {
                tokio::time::sleep(tokio::time::Duration::from_millis(200 * (attempt + 1) as u64)).await;
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}

#[derive(Clone, serde::Serialize)]
struct SubtitlePayload {
    text: String,
    timestamp: u64,
    is_final: bool,
}

#[derive(Clone, serde::Serialize)]
struct AsrErrorPayload {
    message: String,
    retryable: bool,
}

pub async fn remote_asr_start(
    app: AppHandle,
    mut audio_receiver: mpsc::Receiver<Vec<f32>>,
    endpoint: String,
    api_key: String,
    source_lang: String,
) -> Result<(), String> {
    // Reset stop flag at start
    STOP_FLAG.store(false, Ordering::SeqCst);

    let client = reqwest::Client::new();
    let language = Some(source_lang);

    tauri::async_runtime::spawn(async move {
        let mut buffer: Vec<f32> = Vec::with_capacity(SILENCE_WINDOW_SAMPLES);
        let mut silence_start: Option<u64> = None;
        let mut last_sample_time: u64 = 0;
        let mut state = RemoteAsrState::default();
        state.set_running(true);

        loop {
            tokio::select! {
                // NEW: Check for stop signal on each iteration
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(50)) => {
                    if STOP_FLAG.load(Ordering::SeqCst) {
                        // Flush any remaining buffer before exiting
                        if !buffer.is_empty() {
                            let i16_samples = f32_to_i16(&buffer);
                            let audio_wav = encode_wav(&i16_samples, 16000);
                            let _ = transcribe_with_retry(&client, &endpoint, &api_key, &audio_wav, language.as_deref()).await;
                        }
                        state.set_running(false);
                        break;
                    }
                }
                Some(chunk) = audio_receiver.recv() => {
                    last_sample_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0);

                    for &sample in chunk.iter() {
                        buffer.push(sample);

                        if buffer.len() >= MAX_CHUNK_SAMPLES {
                            let i16_samples = f32_to_i16(&buffer);
                            buffer.clear();
                            silence_start = None;

                            let audio_wav = encode_wav(&i16_samples, 16000);
                            let text = match transcribe_with_retry(&client, &endpoint, &api_key, &audio_wav, language.as_deref()).await {
                                Ok(t) => t,
                                Err(e) => {
                                    let _ = app.emit("asr-error", AsrErrorPayload { message: e.clone(), retryable: true });
                                    state.set_error(e);
                                    continue;
                                }
                            };

                            if !text.trim().is_empty() {
                                let timestamp = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .map(|d| d.as_millis() as u64)
                                    .unwrap_or(0);
                                let _ = app.emit("subtitle", SubtitlePayload { text: text.trim().to_string(), timestamp, is_final: true });
                                state.add_chunk(text);
                                state.clear_error();
                            }
                        }

                        let window_start = buffer.len().saturating_sub(SILENCE_WINDOW_SAMPLES);
                        let window = &buffer[window_start..];
                        let silence = is_silence(window);

                        if silence {
                            if silence_start.is_none() {
                                silence_start = Some(last_sample_time);
                            }
                        } else {
                            silence_start = None;
                        }

                        if let Some(silence_start_time) = silence_start {
                            if last_sample_time.saturating_sub(silence_start_time) >= SILENCE_DURATION_MS && buffer.len() > SILENCE_WINDOW_SAMPLES {
                                let flush_len = buffer.len().saturating_sub(SILENCE_WINDOW_SAMPLES / 2);
                                let to_send: Vec<f32> = buffer.drain(..flush_len).collect();
                                silence_start = None;

                                let i16_samples = f32_to_i16(&to_send);
                                let audio_wav = encode_wav(&i16_samples, 16000);
                                let text = match transcribe_with_retry(&client, &endpoint, &api_key, &audio_wav, language.as_deref()).await {
                                    Ok(t) => t,
                                    Err(e) => {
                                        let _ = app.emit("asr-error", AsrErrorPayload { message: e.clone(), retryable: true });
                                        state.set_error(e);
                                        continue;
                                    }
                                };

                                if !text.trim().is_empty() {
                                    let timestamp = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .map(|d| d.as_millis() as u64)
                                        .unwrap_or(0);
                                    let _ = app.emit("subtitle", SubtitlePayload { text: text.trim().to_string(), timestamp, is_final: true });
                                    state.add_chunk(text);
                                    state.clear_error();
                                }
                            }
                        }
                    }
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                    if buffer.len() > SILENCE_WINDOW_SAMPLES && silence_start.is_some() {
                        let flush_len = buffer.len().saturating_sub(SILENCE_WINDOW_SAMPLES / 2);
                        let to_send: Vec<f32> = buffer.drain(..flush_len).collect();
                        silence_start = None;

                        let i16_samples = f32_to_i16(&to_send);
                        let audio_wav = encode_wav(&i16_samples, 16000);
                        let text = match transcribe_with_retry(&client, &endpoint, &api_key, &audio_wav, language.as_deref()).await {
                            Ok(t) => t,
                            Err(e) => {
                                let _ = app.emit("asr-error", AsrErrorPayload { message: e.clone(), retryable: true });
                                state.set_error(e);
                                continue;
                            }
                        };

                        if !text.trim().is_empty() {
                            let timestamp = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_millis() as u64)
                                .unwrap_or(0);
                            let _ = app.emit("subtitle", SubtitlePayload { text: text.trim().to_string(), timestamp, is_final: true });
                            state.add_chunk(text);
                            state.clear_error();
                        }
                    }
                    break;
                }
            }
        }

        if !buffer.is_empty() {
            let i16_samples = f32_to_i16(&buffer);
            let audio_wav = encode_wav(&i16_samples, 16000);
            let _ = transcribe_with_retry(&client, &endpoint, &api_key, &audio_wav, language.as_deref()).await;
        }

        state.set_running(false);
    });

    Ok(())
}

pub async fn remote_asr_stop() -> Result<(), String> {
    // Set the stop flag to signal the spawned task to exit
    STOP_FLAG.store(true, Ordering::SeqCst);
    
    // Give the task a moment to process the signal and exit
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    
    // Reset the flag for potential future use
    STOP_FLAG.store(false, Ordering::SeqCst);
    
    Ok(())
}

pub fn remote_asr_status() -> RemoteAsrStatus {
    RemoteAsrStatus {
        is_running: false,
        chunks_accumulated: 0,
        last_transcript: None,
        error: None,
    }
}