use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, Notify};

use base64::Engine;
use tauri::Emitter;

const MAX_RETRIES: u32 = 3;
const REQUEST_TIMEOUT_SECS: u64 = 30;
const SILENCE_THRESHOLD: f32 = 0.005;
const MIN_SPEECH_RMS: f32 = 0.04;
const MIN_SPEECH_RMS_GEMINI: f32 = 0.02;
const MAX_CHUNK_SAMPLES: usize = 48_000;
const SILENCE_WINDOW_SAMPLES: usize = 1024;
const SILENCE_DURATION_MS: u64 = 300;
const NO_AUDIO_TIMEOUT_SECS: u64 = 5;

#[derive(Clone, Copy, Debug)]
enum AsrProvider {
    OpenAiCompatible,
    GeminiBatch,
}

impl AsrProvider {
    fn from_engine(engine: &str) -> Self {
        if engine.eq_ignore_ascii_case("gemini") {
            Self::GeminiBatch
        } else {
            Self::OpenAiCompatible
        }
    }
}

fn min_speech_rms_for_provider(provider: AsrProvider) -> f32 {
    match provider {
        AsrProvider::GeminiBatch => MIN_SPEECH_RMS_GEMINI,
        AsrProvider::OpenAiCompatible => MIN_SPEECH_RMS,
    }
}

#[derive(Clone, serde::Serialize)]
pub struct RemoteAsrStatus {
    pub is_running: bool,
    pub chunks_accumulated: u32,
    pub last_transcript: Option<String>,
    pub error: Option<String>,
}

pub struct RemoteAsrState {
    inner: Arc<Mutex<RemoteAsrStateInner>>,
    stop_signal: Arc<Notify>,
    running_flag: AtomicBool,
}

struct RemoteAsrStateInner {
    chunks_accumulated: u32,
    last_transcript: Option<String>,
    error: Option<String>,
}

impl RemoteAsrState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(RemoteAsrStateInner {
                chunks_accumulated: 0,
                last_transcript: None,
                error: None,
            })),
            stop_signal: Arc::new(Notify::new()),
            running_flag: AtomicBool::new(false),
        }
    }

    pub fn is_running(&self) -> bool {
        self.running_flag.load(Ordering::SeqCst)
    }

    pub fn stop_signal(&self) -> Arc<Notify> {
        self.stop_signal.clone()
    }

    pub async fn status(&self) -> RemoteAsrStatus {
        let inner = self.inner.lock().await;
        RemoteAsrStatus {
            is_running: self.running_flag.load(Ordering::SeqCst),
            chunks_accumulated: inner.chunks_accumulated,
            last_transcript: inner.last_transcript.clone(),
            error: inner.error.clone(),
        }
    }

    fn set_running(&self, running: bool) {
        self.running_flag.store(running, Ordering::SeqCst);
    }

    async fn record_chunk(&self, text: String) {
        let mut inner = self.inner.lock().await;
        inner.chunks_accumulated += 1;
        inner.last_transcript = Some(text);
    }

    async fn set_error(&self, err: String) {
        let mut inner = self.inner.lock().await;
        inner.error = Some(err);
    }

    async fn clear_error(&self) {
        let mut inner = self.inner.lock().await;
        inner.error = None;
    }

    async fn reset(&self) {
        let mut inner = self.inner.lock().await;
        inner.chunks_accumulated = 0;
        inner.last_transcript = None;
        inner.error = None;
    }
}

fn compute_rms(samples: &[f32]) -> f32 {
    let sum: f32 = samples.iter().map(|&s| s * s).sum();
    (sum / samples.len() as f32).sqrt()
}

fn f32_to_i16(samples: &[f32]) -> Vec<i16> {
    samples
        .iter()
        .map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
        .collect()
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
    provider: AsrProvider,
    endpoint: &str,
    api_key: &str,
    audio_bytes: &[u8],
    language: Option<&str>,
    model: &str,
) -> Result<String, String> {
    match provider {
        AsrProvider::OpenAiCompatible => {
            send_openai_compatible_transcription(client, endpoint, api_key, audio_bytes, language, model).await
        }
        AsrProvider::GeminiBatch => {
            send_gemini_batch_transcription(client, endpoint, api_key, audio_bytes, model).await
        }
    }
}

