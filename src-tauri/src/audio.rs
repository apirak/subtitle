use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SampleRate, StreamConfig};
use rubato::{Fft, FixedSync, Resampler};
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
            .description()
            .unwrap_or_else(|_| "unknown".to_string())
    );

    // Try to get a 16kHz mono f32 config first, fall back to native
    let config = find_16khz_mono_config(&device)
        .unwrap_or_else(|_| find_native_config(&device)?);

    let native_rate = config.sample_rate().0;
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

    // Set up resampler if needed (outside the callback to avoid allocation inside it)
    let resampler: Option<Mutex<Fft<f32>>> = if needs_resampling {
        let chunk = calculate_fft_chunk_size(native_rate as usize, TARGET_SAMPLE_RATE as usize);
        let resamp = Fft::new(
            native_rate as usize,
            TARGET_SAMPLE_RATE as usize,
            chunk,
            1,
            TARGET_CHANNELS as usize,
            FixedSync::Input,
        )
        .map_err(|e| format!("Failed to create resampler: {}", e))?;
        Some(Mutex::new(resamp))
    } else {
        None
    };

    // Resampler input buffer: accumulates mono samples until rubato's required input size
    let resampler_input: Option<Mutex<Vec<f32>>> = if needs_resampling {
        Some(Mutex::new(Vec::new()))
    } else {
        None
    };

    let sample_format = config.sample_format();
    let config: StreamConfig = config.into();

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
                        &resampler,
                        &resampler_input,
                        &tx,
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
                    let f32_data: Vec<f32> = data.iter().map(|s| s.to_f32()).collect();
                    process_audio_chunk(
                        &f32_data,
                        native_channels,
                        needs_downmix,
                        needs_resampling,
                        &accumulator,
                        &resampler,
                        &resampler_input,
                        &tx,
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
                    let f32_data: Vec<f32> = data.iter().map(|s| s.to_f32()).collect();
                    process_audio_chunk(
                        &f32_data,
                        native_channels,
                        needs_downmix,
                        needs_resampling,
                        &accumulator,
                        &resampler,
                        &resampler_input,
                        &tx,
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
    // Drop the stream (stops capture), sender, and receiver
    *state.capture.lock().map_err(|e| e.to_string())? = None;
    *state.sender.lock().map_err(|e| e.to_string())? = None;
    *state.receiver.lock().map_err(|e| e.to_string())? = None;
    log::info!("Audio capture stopped");
    Ok(())
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
    resampler: &Option<Mutex<Fft<f32>>>,
    resampler_input: &Option<Mutex<Vec<f32>>>,
    tx: &mpsc::Sender<Vec<f32>>,
) {
    // Step 1: Downmix to mono if needed
    let mono = if needs_downmix {
        downmix_to_mono(data, native_channels)
    } else {
        data.to_vec()
    };

    // Step 2: Resample if needed
    let samples = if needs_resampling {
        if let Some(resamp_mutex) = resampler {
            if let Some(input_buf_mutex) = resampler_input {
                let mut resamp = match resamp_mutex.lock() {
                    Ok(g) => g,
                    Err(_) => return,
                };
                let mut input_buf = match input_buf_mutex.lock() {
                    Ok(g) => g,
                    Err(_) => return,
                };

                input_buf.extend_from_slice(&mono);

                let mut output = Vec::new();
                let needed = resamp.input_frames_next();

                while input_buf.len() >= needed {
                    let chunk: Vec<f32> = input_buf.drain(..needed).collect();
                    // rubato v2 expects Vec<Vec<f32>> (one vec per channel)
                    let waves_in = vec![chunk];
                    let max_out = resamp.output_frames_max();
                    let mut waves_out = vec![vec![0.0f32; max_out]];

                    use rubato::Adapter;
                    let input = rubato::SequentialSliceOfVecs::new(
                        &waves_in,
                        1,
                        needed,
                    )
                    .unwrap();
                    let mut output_adapter =
                        rubato::SequentialSliceOfVecs::new_mut(&mut waves_out, 1, max_out)
                            .unwrap();

                    match resamp.process_into_buffer(&input, &mut output_adapter, None) {
                        Ok((_, out_frames)) => {
                            output.extend_from_slice(&waves_out[0][..out_frames]);
                        }
                        Err(e) => {
                            log::error!("Resampling error: {}", e);
                            return;
                        }
                    }
                }

                output
            } else {
                mono
            }
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
        if range.min_sample_rate() <= SampleRate(TARGET_SAMPLE_RATE)
            && SampleRate(TARGET_SAMPLE_RATE) <= range.max_sample_rate()
            && range.channels() == TARGET_CHANNELS
            && range.sample_format() == SampleFormat::F32
        {
            return Ok(range.with_sample_rate(SampleRate(TARGET_SAMPLE_RATE)));
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

/// Calculate a reasonable FFT chunk size for the resampler.
/// Uses GCD-based approach for clean integer ratios.
fn calculate_fft_chunk_size(input_rate: usize, output_rate: usize) -> usize {
    use std::cmp::min;
    let gcd = gcd(input_rate, output_rate);
    let ratio_in = input_rate / gcd;
    let ratio_out = output_rate / gcd;
    // Scale up for reasonable FFT size (at least 256, at most 4096)
    let multiplier = min(4096 / (ratio_in.max(ratio_out)).max(1), 16).max(1);
    ratio_in * multiplier
}

fn gcd(a: usize, b: usize) -> usize {
    let mut a = a;
    let mut b = b;
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

use cpal::Sample;
use rubato::audioadapter::Adapter;
