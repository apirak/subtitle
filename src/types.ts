/** Messages sent from Main thread → Worker */
export type MainToWorkerMessage =
  | { type: 'init' }
  | { type: 'transcribe'; audio: Float32Array; language: string | null };

/** Messages sent from Worker → Main thread */
export type WorkerToMainMessage =
  | { status: 'loading'; progress: number }
  | { status: 'ready' }
  | { status: 'processing' }
  | { status: 'result'; text: string; language: string | null }
  | { status: 'error'; message: string };

export type AppStatus = 'idle' | 'loading' | 'listening' | 'processing' | 'error';

export type EngineType = 'whisper' | 'webspeech';

export interface SubtitleLine {
  id: string;
  text: string;
  timestamp: number;
}

export interface SpeechRecognitionEvent {
  resultIndex: number;
  results: SpeechRecognitionResultList;
}

export interface SpeechRecognitionResultList {
  length: number;
  item(index: number): SpeechRecognitionResult;
  [index: number]: SpeechRecognitionResult;
}

export interface SpeechRecognitionResult {
  isFinal: boolean;
  length: number;
  item(index: number): SpeechRecognitionAlternative;
  [index: number]: SpeechRecognitionAlternative;
}

export interface SpeechRecognitionAlternative {
  transcript: string;
  confidence: number;
}

export interface SpeechRecognitionErrorEvent {
  error: string;
  message: string;
}

export interface SpeechRecognitionInstance extends EventTarget {
  continuous: boolean;
  interimResults: boolean;
  lang: string;
  start(): void;
  stop(): void;
  abort(): void;
  onresult: ((event: SpeechRecognitionEvent) => void) | null;
  onerror: ((event: SpeechRecognitionErrorEvent) => void) | null;
  onend: (() => void) | null;
}