fn resolve_openai_transcription_url(endpoint: &str) -> String {
    let endpoint = endpoint.trim_end_matches('/').to_string();

    if endpoint.ends_with("/v1/audio/transcriptions") || endpoint.contains("/v1/inference/") {
        endpoint
    } else if endpoint.ends_with("/v1") {
        format!("{}/audio/transcriptions", endpoint)
    } else {
        format!("{}/v1/audio/transcriptions", endpoint)
    }
}

fn resolve_gemini_generate_content_url(endpoint: &str, model: &str, api_key: &str) -> String {
    let raw_endpoint = endpoint.trim().trim_end_matches('/');
    let base = match reqwest::Url::parse(raw_endpoint) {
        Ok(parsed) => {
            let path = parsed.path();
            let mut version_path = path.to_string();
            if let Some(pos) = path.to_ascii_lowercase().find("/v1") {
                let mut end = pos + 3; // include /v1
                for ch in path[end..].chars() {
                    if ch.is_ascii_alphanumeric() {
                        end += ch.len_utf8();
                    } else {
                        break;
                    }
                }
                version_path = path[..end].to_string();
            }

            let mut normalized = format!("{}{}", parsed.origin().ascii_serialization(), version_path);
            normalized = normalized.trim_end_matches('/').to_string();
            if normalized.is_empty() {
                raw_endpoint.to_string()
            } else {
                normalized
            }
        }
        Err(_) => raw_endpoint.to_string(),
    };

    format!("{}/models/{}:generateContent?key={}", base, model, api_key)
}

fn redact_gemini_url_for_log(url: &str) -> String {
    if let Some((prefix, _)) = url.split_once("?key=") {
        format!("{}?key=<redacted>", prefix)
    } else {
        url.to_string()
    }
}

fn truncate_for_log(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }

    let mut out = String::new();
    for (idx, ch) in input.chars().enumerate() {
        if idx >= max_chars {
            break;
        }
        out.push(ch);
    }
    out.push_str("...(truncated)");
    out
}

fn extract_gemini_text(body_json: &serde_json::Value) -> String {
    body_json
        .get("candidates")
        .and_then(|c| c.as_array())
        .and_then(|c| c.first())
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.as_array())
        .map(|parts| {
            parts
                .iter()
                .filter_map(|part| part.get("text").and_then(|t| t.as_str()))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default()
}

async fn send_openai_compatible_transcription(
    client: &reqwest::Client,
    endpoint: &str,
    api_key: &str,
    audio_bytes: &[u8],
    language: Option<&str>,
    model: &str,
) -> Result<String, String> {
    log::info!(
        "send_openai_compatible_transcription: audio_bytes={} bytes, language={:?}",
        audio_bytes.len(),
        language
    );

    let audio_part = reqwest::multipart::Part::bytes(audio_bytes.to_vec())
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|e| format!("mime_str error: {}", e))?;

    let mut form = reqwest::multipart::Form::new()
        .part("audio", audio_part)
        .text("model", model.to_string());

    if let Some(lang) = language {
        let whisper_lang = lang.split('-').next().unwrap_or(lang).to_string();
        form = form.text("language", whisper_lang);
    }

    let url = resolve_openai_transcription_url(endpoint);
    log::info!("send_openai_compatible_transcription: POST {}", url);

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .timeout(tokio::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status = resp.status();
    log::info!("send_openai_compatible_transcription: response status={}", status);

    if status == 401 {
        return Err("Invalid API key".to_string());
    }
    if status == 429 {
        if let Some(retry_after) = resp.headers().get("Retry-After") {
            if let Ok(delay_str) = retry_after.to_str() {
                if let Ok(delay_secs) = delay_str.parse::<u64>() {
                    log::warn!(
                        "send_openai_compatible_transcription: rate limited, retrying after {}s",
                        delay_secs
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(delay_secs)).await;
                }
            }
        }
    }
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        log::error!("send_openai_compatible_transcription: API error {}: {}", status, body);
        return Err(format!("API error {}: {}", status, body));
    }

    #[derive(serde::Deserialize)]
    struct Segment {
        #[allow(dead_code)]
        start: f32,
        #[allow(dead_code)]
        end: f32,
        avg_logprob: f32,
    }

    #[derive(serde::Deserialize)]
    struct TranscriptionResponse {
        text: String,
        segments: Vec<Segment>,
    }

    const MIN_AVG_LOGPROB: f32 = -0.8;
    let body_text = resp
        .text()
        .await
        .map_err(|e| format!("read body failed: {}", e))?;
    log::info!(
        "send_openai_compatible_transcription: raw response body: {}",
        body_text
    );
    let transcription: TranscriptionResponse = serde_json::from_str(&body_text)
        .map_err(|e| format!("JSON parse failed: {} (body: {})", e, body_text))?;

    let avg_confidence = transcription
        .segments
        .iter()
        .map(|s| s.avg_logprob)
        .fold(f32::NEG_INFINITY, f32::max);
    if avg_confidence < MIN_AVG_LOGPROB {
        log::info!(
            "send_openai_compatible_transcription: discarding low-confidence result (avg_logprob={:.4}, threshold={})",
            avg_confidence,
            MIN_AVG_LOGPROB
        );
        return Ok(String::new());
    }

    Ok(transcription.text)
}

