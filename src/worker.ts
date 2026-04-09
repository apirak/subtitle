import { pipeline, env, type AutomaticSpeechRecognitionPipeline } from '@huggingface/transformers';
import type { MainToWorkerMessage, WorkerToMainMessage } from './types';

// Use local model files served from public/models/
env.allowLocalModels = true;
env.localModelPath = '/models/';
// Disable remote fallback so it only loads from local
env.allowRemoteModels = false;

function post(msg: WorkerToMainMessage) {
  self.postMessage(msg);
}

class WhisperPipeline {
  static instance: Promise<AutomaticSpeechRecognitionPipeline> | null = null;

  static async getInstance(progressCallback?: (progress: number) => void) {
    if (!this.instance) {
      this.instance = pipeline(
        'automatic-speech-recognition',
        'onnx-community/whisper-base',
        {
          dtype: 'fp32',
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

      const options: Record<string, unknown> = {};
      if (message.language) {
        options.language = message.language;
        options.task = 'transcribe';
      }
      console.log('[worker] transcribe with language:', message.language ?? 'auto-detect', options);

      const result = await transcriber(message.audio, options);
      console.log('[worker] result:', result);

      const text = (Array.isArray(result) ? result[0]?.text : (result as { text: string }).text) ?? '';
      const trimmed = typeof text === 'string' ? text.trim() : '';

      // Detect language from result if auto-detect mode
      const detectedLang = !message.language
        ? ((Array.isArray(result) ? result[0] : result) as Record<string, unknown>)?.language as string | undefined ?? null
        : message.language;

      if (trimmed.length > 0) {
        post({ status: 'result', text: trimmed, language: detectedLang });
      }
    } catch (e) {
      post({ status: 'error', message: e instanceof Error ? e.message : 'Transcription failed' });
    }
    return;
  }
});
