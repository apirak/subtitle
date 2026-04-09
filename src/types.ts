/** Messages sent from Main thread → Worker */
export type MainToWorkerMessage =
  | { type: 'init' }
  | { type: 'transcribe'; audio: Float32Array };

/** Messages sent from Worker → Main thread */
export type WorkerToMainMessage =
  | { status: 'loading'; progress: number }
  | { status: 'ready' }
  | { status: 'processing' }
  | { status: 'result'; text: string }
  | { status: 'error'; message: string };

export type AppStatus = 'idle' | 'loading' | 'listening' | 'processing' | 'error';

export interface SubtitleLine {
  id: string;
  text: string;
  timestamp: number;
}
