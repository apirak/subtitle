import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { type TranslationConfig, translateWithOpenAI } from './openai-translator';
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
  engine = $state<'browser' | 'vosk' | 'remote' | 'gemini'>('browser');
  remoteEndpoint = $state<string>('');
  apiKey = $state<string>('');
  translationEngine = $state<string>('remote');
  translationEndpoint = $state<string>('');
  translationModel = $state<string>('');
  translationApiKey = $state<string>('');

  private recognition: SpeechRecognitionInstance | null = null;
  private stopping = false;

  private _subtitleUnlisten: (() => void) | null = null;
  private _errorUnlisten: (() => void) | null = null;
  private _statusPollInterval: ReturnType<typeof setInterval> | null = null;

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

    this.unlistenError = await listen<{ code: string; message: string }>('backend://subtitle/error', (event) => {
      const payload = event.payload;
      this.setError(`${payload.code}: ${payload.message}`);
    });
  };

  addInterimSubtitle = (id: string, text: string) => {
    const interimId = `interim-${id}`;
    this.subtitles = appendSubtitle(this.subtitles, interimId, {
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

  setTranslation = (_id: string, _original: string, _translated: string) => {
    // Translation storage is managed by app.svelte
    // This listener exists for future direct backend translation integration
  };

  setError = (message: string) => {
    this.errorMessage = message;
    this.status = 'error';
  };

  destroy = () => {
    this.stopStatusPolling();
    this.unlistenUpdate?.();
    this.unlistenFinal?.();
    this.unlistenTranslated?.();
    this.unlistenError?.();
  };

  private startStatusPolling = () => {
    this.stopStatusPolling();
    this._statusPollInterval = setInterval(async () => {
      if (this.engine !== 'remote' && this.engine !== 'gemini') return;
      if (this.status !== 'listening') return;

      try {
        const status = await invoke<{
          is_running: boolean;
          chunks_accumulated: number;
          last_transcript: string | null;
          error: string | null;
        }>('remote_asr_status');

        console.log('[RemoteASR] status:', {
          is_running: status.is_running,
          chunks_accumulated: status.chunks_accumulated,
          has_last_transcript: Boolean(status.last_transcript),
          error: status.error,
        });
      } catch (err) {
        console.error('[RemoteASR] status poll failed:', err);
      }
    }, 2000);
  };

  private stopStatusPolling = () => {
    if (this._statusPollInterval) {
      clearInterval(this._statusPollInterval);
      this._statusPollInterval = null;
    }
  };

  startCapture = async (): Promise<{ sample_rate: number; channels: number }> => {
    if (this.engine === 'browser') {
      // Browser STT uses Web Speech API directly, so no Rust capture is required.
      return { sample_rate: 16000, channels: 1 };
    }
    if (this.engine === 'remote' || this.engine === 'gemini') {
      try {
        const result = await invoke<{ sample_rate: number; channels: number }>('audio_capture_start');
        return result;
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        this.setError(`Audio capture failed: ${message}`);
        throw err;
      }
    }
    if (this.engine === 'vosk') {
      try {
        const result = await invoke<{ sample_rate: number; channels: number }>('audio_capture_start');
        return result;
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        this.setError(`Audio capture failed: ${message}`);
        throw err;
      }
    }
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
    if (this.engine === 'browser') {
      return;
    }
    if (this.engine === 'remote' || this.engine === 'gemini') {
      try {
        await invoke('remote_asr_stop');
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        console.error('Remote ASR stop error:', message);
      }
    }
    if (this.engine === 'vosk') {
      try {
        await invoke('vosk_stop');
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        console.error('Vosk stop error:', message);
      }
    }
    try {
      await invoke('audio_capture_stop');
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      console.error('Audio capture stop error:', message);
    }
  };

  start = async () => {
    if (this.engine === 'remote' || this.engine === 'gemini') {
      console.log(
        '[RemoteASR] start called, apiKey length:',
        this.apiKey?.length ?? 0,
        'endpoint saved:',
        this.remoteEndpoint
      );
      try {
        console.log('[RemoteASR] starting audio capture...');
        await this.startCapture();
        console.log('[RemoteASR] audio capture started, invoking remote_asr_start...');
        await invoke('remote_asr_start', { apiKey: this.apiKey });
        console.log('[RemoteASR] remote_asr_start succeeded');
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        console.error('[RemoteASR] start failed:', message);
        this.setError(`Failed to start Remote ASR: ${message}`);
        return;
      }

      this._subtitleUnlisten = await listen<{ text: string; timestamp: number; is_final: boolean }>(
        'subtitle',
        (event) => {
          const { text, timestamp, is_final } = event.payload;
          console.log('[RemoteASR] subtitle event:', { text, is_final, timestamp });
          if (!text || !text.trim()) return;
          const id = `remote-${timestamp}`;
          if (is_final) {
            this.addFinalSubtitle(id, text.trim());
          } else {
            this.addInterimSubtitle(id, text.trim());
          }
        }
      );

      this._errorUnlisten = await listen<{ message: string; retryable: boolean }>('asr-error', (event) => {
        const { message, retryable } = event.payload;
        console.error('[RemoteASR] asr-error event:', { message, retryable });
        this.errorMessage = message;
        if (!retryable) {
          this.status = 'error';
        }
      });

      this.subtitles = [];
      this.errorMessage = '';
      this.status = 'listening';
      this.startStatusPolling();
      return;
    }

    if (this.engine === 'vosk') {
      this.subtitles = [];
      this.errorMessage = '';

      try {
        await this.startCapture();
      } catch {
        return;
      }

      try {
        // Check if Vosk model exists
        const modelInfo = await invoke<{ status: string; model_path: string; model_name: string | null }>(
          'vosk_model_status'
        );

        if (modelInfo.status === 'not_found') {
          await this.stopCapture();
          this.status = 'vosk_setup';
          return;
        }

        await invoke('vosk_load_model');
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        this.setError(`Failed to load Vosk model: ${message}`);
        return;
      }

      try {
        await invoke('vosk_start');
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        this.setError(`Failed to start Vosk ASR: ${message}`);
        return;
      }

      this.status = 'listening';
      return;
    }

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
        try {
          recognition.start();
        } catch {
          /* restart failed */
        }
      }
    };

    this.recognition = recognition;

    try {
      recognition.start();
    } catch (e) {
      this.errorMessage = e instanceof Error ? e.message : 'Failed to start recognition';
      this.status = 'error';
    }
  };

  stop = async () => {
    console.log('[Speech] stop called, engine:', this.engine);
    if (this._subtitleUnlisten) {
      this._subtitleUnlisten();
      this._subtitleUnlisten = null;
    }
    if (this._errorUnlisten) {
      this._errorUnlisten();
      this._errorUnlisten = null;
    }

    this.stopStatusPolling();

    await this.stopCapture();

    this.stopping = true;
    if (this.recognition) {
      this.recognition.onend = null;
      this.recognition.stop();
      this.recognition = null;
    }
    this.status = 'idle';
  };

  setLanguage = (lang: string) => {
    this.language = lang;
    if (this.recognition) {
      this.recognition.onend = null;
      this.recognition.stop();
      this.recognition = null;
      setTimeout(() => this.start(), 100);
    }
  };

  setEngine = (engine: 'browser' | 'vosk' | 'remote' | 'gemini') => {
    this.engine = engine;
    if (this.status === 'listening') {
      this.stop();
      setTimeout(() => this.start(), 100);
    }
  };

  translate = async (text: string, sourceLang: string, targetLang: string): Promise<string> => {
    if (this.translationEngine === 'none') {
      return '';
    }

    if (this.translationEngine === 'remote') {
      const config: TranslationConfig = {
        engine: 'remote',
        model: this.translationModel,
        endpoint: this.translationEndpoint,
        apiKey: this.translationApiKey,
      };

      try {
        return await translateWithOpenAI(text, sourceLang, targetLang, config);
      } catch (err) {
        console.error('[Translation] Error:', err);
        return '';
      }
    }

    // Fallback to backend stub (should not reach here with current UI)
    try {
      const result = await invoke<{ original: string; translated: string }>('translate', {
        text,
        source_lang: sourceLang,
        target_lang: targetLang,
      });
      return result.translated;
    } catch (err) {
      console.error('Translation error:', err);
      return '';
    }
  };

  saveSetting = async (key: string, value: string | number) => {
    const stringValue = typeof value === 'number' ? String(value) : value;
    await invoke('settings_set', { key, value: stringValue });
    if (key === 'remote_endpoint') {
      this.remoteEndpoint = stringValue;
    }
    if (key === 'translation_engine') {
      this.translationEngine = stringValue;
    }
    if (key === 'translation_endpoint') {
      this.translationEndpoint = stringValue;
    }
    if (key === 'translation_model') {
      this.translationModel = stringValue;
    }
  };

  saveApiKey = async (keyName: string, keyValue: string) => {
    console.log('saveApiKey called:', keyName, 'value length:', keyValue.length);
    if (!keyValue || !keyValue.trim()) {
      console.error('saveApiKey: empty or whitespace API key, skipping');
      return;
    }
    try {
      const { saveApiKey: strongholdSave } = await import('./stronghold');
      await strongholdSave(keyName, keyValue);
      console.log('saveApiKey: Stronghold save succeeded');
    } catch (e) {
      console.error('saveApiKey: Stronghold save failed', e);
    }
  };
}

function appendSubtitle(subs: SubtitleLine[], removeId: string | null, add: SubtitleLine): SubtitleLine[] {
  const filtered = removeId ? subs.filter((s) => s.id !== removeId) : subs;
  return [...filtered, add].slice(-MAX_SUBTITLES);
}

export const speech = new Speech();

export function createSpeechForTest() {
  return new Speech();
}
