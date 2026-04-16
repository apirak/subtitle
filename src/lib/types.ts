export type AppStatus = 'idle' | 'listening' | 'error' | 'vosk_setup';

export interface SubtitleLine {
  id: string;
  text: string;
  timestamp: number;
}
