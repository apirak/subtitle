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

  start = () => {
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

  stop = () => {
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
