import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { AppStatus, SubtitleLine } from './types';

const MAX_SUBTITLES = 12;

interface SpeechRecognitionInstance extends EventTarget {
  continuous: boolean;
  interimResults: boolean;
  lang: string;
  start(): void;
  stop(): void;
  onresult: ((event: SpeechRecognitionEvent) => void) | null;
  onerror: ((event: SpeechRecognitionErrorEvent) => void) | null;
  onend: (() => void) | null;
}

interface SpeechRecognitionEvent {
  resultIndex: number;
  results: SpeechRecognitionResultList;
}

interface SpeechRecognitionResultList {
  length: number;
  [index: number]: SpeechRecognitionResult;
}

interface SpeechRecognitionResult {
  isFinal: boolean;
  [index: number]: { transcript: string };
}

interface SpeechRecognitionErrorEvent {
  error: string;
  message: string;
}

function createRecognition(): SpeechRecognitionInstance | null {
  const SR = (window as any).SpeechRecognition ?? (window as any).webkitSpeechRecognition;
  if (!SR) return null;
  return new SR();
}

class Speech {
  status = $state<AppStatus>('idle');
  subtitles = $state<SubtitleLine[]>([]);
  language = $state('en-US');
  errorMessage = $state('');

  private recognition: SpeechRecognitionInstance | null = null;
  private stopping = false;

  private unlistenUpdate?: UnlistenFn;
  private unlistenFinal?: UnlistenFn;
  private unlistenTranslated?: UnlistenFn;
  private unlistenError?: UnlistenFn;

  constructor() {
    this.setupEventListeners();
  }

  private setupEventListeners = async () => {
    this.unlistenUpdate = await listen<{ id: string; text: string; is_final: boolean; timestamp: number }>(
      'backend://subtitle/update',
      (event) => {
        const payload = event.payload;
        if (!payload.is_final) {
          this.addInterimSubtitle(payload.id, payload.text);
        }
      }
    );

    this.unlistenFinal = await listen<{ id: string; text: string; is_final: boolean; timestamp: number }>(
      'backend://subtitle/final',
      (event) => {
        const payload = event.payload;
        this.addFinalSubtitle(payload.id, payload.text);
      }
    );

    this.unlistenTranslated = await listen<{ id: string; original: string; translated: string; timestamp: number }>(
      'backend://subtitle/translated',
      (event) => {
        const payload = event.payload;
        this.setTranslation(payload.id, payload.original, payload.translated);
      }
    );

    this.unlistenError = await listen<{ code: string; message: string }>(
      'backend://subtitle/error',
      (event) => {
        const payload = event.payload;
        this.setError(`${payload.code}: ${payload.message}`);
      }
    );
  };

  addInterimSubtitle = (id: string, text: string) => {
    const interimId = `interim-${id}`;
    this.subtitles = appendSubtitle(this.subtitles, null, {
      id: interimId,
      text,
      timestamp: Date.now(),
    });
  };

  addFinalSubtitle = (id: string, text: string) => {
    // Remove any interim subtitle with the same base id
    const toRemove = [...this.subtitles].find((s) => s.id === `interim-${id}`)?.id;
    this.subtitles = appendSubtitle(this.subtitles, toRemove ?? null, {
      id,
      text,
      timestamp: Date.now(),
    });
  };

  setTranslation = (_id: string, _original: string, translated: string) => {
    // Translation storage is managed by app.svelte
    // This listener exists for future direct backend translation integration
  };

  setError = (message: string) => {
    this.errorMessage = message;
    this.status = 'error';
  };

  destroy = () => {
    this.unlistenUpdate?.();
    this.unlistenFinal?.();
    this.unlistenTranslated?.();
    this.unlistenError?.();
  };

  startCapture = async (): Promise<{ sample_rate: number; channels: number }> => {
    try {
      const result = await invoke<{ sample_rate: number; channels: number }>('audio_capture_start');
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      this.setError(`Audio capture failed: ${message}`);
      throw err;
    }
  };

  stopCapture = async (): Promise<void> => {
    try {
      await invoke('audio_capture_stop');
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      console.error('Audio capture stop error:', message);
    }
  };

  start = async () => {
    // Start Rust audio capture (microphone -> 16kHz mono chunks)
    try {
      await this.startCapture();
    } catch {
      // Error already set by startCapture via setError
      return;
    }

    const recognition = createRecognition();
    if (!recognition) {
      this.errorMessage = 'Web Speech API is not supported in this browser. Try Safari or Chrome.';
      this.status = 'error';
      return;
    }

    this.subtitles = [];
    this.errorMessage = '';
    this.stopping = false;
    this.status = 'listening';

    recognition.continuous = true;
    recognition.interimResults = true;
    recognition.lang = this.language;

    let interimId: string | null = null;
    let hasErrored = false;

    recognition.onresult = (event) => {
      for (let i = event.resultIndex; i < event.results.length; i++) {
        const result = event.results[i];
        const transcript = result[0].transcript.trim();
        if (!transcript) continue;

        if (result.isFinal) {
          const toRemove = interimId;
          interimId = null;
          this.subtitles = appendSubtitle(this.subtitles, toRemove, {
            id: `${Date.now()}-${Math.random()}`,
            text: transcript,
            timestamp: Date.now(),
          });
        } else {
          const newId = `interim-${Date.now()}`;
          this.subtitles = appendSubtitle(this.subtitles, interimId, {
            id: newId,
            text: transcript,
            timestamp: Date.now(),
          });
          interimId = newId;
        }
      }
    };

    recognition.onerror = (event) => {
      if (event.error === 'aborted' || event.error === 'no-speech') return;
      hasErrored = true;

      const messages: Record<string, string> = {
        network: 'No internet connection.',
        'not-allowed': 'Microphone access denied. Please allow microphone permission.',
        'audio-capture': 'No microphone found. Please connect a microphone.',
        service: 'Speech service unavailable.',
      };

      this.errorMessage = messages[event.error] ?? `${event.error}: ${event.message}`;
      this.status = 'error';
    };

    recognition.onend = () => {
      if (!this.stopping && !hasErrored) {
        try { recognition.start(); } catch { /* restart failed */ }
      }
    };

    this.recognition = recognition;

    try {
      recognition.start();
    } catch (e) {
      this.errorMessage = e instanceof Error ? e.message : 'Failed to start recognition';
      this.status = 'error';
    }
  }

  stop = async () => {
    // Stop Rust audio capture
    await this.stopCapture();

    this.stopping = true;
    if (this.recognition) {
      this.recognition.onend = null;
      this.recognition.stop();
      this.recognition = null;
    }
    this.status = 'idle';
  }

  setLanguage = (lang: string) => {
    this.language = lang;
    if (this.recognition) {
      this.recognition.onend = null;
      this.recognition.stop();
      this.recognition = null;
      setTimeout(() => this.start(), 100);
    }
  }

  translate = async (text: string, sourceLang: string, targetLang: string): Promise<string> => {
    try {
      const result = await invoke<{ original: string; translated: string }>('translate', {
        text,
        source_lang: sourceLang,
        target_lang: targetLang,
      });
      return result.translated;
    } catch (err) {
      console.error('Translation error:', err);
      throw err;
    }
  };
}

function appendSubtitle(
  subs: SubtitleLine[],
  removeId: string | null,
  add: SubtitleLine,
): SubtitleLine[] {
  const filtered = removeId ? subs.filter((s) => s.id !== removeId) : subs;
  return [...filtered, add].slice(-MAX_SUBTITLES);
}

export const speech = new Speech();
