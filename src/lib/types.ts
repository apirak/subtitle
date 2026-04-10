export type AppStatus = 'idle' | 'listening' | 'error';

export interface SubtitleLine {
  id: string;
  text: string;
  timestamp: number;
}