async fn send_gemini_batch_transcription(
    client: &reqwest::Client,
    endpoint: &str,
    api_key: &str,
    audio_bytes: &[u8],
    model: &str,
) -> Result<String, String> {
    let url = resolve_gemini_generate_content_url(endpoint, model, api_key);
    let loggable_url = redact_gemini_url_for_log(&url);
    log::info!(
        "send_gemini_batch_transcription: POST {} (audio={} bytes, model={})",
        loggable_url,
        audio_bytes.len()
        ,model
    );

    let audio_base64 = base64::engine::general_purpose::STANDARD.encode(audio_bytes);

    let payload = serde_json::json!({
        "contents": [{
            "parts": [
                { "inlineData": { "mimeType": "audio/wav", "data": audio_base64 } },
                { "text": "Transcribe this audio. Return only the transcribed text." }
            ]
        }],
        "generationConfig": {
            "temperature": 0
        }
    });

    let resp = client
        .post(&url)
        .json(&payload)
        .timeout(tokio::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status = resp.status();
    log::info!(
        "send_gemini_batch_transcription: response status={} from {}",
        status,
        loggable_url
    );
    if status == 401 || status == 403 {
        return Err("Invalid API key".to_string());
    }

    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        log::error!("send_gemini_batch_transcription: API error {}: {}", status, body);
        return Err(format!("API error {}: {}", status, body));
    }

    let body_text = resp
        .text()
        .await
        .map_err(|e| format!("read body failed: {}", e))?;
    log::debug!(
        "send_gemini_batch_transcription: raw response body={} ",
        truncate_for_log(&body_text, 2000)
    );
    let body_json: serde_json::Value = serde_json::from_str(&body_text)
        .map_err(|e| format!("JSON parse failed: {} (body: {})", e, body_text))?;

    let text = extract_gemini_text(&body_json);

    if text.trim().is_empty() {
        log::warn!(
            "send_gemini_batch_transcription: extracted empty text; candidate_count={} finish_reason={:?}",
            body_json
                .get("candidates")
                .and_then(|c| c.as_array())
                .map(|c| c.len())
                .unwrap_or(0),
            body_json
                .get("candidates")
                .and_then(|c| c.as_array())
                .and_then(|c| c.first())
                .and_then(|c| c.get("finishReason"))
                .and_then(|f| f.as_str())
        );
    } else {
        log::info!(
            "send_gemini_batch_transcription: extracted text ({} chars)",
            text.trim().chars().count()
        );
    }

    Ok(text.trim().to_string())
}

