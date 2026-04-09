import { useState, useRef, useCallback } from 'preact/hooks';
import type { AppStatus, SubtitleLine, SpeechRecognitionInstance } from './types';

export const LANGUAGES = [
  { code: '', label: 'Auto-detect' },
  { code: 'en-US', label: 'English' },
  { code: 'th-TH', label: 'ไทย' },
  { code: 'zh-CN', label: '中文' },
  { code: 'ja-JP', label: '日本語' },
  { code: 'ko-KR', label: '한국어' },
  { code: 'es-ES', label: 'Español' },
  { code: 'fr-FR', label: 'Français' },
  { code: 'de-DE', label: 'Deutsch' },
  { code: 'pt-BR', label: 'Português' },
  { code: 'ru-RU', label: 'Русский' },
  { code: 'ar-SA', label: 'العربية' },
  { code: 'hi-IN', label: 'हिन्दी' },
  { code: 'vi-VN', label: 'Tiếng Việt' },
  { code: 'id-ID', label: 'Bahasa Indonesia' },
] as const;

const MAX_SUBTITLES = 12;

function createRecognition(): SpeechRecognitionInstance | null {
  const SpeechRecognition = (window as unknown as Record<string, unknown>).SpeechRecognition
    ?? (window as unknown as Record<string, unknown>).webkitSpeechRecognition;
  console.log('[webspeech] createRecognition — available:', !!SpeechRecognition);
  if (!SpeechRecognition) return null;
  return new (SpeechRecognition as new () => SpeechRecognitionInstance)();
}

export function useWebSpeechApi() {
  const [status, setStatus] = useState<AppStatus>('idle');
  const [subtitles, setSubtitles] = useState<SubtitleLine[]>([]);
  const [loadProgress, setLoadProgress] = useState(0);
  const [errorMessage, setErrorMessage] = useState('');
  const [language, setLanguage] = useState('');
  const [detectedLanguage, setDetectedLanguage] = useState<string | null>(null);
  const languageRef = useRef('');

  const recognitionRef = useRef<SpeechRecognitionInstance | null>(null);
  const stoppingRef = useRef(false);

  const startSession = useCallback(async () => {
    console.log('[webspeech] startSession — lang:', languageRef.current || 'auto');
    console.log('[webspeech] navigator.onLine:', navigator.onLine);
    console.log('[webspeech] userAgent:', navigator.userAgent);
    console.log('[webspeech] protocol:', window.location.protocol);
    const recognition = createRecognition();
    if (!recognition) {
      console.error('[webspeech] SpeechRecognition not available');
      setErrorMessage('Web Speech API is not supported in this browser. Try Safari or Chrome.');
      setStatus('error');
      return;
    }

    setSubtitles([]);
    setLoadProgress(0);
    setErrorMessage('');
    stoppingRef.current = false;
    setStatus('listening');

    recognition.continuous = true;
    recognition.interimResults = true;
    recognition.lang = languageRef.current;

    let interimId: string | null = null;
    let hasErrored = false;

    recognition.onresult = (event) => {
      console.log('[webspeech] onresult — resultIndex:', event.resultIndex, 'results:', event.results.length);
      for (let i = event.resultIndex; i < event.results.length; i++) {
        const result = event.results[i];
        const transcript = result[0].transcript.trim();

        if (!transcript) continue;

        console.log('[webspeech]', result.isFinal ? 'FINAL' : 'interim', `"${transcript}"`);
        if (result.isFinal) {
          interimId = null;
          setSubtitles((prev) => {
            const next = [
              ...prev,
              { id: `${Date.now()}-${Math.random()}`, text: transcript, timestamp: Date.now() },
            ];
            return next.slice(-MAX_SUBTITLES);
          });
        } else {
          setSubtitles((prev) => {
            const filtered = interimId ? prev.filter((s) => s.id !== interimId) : prev;
            const newId = `interim-${Date.now()}`;
            interimId = newId;
            const next = [
              ...filtered,
              { id: newId, text: transcript, timestamp: Date.now() },
            ];
            return next.slice(-MAX_SUBTITLES);
          });
        }
      }
    };

    recognition.onerror = (event) => {
      if (event.error === 'aborted' || event.error === 'no-speech') {
        console.log('[webspeech] onerror (ignored):', event.error);
        return;
      }
      hasErrored = true;
      console.error('[webspeech] onerror:', event.error, event.message);

      const messages: Record<string, string> = {
        network: 'No internet connection. Enable Offline mode to use Whisper AI without network.',
        'not-allowed': 'Microphone access denied. Please allow microphone permission.',
        'audio-capture': 'No microphone found. Please connect a microphone.',
        service: 'Speech service unavailable. Try Offline mode instead.',
      };

      setErrorMessage(messages[event.error] ?? `${event.error}: ${event.message}`);
      setStatus('error');
    };

    recognition.onend = () => {
      console.log('[webspeech] onend — stopping:', stoppingRef.current, 'hasErrored:', hasErrored);
      // Only auto-restart if user hasn't stopped and no fatal error occurred
      if (!stoppingRef.current && !hasErrored) {
        console.log('[webspeech] auto-restarting...');
        try {
          recognition.start();
        } catch (e) {
          console.warn('[webspeech] restart failed:', e);
        }
      }
    };

    recognitionRef.current = recognition;

    try {
      console.log('[webspeech] calling recognition.start()');
      recognition.start();
    } catch (e) {
      console.error('[webspeech] start failed:', e);
      setErrorMessage(e instanceof Error ? e.message : 'Failed to start recognition');
      setStatus('error');
    }
  }, []);

  const stopSession = useCallback(() => {
    console.log('[webspeech] stopSession');
    stoppingRef.current = true;
    if (recognitionRef.current) {
      recognitionRef.current.onend = null;
      recognitionRef.current.stop();
      recognitionRef.current = null;
    }
    setStatus('idle');
  }, []);

  const updateLanguage = useCallback((lang: string) => {
    console.log('[webspeech] updateLanguage:', lang || 'auto');
    setLanguage(lang);
    languageRef.current = lang;
    // Restart recognition with new language if currently listening
    if (recognitionRef.current) {
      const recognition = recognitionRef.current;
      recognition.onend = null;
      recognition.stop();
      recognitionRef.current = null;
      // Small delay to allow stop to complete
      setTimeout(() => startSession(), 100);
    }
  }, [startSession]);

  return {
    status,
    subtitles,
    loadProgress,
    errorMessage,
    language,
    setLanguage: updateLanguage,
    detectedLanguage,
    startSession,
    stopSession,
  };
}
