import { useState, useEffect, useRef, useCallback } from 'preact/hooks';
import type { AppStatus, SubtitleLine, WorkerToMainMessage } from './types';

const CHUNK_DURATION_S = 3;
const TARGET_SAMPLE_RATE = 16000;
const MAX_SUBTITLES = 12;

function resample(audioData: Float32Array, fromRate: number, toRate: number): Float32Array {
  if (fromRate === toRate) return audioData;
  const ratio = fromRate / toRate;
  const newLength = Math.round(audioData.length / ratio);
  const result = new Float32Array(newLength);
  for (let i = 0; i < newLength; i++) {
    const srcIndex = i * ratio;
    const low = Math.floor(srcIndex);
    const high = Math.min(low + 1, audioData.length - 1);
    const frac = srcIndex - low;
    result[i] = audioData[low] * (1 - frac) + audioData[high] * frac;
  }
  return result;
}

export function useSpeechToText() {
  const [status, setStatus] = useState<AppStatus>('idle');
  const [subtitles, setSubtitles] = useState<SubtitleLine[]>([]);
  const [loadProgress, setLoadProgress] = useState(0);

  const workerRef = useRef<Worker | null>(null);
  const audioContextRef = useRef<AudioContext | null>(null);
  const sourceRef = useRef<MediaStreamAudioSourceNode | null>(null);
  const processorRef = useRef<ScriptProcessorNode | null>(null);
  const streamRef = useRef<MediaStream | null>(null);
  const bufferRef = useRef<Float32Array[]>([]);
  const isReadyRef = useRef(false);
  const chunkTimerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const flushBuffer = useCallback(() => {
    if (!workerRef.current || !isReadyRef.current) return;
    const chunks = bufferRef.current;
    if (chunks.length === 0) return;

    const totalLength = chunks.reduce((sum, c) => sum + c.length, 0);
    if (totalLength === 0) return;

    const merged = new Float32Array(totalLength);
    let offset = 0;
    for (const chunk of chunks) {
      merged.set(chunk, offset);
      offset += chunk.length;
    }
    bufferRef.current = [];

    const audioCtx = audioContextRef.current;
    const nativeSampleRate = audioCtx?.sampleRate ?? TARGET_SAMPLE_RATE;
    const resampled = resample(merged, nativeSampleRate, TARGET_SAMPLE_RATE);

    workerRef.current.postMessage({ type: 'transcribe', audio: resampled });
  }, []);

  const handleWorkerMessage = useCallback((e: MessageEvent<WorkerToMainMessage>) => {
    const msg = e.data;
    switch (msg.status) {
      case 'loading':
        setLoadProgress(Math.round(msg.progress));
        break;
      case 'ready':
        isReadyRef.current = true;
        setStatus('listening');
        break;
      case 'processing':
        break;
      case 'result':
        setSubtitles((prev) => {
          const next = [
            ...prev,
            { id: `${Date.now()}-${Math.random()}`, text: msg.text, timestamp: Date.now() },
          ];
          return next.slice(-MAX_SUBTITLES);
        });
        setStatus('listening');
        break;
      case 'error':
        console.error('Worker error:', msg.message);
        setStatus('error');
        break;
    }
  }, []);

  const startSession = useCallback(async () => {
    try {
      setStatus('loading');
      setLoadProgress(0);
      setSubtitles([]);
      isReadyRef.current = false;

      // Create worker
      const worker = new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' });
      worker.addEventListener('message', handleWorkerMessage);
      workerRef.current = worker;

      // Start model loading
      worker.postMessage({ type: 'init' });

      // Request mic access
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      streamRef.current = stream;

      // Setup audio capture
      const audioCtx = new AudioContext();
      audioContextRef.current = audioCtx;

      const source = audioCtx.createMediaStreamSource(stream);
      sourceRef.current = source;

      const processor = audioCtx.createScriptProcessor(4096, 1, 1);
      processorRef.current = processor;

      processor.onaudioprocess = (event) => {
        const inputData = event.inputBuffer.getChannelData(0);
        bufferRef.current.push(new Float32Array(inputData));
      };

      source.connect(processor);
      processor.connect(audioCtx.destination);

      // Flush buffer at regular intervals
      chunkTimerRef.current = setInterval(flushBuffer, CHUNK_DURATION_S * 1000);
    } catch (err) {
      console.error('Failed to start session:', err);
      setStatus('error');
    }
  }, [flushBuffer, handleWorkerMessage]);

  const stopSession = useCallback(() => {
    // Stop timer
    if (chunkTimerRef.current) {
      clearInterval(chunkTimerRef.current);
      chunkTimerRef.current = null;
    }

    // Disconnect audio nodes
    processorRef.current?.disconnect();
    sourceRef.current?.disconnect();
    processorRef.current = null;
    sourceRef.current = null;

    // Close audio context
    audioContextRef.current?.close();
    audioContextRef.current = null;

    // Stop mic tracks
    streamRef.current?.getTracks().forEach((t) => t.stop());
    streamRef.current = null;

    // Terminate worker
    workerRef.current?.terminate();
    workerRef.current = null;

    // Reset
    isReadyRef.current = false;
    bufferRef.current = [];
    setStatus('idle');
  }, []);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      stopSession();
    };
  }, [stopSession]);

  return { status, subtitles, loadProgress, startSession, stopSession };
}
