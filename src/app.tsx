import { useState } from 'preact/hooks';
import { useSpeechToText, LANGUAGES as WHISPER_LANGUAGES } from './useSpeechToText'
import { useWebSpeechApi, LANGUAGES as WEBSPEECH_LANGUAGES } from './useWebSpeechApi'
import { Dropdown } from './Dropdown'
import type { EngineType } from './types'
import styles from './app.module.css'

const ENGINE_OPTIONS = [
  { value: 'webspeech', label: 'Built-in (online)' },
  { value: 'whisper', label: 'Whisper AI (offline)' },
]

export function App() {
  const [engine, setEngine] = useState<EngineType>('webspeech')
  const whisper = useSpeechToText()
  const webspeech = useWebSpeechApi()

  const current = engine === 'whisper' ? whisper : webspeech
  const { status, subtitles, loadProgress, errorMessage, language, setLanguage, detectedLanguage, startSession, stopSession } = current
  const languages = (engine === 'whisper' ? WHISPER_LANGUAGES : WEBSPEECH_LANGUAGES).map((l) => ({
    value: l.code ?? '',
    label: l.label,
  }))

  const handleEngineChange = (value: string) => {
    if (status !== 'idle') return
    setEngine(value as EngineType)
  }

  const handleLanguageChange = (value: string) => {
    setLanguage(value === '' ? null : value)
  }

  return (
    <div class={styles.container}>
      {status === 'idle' && (
        <div class={styles.idleScreen}>
          <div class={styles.titleGroup}>
            <span class={styles.title}>Real-time Subtitles</span>
          </div>

          <Dropdown value={engine} options={ENGINE_OPTIONS} onChange={handleEngineChange} />
          <Dropdown value={language ?? ''} options={languages} onChange={handleLanguageChange} />

          <button class={styles.startButton} onClick={startSession}>
            <span class={styles.micIcon}>
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" />
                <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
                <line x1="12" y1="19" x2="12" y2="22" />
              </svg>
            </span>
            Start Session
          </button>
        </div>
      )}

      {status === 'loading' && (
        <div class={styles.loadingScreen}>
          <span class={styles.loadingText}>Loading AI Model…</span>
          <div class={styles.progressBar}>
            <div class={styles.progressFill} style={{ width: `${loadProgress}%` }} />
          </div>
          <span class={styles.loadingPercent}>{loadProgress}%</span>
        </div>
      )}

      {(status === 'listening' || status === 'processing') && (
        <div class={styles.listeningScreen}>
          <div class={styles.statusBar}>
            <div class={styles.statusIndicator}>
              <span class={styles.statusDot} />
              {status === 'processing' ? 'Processing…' : 'Listening…'}
              {detectedLanguage && (
                <span class={styles.langBadge}>{detectedLanguage.toUpperCase()}</span>
              )}
              <span class={styles.engineBadge}>{engine === 'whisper' ? 'Whisper AI' : 'Built-in'}</span>
            </div>
            <button class={styles.stopButton} onClick={stopSession}>
              Stop
            </button>
          </div>

          <div class={styles.subtitleContainer}>
            {subtitles.map((line) => (
              <div key={line.id} class={styles.subtitleLine}>
                {line.text}
              </div>
            ))}
          </div>
        </div>
      )}

      {status === 'error' && (
        <div class={styles.errorScreen}>
          <span class={styles.errorText}>
            Something went wrong. Please check microphone permissions.
          </span>
          {errorMessage && (
            <span class={styles.errorDetail}>{errorMessage}</span>
          )}
          <button class={styles.startButton} onClick={startSession}>
            <span class={styles.micIcon}>
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" />
                <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
                <line x1="12" y1="19" x2="12" y2="22" />
              </svg>
            </span>
            Try Again
          </button>
        </div>
      )}
    </div>
  )
}