async fn transcribe_with_retry(
    client: &reqwest::Client,
    provider: AsrProvider,
    endpoint: &str,
    api_key: &str,
    audio_bytes: &[u8],
    language: Option<&str>,
    model: &str,
) -> Result<String, String> {
    for attempt in 0..MAX_RETRIES {
        log::info!(
            "transcribe_with_retry: attempt {}/{} provider={:?} model={}",
            attempt + 1,
            MAX_RETRIES,
            provider,
            model
        );
        match send_transcription(client, provider, endpoint, api_key, audio_bytes, language, model).await {
            Ok(text) => {
                log::info!("transcribe_with_retry: got text ({}) chars", text.len());
                return Ok(text);
            }
            Err(e) if attempt < MAX_RETRIES - 1 => {
                log::warn!(
                    "transcribe_with_retry: attempt {} failed: {}, retrying...",
                    attempt + 1,
                    e
                );
                tokio::time::sleep(tokio::time::Duration::from_millis(
                    200 * (attempt + 1) as u64,
                ))
                .await;
                continue;
            }
            Err(e) => {
                log::error!(
                    "transcribe_with_retry: all {} attempts failed: {}",
                    MAX_RETRIES,
                    e
                );
                return Err(e);
            }
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
    app: tauri::AppHandle,
    state: Arc<RemoteAsrState>,
    mut audio_receiver: mpsc::Receiver<Vec<f32>>,
    endpoint: String,
    api_key: String,
    source_lang: String,
    model: String,
    engine: String,
) -> Result<(), String> {
    // If already running, stop first
    if state.is_running() {
        return Err("Remote ASR is already running".to_string());
    }

    log::info!(
        "remote_asr_start: spawning task, endpoint={}, lang={}, api_key_len={}, engine={}",
        endpoint,
        source_lang,
        api_key.len(),
        engine
    );

    state.set_running(true);
    state.reset().await;

    let client = reqwest::Client::new();
    let provider = AsrProvider::from_engine(&engine);
    let min_speech_rms = min_speech_rms_for_provider(provider);
    let language = Some(source_lang);
    let model = if model.trim().is_empty() {
        match provider {
            AsrProvider::GeminiBatch => "gemini-2.0-flash".to_string(),
            AsrProvider::OpenAiCompatible => "whisper-1".to_string(),
        }
    } else {
        model
    };
    let stop_signal = state.stop_signal();
    let mut chunk_count: u64 = 0;

    log::info!(
        "remote_asr_start: provider={:?}, model={}, min_speech_rms={:.4}",
        provider,
        model,
        min_speech_rms
    );

    tauri::async_runtime::spawn(async move {
        let mut buffer: Vec<f32> = Vec::with_capacity(SILENCE_WINDOW_SAMPLES);
        let mut silence_start: Option<u64> = None;
        let mut last_sample_time: u64 = 0;

        loop {
            let recv_result = tokio::select! {
                biased;

                // Priority 1: check stop signal
                _ = stop_signal.notified() => {
                    log::info!("Remote ASR: stop signal received");
                    break;
                }

                // Priority 2: receive audio data
                chunk = audio_receiver.recv() => {
                    match chunk {
                        Some(data) => {
                            chunk_count += 1;
                            if chunk_count <= 5 || chunk_count % 100 == 0 {
                                log::info!("Remote ASR: received audio chunk #{} ({} samples, buffer={})", chunk_count, data.len(), buffer.len());
                            }
                            Some(data)
                        }
                        None => {
                            log::info!("Remote ASR: audio channel closed after {} chunks", chunk_count);
                            break;
                        }
                    }
                }

                // Priority 3: no-audio timeout (only fires if recv stalls)
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(NO_AUDIO_TIMEOUT_SECS)) => {
                    log::info!("Remote ASR: no audio for {}s, flushing remaining buffer", NO_AUDIO_TIMEOUT_SECS);
                    break;
                }
            };

            let chunk = match recv_result {
                Some(c) => c,
                None => continue,
            };

            last_sample_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);

            for &sample in chunk.iter() {
                buffer.push(sample);

                // Max chunk: flush full buffer
                if buffer.len() >= MAX_CHUNK_SAMPLES {
                    let buffer_rms = compute_rms(&buffer);
                    log::info!(
                        "Remote ASR: max chunk reached, flushing {} samples ({}ms), rms={:.4}",
                        buffer.len(),
                        buffer.len() * 1000 / 16000,
                        buffer_rms
                    );

                    if buffer_rms < min_speech_rms {
                        log::info!("Remote ASR: skipping transcription, buffer rms={:.4} below threshold {:.4}", buffer_rms, min_speech_rms);
                        buffer.clear();
                        silence_start = None;
                        continue;
                    }

                    let i16_samples = f32_to_i16(&buffer);
                    buffer.clear();
                    silence_start = None;

                    let audio_wav = encode_wav(&i16_samples, 16000);
                    let text = match transcribe_with_retry(
                        &client,
                        provider,
                        &endpoint,
                        &api_key,
                        &audio_wav,
                        language.as_deref(),
                        &model,
                    )
                    .await
                    {
                        Ok(t) => t,
                        Err(e) => {
                            let _ = app.emit(
                                "asr-error",
                                AsrErrorPayload {
                                    message: e.clone(),
                                    retryable: true,
                                },
                            );
                            state.set_error(e).await;
                            continue;
                        }
                    };

                    if !text.trim().is_empty() {
                        let timestamp = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_millis() as u64)
                            .unwrap_or(0);
                        log::info!(
                            "Remote ASR: emitting subtitle: {:?} ({} chars)",
                            text.trim(),
                            text.trim().len()
                        );
                        let _ = app.emit(
                            "subtitle",
                            SubtitlePayload {
                                text: text.trim().to_string(),
                                timestamp,
                                is_final: true,
                            },
                        );
                        state.record_chunk(text).await;
                        state.clear_error().await;
                    }
                }

                // Silence detection
                let window_start = buffer.len().saturating_sub(SILENCE_WINDOW_SAMPLES);
                let window = &buffer[window_start..];
                let rms = compute_rms(window);
                let silence = rms < SILENCE_THRESHOLD;

                if silence {
                    if silence_start.is_none() {
                        silence_start = Some(last_sample_time);
                    }
                } else {
                    silence_start = None;
                }

                if let Some(silence_start_time) = silence_start {
                    if last_sample_time.saturating_sub(silence_start_time) >= SILENCE_DURATION_MS
                        && buffer.len() > SILENCE_WINDOW_SAMPLES
                    {
                        let flush_len = buffer.len().saturating_sub(SILENCE_WINDOW_SAMPLES / 2);
                        let to_send: Vec<f32> = buffer.drain(..flush_len).collect();
                        let send_rms = compute_rms(&to_send);
                        log::info!("Remote ASR: silence detected (window rms={:.4}), flushing {} samples ({}ms), buffer rms={:.4}", rms, flush_len, flush_len * 1000 / 16000, send_rms);
                        silence_start = None;

                        if send_rms < min_speech_rms {
                            log::info!("Remote ASR: skipping silence flush, buffer rms={:.4} below threshold {:.4}", send_rms, min_speech_rms);
                            continue;
                        }

                        let i16_samples = f32_to_i16(&to_send);
                        let audio_wav = encode_wav(&i16_samples, 16000);
                        let text = match transcribe_with_retry(
                            &client,
                            provider,
                            &endpoint,
                            &api_key,
                            &audio_wav,
                            language.as_deref(),
                            &model,
                        )
                        .await
                        {
                            Ok(t) => t,
                            Err(e) => {
                                let _ = app.emit(
                                    "asr-error",
                                    AsrErrorPayload {
                                        message: e.clone(),
                                        retryable: true,
                                    },
                                );
                                state.set_error(e).await;
                                continue;
                            }
                        };

                        if !text.trim().is_empty() {
                            let timestamp = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_millis() as u64)
                                .unwrap_or(0);
                            log::info!(
                                "Remote ASR: emitting subtitle (silence flush): {:?} ({} chars)",
                                text.trim(),
                                text.trim().len()
                            );
                            let _ = app.emit(
                                "subtitle",
                                SubtitlePayload {
                                    text: text.trim().to_string(),
                                    timestamp,
                                    is_final: true,
                                },
                            );
                            state.record_chunk(text).await;
                            state.clear_error().await;
                        }
                    }
                }
            }
        }

        // Flush remaining buffer on exit
        if !buffer.is_empty() {
            let exit_rms = compute_rms(&buffer);
            if exit_rms < min_speech_rms {
                log::info!("Remote ASR: discarding {} remaining samples on exit, rms={:.4} below threshold", buffer.len(), exit_rms);
            } else {
                log::info!(
                    "Remote ASR: flushing {} remaining samples on exit, rms={:.4}",
                    buffer.len(),
                    exit_rms
                );
                let i16_samples = f32_to_i16(&buffer);
                let audio_wav = encode_wav(&i16_samples, 16000);
                match transcribe_with_retry(
                    &client,
                    provider,
                    &endpoint,
                    &api_key,
                    &audio_wav,
                    language.as_deref(),
                    &model,
                )
                .await
                {
                    Ok(text) if !text.trim().is_empty() => {
                        let timestamp = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_millis() as u64)
                            .unwrap_or(0);
                        let _ = app.emit(
                            "subtitle",
                            SubtitlePayload {
                                text: text.trim().to_string(),
                                timestamp,
                                is_final: true,
                            },
                        );
                    }
                    Ok(_) => {}
                    Err(e) => log::warn!("Remote ASR: failed to flush final buffer: {}", e),
                }
            }
        }

        state.set_running(false);
        log::info!("Remote ASR: task exited");
    });

    Ok(())
}

