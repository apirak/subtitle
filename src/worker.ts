import { pipeline, type AutomaticSpeechRecognitionPipeline } from '@huggingface/transformers';
import type { MainToWorkerMessage, WorkerToMainMessage } from './types';

function post(msg: WorkerToMainMessage) {
  self.postMessage(msg);
}

class WhisperPipeline {
  static instance: Promise<AutomaticSpeechRecognitionPipeline> | null = null;

  static async getInstance(progressCallback?: (progress: number) => void) {
    if (!this.instance) {
      this.instance = pipeline(
        'automatic-speech-recognition',
        'Xenova/whisper-tiny',
        {
          progress_callback: (data: Record<string, unknown>) => {
            if (data.status === 'progress' && typeof data.progress === 'number') {
              progressCallback?.(data.progress);
            }
          },
        },
      ) as Promise<AutomaticSpeechRecognitionPipeline>;
    }
    return this.instance;
  }
}

self.addEventListener('message', async (event: MessageEvent<MainToWorkerMessage>) => {
  const message = event.data;

  if (message.type === 'init') {
    try {
      await WhisperPipeline.getInstance((progress) => {
        post({ status: 'loading', progress });
      });
      post({ status: 'ready' });
    } catch (e) {
      post({ status: 'error', message: e instanceof Error ? e.message : 'Failed to load model' });
    }
    return;
  }

  if (message.type === 'transcribe') {
    try {
      post({ status: 'processing' });
      const transcriber = await WhisperPipeline.getInstance();
      const result = await transcriber(message.audio);

      const text = (Array.isArray(result) ? result[0]?.text : (result as { text: string }).text) ?? '';
      const trimmed = typeof text === 'string' ? text.trim() : '';

      if (trimmed.length > 0) {
        post({ status: 'result', text: trimmed });
      }
    } catch (e) {
      post({ status: 'error', message: e instanceof Error ? e.message : 'Transcription failed' });
    }
    return;
  }
});
