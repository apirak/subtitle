use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig};
use std::sync::Mutex;
use tokio::sync::mpsc;

const TARGET_SAMPLE_RATE: u32 = 16000;
const TARGET_CHANNELS: u16 = 1;
const CHUNK_DURATION_MS: u32 = 30;
const CHUNK_SIZE: usize = (TARGET_SAMPLE_RATE * CHUNK_DURATION_MS / 1000) as usize; // 480
const CHANNEL_CAPACITY: usize = 32;

/// Holds the active audio stream. Dropping this stops capture.
pub struct AudioCapture {
    #[allow(dead_code)]
    stream: cpal::Stream,
}

/// Shared state managed by Tauri. Allows start/stop commands to coordinate.
pub struct AudioState {
    capture: Mutex<Option<AudioCapture>>,
    sender: Mutex<Option<mpsc::Sender<Vec<f32>>>>,
    receiver: Mutex<Option<mpsc::Receiver<Vec<f32>>>>,
}

impl AudioState {
    pub fn new() -> Self {
        Self {
            capture: Mutex::new(None),
            sender: Mutex::new(None),
            receiver: Mutex::new(None),
        }
    }
}

/// Start capturing audio from the default microphone.
///
/// Captures at the device's native sample rate, downmixes to mono,
/// resamples to 16kHz if needed, and delivers 480-sample (30ms) chunks
/// through an mpsc channel.
pub fn start_capture(state: &AudioState) -> Result<u32, String> {
    // Check if already capturing
    {
        let capture = state.capture.lock().map_err(|e| e.to_string())?;
        if capture.is_some() {
            return Err("Audio capture is already running".to_string());
        }
    }

    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or("No input device available".to_string())?;

    log::info!(
        "Using input device: {}",
        device
            .name()
            .unwrap_or_else(|_| "unknown".to_string())
    );

    // Try to get a 16kHz mono f32 config first, fall back to native
    let config = match find_16khz_mono_config(&device) {
        Ok(c) => c,
        Err(_) => find_native_config(&device)?,
    };

    let native_rate = config.sample_rate();
    let native_channels = config.channels();
    let needs_resampling = native_rate != TARGET_SAMPLE_RATE;
    let needs_downmix = native_channels > 1;

    log::info!(
        "Audio config: {}Hz, {}ch, format {:?} (resample: {}, downmix: {})",
        native_rate,
        native_channels,
        config.sample_format(),
        needs_resampling,
        needs_downmix
    );

    // Clamp config values to valid ranges (threat model T-2-01)
    assert!(
        (8000..=192000).contains(&native_rate),
        "Invalid sample rate: {}",
        native_rate
    );
    assert!(
        (1..=8).contains(&native_channels),
        "Invalid channel count: {}",
        native_channels
    );

    let (tx, rx) = mpsc::channel::<Vec<f32>>(CHANNEL_CAPACITY);

    // Pre-allocate accumulator buffer
    let accumulator = Mutex::new(Vec::<f32>::with_capacity(CHUNK_SIZE * 4));

    // Resample ratio for linear interpolation (only used if needs_resampling)
    let resample_ratio = if needs_resampling {
        native_rate as f64 / TARGET_SAMPLE_RATE as f64
    } else {
        1.0
    };
    let resample_info = if needs_resampling {
        Some(Mutex::new(ResampleState {
            ratio: resample_ratio,
            position: 0.0,
        }))
    } else {
        None
    };

    let sample_format = config.sample_format();
    let config: StreamConfig = config.into();

    let tx_callback = tx.clone();

    let stream = match sample_format {
        SampleFormat::F32 => device
            .build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    process_audio_chunk(
                        data,
                        native_channels,
                        needs_downmix,
                        needs_resampling,
                        &accumulator,
                        &resample_info,
                        &tx_callback,
                    );
                },
                |err| log::error!("Audio stream error: {}", err),
                None,
            )
            .map_err(|e| format!("Failed to build stream: {}", e))?,
        SampleFormat::I16 => device
            .build_input_stream(
                &config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let f32_data: Vec<f32> = data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                    process_audio_chunk(
                        &f32_data,
                        native_channels,
                        needs_downmix,
                        needs_resampling,
                        &accumulator,
                        &resample_info,
                        &tx_callback,
                    );
                },
                |err| log::error!("Audio stream error: {}", err),
                None,
            )
            .map_err(|e| format!("Failed to build stream: {}", e))?,
        SampleFormat::I32 => device
            .build_input_stream(
                &config,
                move |data: &[i32], _: &cpal::InputCallbackInfo| {
                    let f32_data: Vec<f32> = data.iter().map(|&s| s as f32 / i32::MAX as f32).collect();
                    process_audio_chunk(
                        &f32_data,
                        native_channels,
                        needs_downmix,
                        needs_resampling,
                        &accumulator,
                        &resample_info,
                        &tx_callback,
                    );
                },
                |err| log::error!("Audio stream error: {}", err),
                None,
            )
            .map_err(|e| format!("Failed to build stream: {}", e))?,
        _ => return Err(format!("Unsupported sample format: {:?}", sample_format)),
    };

    stream
        .play()
        .map_err(|e| format!("Failed to start stream: {}", e))?;

    *state.capture.lock().map_err(|e| e.to_string())? = Some(AudioCapture { stream });
    *state.sender.lock().map_err(|e| e.to_string())? = Some(tx);
    *state.receiver.lock().map_err(|e| e.to_string())? = Some(rx);

    log::info!("Audio capture started successfully");
    Ok(TARGET_SAMPLE_RATE)
}