pub async fn remote_asr_stop(state: &RemoteAsrState) -> Result<(), String> {
    if !state.is_running() {
        return Ok(());
    }
    state.stop_signal().notify_one();
    state.set_running(false);
    Ok(())
}

pub async fn remote_asr_status(state: &RemoteAsrState) -> RemoteAsrStatus {
    state.status().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_rms_silence() {
        let silence_samples = vec![0.0f32; 1024];
        let rms = compute_rms(&silence_samples);
        assert!(rms < SILENCE_THRESHOLD);
    }

    #[test]
    fn test_compute_rms_speech() {
        let speech_samples = vec![0.5f32, -0.3f32, 0.8f32, -0.6f32, 0.4f32];
        let rms = compute_rms(&speech_samples);
        assert!(rms > SILENCE_THRESHOLD);
    }

    #[test]
    fn test_is_silence_detects_quiet_audio() {
        let silence_samples = vec![0.001f32; 1024];
        assert!(compute_rms(&silence_samples) < SILENCE_THRESHOLD);
    }

    #[test]
    fn test_is_silence_rejects_loud_audio() {
        let loud_samples = vec![0.5f32; 1024];
        assert!(compute_rms(&loud_samples) >= SILENCE_THRESHOLD);
    }

    #[test]
    fn test_f32_to_i16_conversion() {
        let samples = vec![0.0f32, 1.0f32, -1.0f32, 0.5f32, -0.5f32];
        let i16_samples = f32_to_i16(&samples);

        assert_eq!(i16_samples[0], 0i16);
        assert_eq!(i16_samples[1], i16::MAX);
        assert_eq!(i16_samples[2], -32767i16);
        assert_eq!(i16_samples[3], (0.5f32 * i16::MAX as f32) as i16);
        assert_eq!(i16_samples[4], (-0.5f32 * i16::MAX as f32) as i16);
    }

    #[test]
    fn test_f32_to_i16_clamping() {
        let samples = vec![2.0f32, -2.0f32];
        let i16_samples = f32_to_i16(&samples);

        assert_eq!(i16_samples[0], i16::MAX);
        assert_eq!(i16_samples[1], -32767i16);
    }

    #[test]
    fn test_encode_wav_header() {
        let samples = vec![0i16; 16000];
        let wav = encode_wav(&samples, 16000);

        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        assert_eq!(&wav[22..24], &[1u8, 0u8]);
        assert_eq!(&wav[24..28], 16000u32.to_le_bytes());
        assert_eq!(&wav[28..32], 32000u32.to_le_bytes());
        assert_eq!(&wav[36..40], b"data");
    }

    #[test]
    fn test_encode_wav_data_size() {
        let samples: Vec<i16> = (0..16000).map(|i| i as i16).collect();
        let wav = encode_wav(&samples, 16000);

        let expected_data_size = 16000 * 2;
        let actual_data_size = u32::from_le_bytes([wav[40], wav[41], wav[42], wav[43]]) as usize;
        assert_eq!(actual_data_size, expected_data_size);

        let riff_chunk_size = u32::from_le_bytes([wav[4], wav[5], wav[6], wav[7]]) as usize;
        assert_eq!(riff_chunk_size, 36 + expected_data_size);
    }

    #[test]
    fn test_encode_wav_single_channel() {
        let samples = vec![0i16; 1000];
        let wav = encode_wav(&samples, 16000);

        let num_channels = u16::from_le_bytes([wav[22], wav[23]]);
        assert_eq!(num_channels, 1);
    }

    #[test]
    fn test_encode_wav_16bit() {
        let samples = vec![0i16; 1000];
        let wav = encode_wav(&samples, 16000);

        let bits_per_sample = u16::from_le_bytes([wav[34], wav[35]]);
        assert_eq!(bits_per_sample, 16);
    }

    #[tokio::test]
    async fn test_remote_asr_state_running() {
        let state = Arc::new(RemoteAsrState::new());
        assert!(!state.is_running());

        state.set_running(true);
        assert!(state.is_running());

        let status = state.status().await;
        assert!(status.is_running);
    }

    #[tokio::test]
    async fn test_remote_asr_state_record_chunk() {
        let state = Arc::new(RemoteAsrState::new());
        state.record_chunk("Hello".to_string()).await;

        let status = state.status().await;
        assert_eq!(status.chunks_accumulated, 1);
        assert_eq!(status.last_transcript, Some("Hello".to_string()));
    }

    #[tokio::test]
    async fn test_remote_asr_state_error_handling() {
        let state = Arc::new(RemoteAsrState::new());
        state.set_error("API error".to_string()).await;

        let status = state.status().await;
        assert_eq!(status.error, Some("API error".to_string()));

        state.clear_error().await;
        let status = state.status().await;
        assert!(status.error.is_none());
    }

    #[tokio::test]
    async fn test_remote_asr_state_reset() {
        let state = Arc::new(RemoteAsrState::new());
        state.set_running(true);
        state.record_chunk("test".to_string()).await;
        state.set_error("err".to_string()).await;

        state.reset().await;

        let status = state.status().await;
        assert_eq!(status.chunks_accumulated, 0);
        assert!(status.last_transcript.is_none());
        assert!(status.error.is_none());
    }

    #[test]
    fn test_silence_threshold_constant() {
        assert_eq!(SILENCE_THRESHOLD, 0.005);
        assert_eq!(MIN_SPEECH_RMS, 0.04);
        assert_eq!(MIN_SPEECH_RMS_GEMINI, 0.02);
    }

    #[test]
    fn test_max_chunk_samples_constant() {
        assert_eq!(MAX_CHUNK_SAMPLES, 48_000);
    }

    #[test]
    fn test_silence_window_samples_constant() {
        assert_eq!(SILENCE_WINDOW_SAMPLES, 1024);
    }

    #[test]
    fn test_silence_duration_ms_constant() {
        assert_eq!(SILENCE_DURATION_MS, 300);
    }

    // --- VAD: MIN_SPEECH_RMS gate tests ---

    #[test]
    fn test_vad_rejects_silence() {
        // Pure digital silence
        let samples = vec![0.0f32; 48000];
        assert!(compute_rms(&samples) < MIN_SPEECH_RMS);
    }

    #[test]
    fn test_vad_rejects_low_noise() {
        // Background noise around RMS 0.03 (the "Okay." / "." cases from logs)
        let samples: Vec<f32> = (0..48000)
            .map(|i| 0.03 * ((i as f32 * 0.1).sin()))
            .collect();
        let rms = compute_rms(&samples);
        assert!(
            rms < MIN_SPEECH_RMS,
            "RMS {} should be below MIN_SPEECH_RMS {}",
            rms,
            MIN_SPEECH_RMS
        );
    }

    #[test]
    fn test_vad_accepts_quiet_speech() {
        // Quiet speech that should pass the gate (amplitude ~0.06 → RMS ~0.042)
        let samples: Vec<f32> = (0..48000)
            .map(|i| 0.06 * (440.0 * i as f32 / 16000.0 * 2.0 * std::f32::consts::PI).sin())
            .collect();
        let rms = compute_rms(&samples);
        assert!(
            rms >= MIN_SPEECH_RMS,
            "RMS {} should be >= MIN_SPEECH_RMS {}",
            rms,
            MIN_SPEECH_RMS
        );
    }

    #[test]
    fn test_vad_accepts_normal_speech() {
        // Normal speech around RMS 0.1+
        let samples: Vec<f32> = (0..48000)
            .map(|i| 0.2 * (440.0 * i as f32 / 16000.0 * 2.0 * std::f32::consts::PI).sin())
            .collect();
        let rms = compute_rms(&samples);
        assert!(rms >= MIN_SPEECH_RMS);
    }

    // --- Logprob filter tests ---

    #[test]
    fn test_logprob_filter_rejects_hallucination() {
        // "Okay." from logs had avg_logprob=-1.05, "So" had -1.68
        #[derive(serde::Deserialize)]
        struct Segment {
            avg_logprob: f32,
        }
        #[derive(serde::Deserialize)]
        struct Response {
            text: String,
            segments: Vec<Segment>,
        }

        let body = r#"{"text":"Okay.","segments":[{"id":0,"seek":0,"start":0.0,"end":3.0,"text":" Okay.","avg_logprob":-1.05}]}"#;
        let resp: Response = serde_json::from_str(body).unwrap();
        let max_logprob = resp
            .segments
            .iter()
            .map(|s| s.avg_logprob)
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(
            max_logprob < -0.8,
            "hallucination logprob {} should be below -0.8",
            max_logprob
        );
    }

    #[test]
    fn test_logprob_filter_rejects_gibberish() {
        // "So" from logs had avg_logprob=-1.68
        #[derive(serde::Deserialize)]
        struct Segment {
            avg_logprob: f32,
        }
        #[derive(serde::Deserialize)]
        struct Response {
            text: String,
            segments: Vec<Segment>,
        }

        let body = r#"{"text":"So","segments":[{"id":0,"seek":0,"start":0.0,"end":3.0,"text":" So","avg_logprob":-1.68}]}"#;
        let resp: Response = serde_json::from_str(body).unwrap();
        let max_logprob = resp
            .segments
            .iter()
            .map(|s| s.avg_logprob)
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(
            max_logprob < -0.8,
            "gibberish logprob {} should be below -0.8",
            max_logprob
        );
    }

    #[test]
    fn test_logprob_filter_accepts_real_speech() {
        // Real transcription typically has avg_logprob > -0.5
        #[derive(serde::Deserialize)]
        struct Segment {
            avg_logprob: f32,
        }
        #[derive(serde::Deserialize)]
        struct Response {
            text: String,
            segments: Vec<Segment>,
        }

        let body = r#"{"text":"Open source real time caption","segments":[{"id":0,"seek":0,"start":0.0,"end":3.0,"text":" Open source real time caption","avg_logprob":-0.35}]}"#;
        let resp: Response = serde_json::from_str(body).unwrap();
        let max_logprob = resp
            .segments
            .iter()
            .map(|s| s.avg_logprob)
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(
            max_logprob >= -0.8,
            "real speech logprob {} should be >= -0.8",
            max_logprob
        );
    }

    #[test]
    fn test_logprob_filter_boundary() {
        // Exactly at the boundary
        #[derive(serde::Deserialize)]
        struct Segment {
            avg_logprob: f32,
        }
        #[derive(serde::Deserialize)]
        struct Response {
            text: String,
            segments: Vec<Segment>,
        }

        let body = r#"{"text":"test","segments":[{"avg_logprob":-0.8}]}"#;
        let resp: Response = serde_json::from_str(body).unwrap();
        let max_logprob = resp
            .segments
            .iter()
            .map(|s| s.avg_logprob)
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(max_logprob >= -0.8, "boundary logprob should pass");
    }

    #[test]
    fn test_provider_from_engine() {
        assert!(matches!(
            AsrProvider::from_engine("gemini"),
            AsrProvider::GeminiBatch
        ));
        assert!(matches!(
            AsrProvider::from_engine("GEMINI"),
            AsrProvider::GeminiBatch
        ));
        assert!(matches!(
            AsrProvider::from_engine("remote"),
            AsrProvider::OpenAiCompatible
        ));
    }

    #[test]
    fn test_resolve_gemini_generate_content_url_from_v1beta_openai_path() {
        let url = resolve_gemini_generate_content_url(
            "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions",
            "gemini-2.0-flash",
            "AIza-test",
        );
        assert_eq!(
            url,
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key=AIza-test"
        );
    }

    #[test]
    fn test_resolve_gemini_generate_content_url_from_v1beta_base() {
        let url = resolve_gemini_generate_content_url(
            "https://generativelanguage.googleapis.com/v1beta",
            "gemini-2.0-flash",
            "AIza-test",
        );
        assert_eq!(
            url,
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key=AIza-test"
        );
    }

    #[test]
    fn test_resolve_gemini_generate_content_url_from_bare_host() {
        let url = resolve_gemini_generate_content_url(
            "https://generativelanguage.googleapis.com",
            "gemini-2.0-flash",
            "AIza-test",
        );
        assert_eq!(
            url,
            "https://generativelanguage.googleapis.com/models/gemini-2.0-flash:generateContent?key=AIza-test"
        );
    }

    #[test]
    fn test_extract_gemini_text_from_candidate_parts() {
        let body = serde_json::json!({
            "candidates": [
                {
                    "content": {
                        "parts": [
                            { "text": "hello" },
                            { "text": "world" }
                        ]
                    }
                }
            ]
        });
        assert_eq!(extract_gemini_text(&body), "hello world");
    }

    #[test]
    fn test_extract_gemini_text_handles_missing_candidates() {
        let body = serde_json::json!({ "candidates": [] });
        assert_eq!(extract_gemini_text(&body), "");
    }
}