/// Stop capturing audio.
pub fn stop_capture(state: &AudioState) -> Result<(), String> {
    *state.capture.lock().map_err(|e| e.to_string())? = None;
    *state.sender.lock().map_err(|e| e.to_string())? = None;
    *state.receiver.lock().map_err(|e| e.to_string())? = None;
    log::info!("Audio capture stopped");
    Ok(())
}

/// State for linear interpolation resampling.
struct ResampleState {
    ratio: f64,      // from_rate / to_rate
    position: f64,   // fractional position in input stream
}

/// Process a chunk of audio data from the cpal callback.
///
/// This runs on the audio thread. It downmixes to mono, resamples if needed,
/// accumulates samples into CHUNK_SIZE chunks, and sends them via try_send.
fn process_audio_chunk(
    data: &[f32],
    native_channels: u16,
    needs_downmix: bool,
    needs_resampling: bool,
    accumulator: &Mutex<Vec<f32>>,
    resample_info: &Option<Mutex<ResampleState>>,
    tx: &mpsc::Sender<Vec<f32>>,
) {
    // Step 1: Downmix to mono if needed
    let mono = if needs_downmix {
        downmix_to_mono(data, native_channels)
    } else {
        data.to_vec()
    };

    // Step 2: Resample if needed using linear interpolation
    let samples = if needs_resampling {
        if let Some(info_mutex) = resample_info {
            let mut info = match info_mutex.lock() {
                Ok(g) => g,
                Err(_) => return,
            };
            resample_linear(&mono, &mut info)
        } else {
            mono
        }
    } else {
        mono
    };

    // Step 3: Accumulate and send in CHUNK_SIZE chunks
    let mut acc = match accumulator.lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    acc.extend_from_slice(&samples);
    while acc.len() >= CHUNK_SIZE {
        let chunk: Vec<f32> = acc.drain(..CHUNK_SIZE).collect();
        if tx.try_send(chunk).is_err() {
            // Channel full -- drop chunk (real-time constraint)
        }
    }
}

/// Linear interpolation resampling. Good enough for ASR preprocessing.
fn resample_linear(input: &[f32], state: &mut ResampleState) -> Vec<f32> {
    if input.is_empty() {
        return Vec::new();
    }

    // Estimate output size based on ratio
    let output_len = ((input.len() as f64 / state.ratio) * 1.1) as usize + 1;
    let mut output = Vec::with_capacity(output_len);

    let input_len = input.len() as f64;

    while state.position < input_len {
        let idx = state.position as usize;
        let frac = state.position - idx as f64;

        let sample = if idx + 1 < input.len() {
            input[idx] * (1.0 - frac as f32) + input[idx + 1] * frac as f32
        } else {
            input[idx.min(input.len() - 1)]
        };
        output.push(sample);
        state.position += state.ratio;
    }

    // Carry over fractional position for next call
    state.position -= input_len;

    output
}

/// Downmix interleaved multi-channel audio to mono by averaging channels per frame.
fn downmix_to_mono(data: &[f32], channels: u16) -> Vec<f32> {
    if channels <= 1 {
        return data.to_vec();
    }
    let ch = channels as usize;
    let frames = data.len() / ch;
    (0..frames)
        .map(|i| {
            let frame_start = i * ch;
            let sum: f32 = (0..ch).map(|c| data[frame_start + c]).sum();
            sum / ch as f32
        })
        .collect()
}

/// Try to find a device config that supports 16kHz mono f32 natively.
fn find_16khz_mono_config(device: &cpal::Device) -> Result<cpal::SupportedStreamConfig, String> {
    let configs = device
        .supported_input_configs()
        .map_err(|e| format!("Failed to query device configs: {}", e))?;

    for range in configs {
        if range.min_sample_rate() <= TARGET_SAMPLE_RATE
            && TARGET_SAMPLE_RATE <= range.max_sample_rate()
            && range.channels() == TARGET_CHANNELS
            && range.sample_format() == SampleFormat::F32
        {
            return Ok(range.with_sample_rate(TARGET_SAMPLE_RATE));
        }
    }
    Err(format!(
        "Device does not support {}Hz mono f32",
        TARGET_SAMPLE_RATE
    ))
}

/// Fall back to the device's default input config.
fn find_native_config(device: &cpal::Device) -> Result<cpal::SupportedStreamConfig, String> {
    device
        .default_input_config()
        .map_err(|e| format!("Failed to get default input config: {}", e))
}
