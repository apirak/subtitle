import { useState, useEffect, useRef, useCallback } from 'preact/hooks';
import { useSpeechToText, LANGUAGES as WHISPER_LANGUAGES } from './useSpeechToText'
import { useWebSpeechApi, LANGUAGES as WEBSPEECH_LANGUAGES } from './useWebSpeechApi'
import { useTranslation, TARGET_LANGUAGES } from './useTranslation'
import { Settings } from './Settings'
import type { EngineType } from './types'
import styles from './app.module.css'

export function App() {
  const [engine, setEngine] = useState<EngineType>('webspeech')
  const [targetLang, setTargetLang] = useState('th')
  const [translations, setTranslations] = useState<Record<string, string>>({})
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [subtitlePosition, setSubtitlePosition] = useState(20)
  const translatedRef = useRef<Set<string>>(new Set())

  const whisper = useSpeechToText()
  const webspeech = useWebSpeechApi()
  const current = engine === 'whisper' ? whisper : webspeech
  const { status, subtitles, loadProgress, errorMessage, language, setLanguage, startSession, stopSession } = current

  const { translate } = useTranslation(targetLang)

  const updateTranslation = useCallback((id: string, translation: string) => {
    setTranslations((prev) => ({ ...prev, [id]: translation }))
  }, [])

  useEffect(() => {
    console.log('[app] subtitles changed:', subtitles.length, 'targetLang:', targetLang)
    for (const line of subtitles) {
      if (line.id.startsWith('interim-')) continue
      if (translatedRef.current.has(line.id)) continue
      console.log('[app] new final subtitle:', line.id, `"${line.text}"`)
      translatedRef.current.add(line.id)
      if (targetLang) {
        console.log('[app] calling translate for', line.id)
        translate(line.id, line.text, updateTranslation)
      } else {
        console.warn('[app] no targetLang set, skipping translation')
      }
    }
  }, [subtitles, targetLang, translate, updateTranslation])

  const handleLanguageChange = (value: string) => {
    setLanguage(value === '' ? null : value)
  }

  // Get display labels for current settings
  const languages = engine === 'whisper' ? WHISPER_LANGUAGES : WEBSPEECH_LANGUAGES
  const sourceLabel = languages.find((l) => (l.code ?? '') === (language ?? ''))?.label ?? 'Auto'
  const targetLabel = TARGET_LANGUAGES.find((l) => l.value === targetLang)?.label ?? 'English'

  return (
    <div class={styles.container}>
      {status === 'idle' && (
        <div class={styles.idleScreen}>
          <div class={styles.titleGroup}>
            <span class={styles.title}>Real-time Subtitles</span>
            <span class={styles.subtitleConfig}>{sourceLabel} → {targetLabel}</span>
          </div>

          <button class={styles.startButton} onClick={startSession}>
            <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" />
              <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
              <line x1="12" y1="19" x2="12" y2="22" />
            </svg>
            Start
          </button>

          <button class={styles.settingsButton} onClick={() => setSettingsOpen(true)}>
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="3" />
              <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1Z" />
            </svg>
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
        <>
          <div class={styles.listeningScreen} />

          <div class={styles.statusBar}>
            <div class={styles.statusIndicator}>
              <span class={styles.statusDot} />
              {status === 'processing' ? 'Processing…' : 'Listening…'}
              <span class={styles.langBadge}>{sourceLabel} → {targetLabel}</span>
            </div>
            <button class={styles.stopButton} onClick={stopSession}>
              Stop
            </button>
          </div>

          <div class={styles.subtitleContainer} style={{ bottom: `${subtitlePosition}%` }}>
            {subtitles.map((line) => {
              const isFinal = !line.id.startsWith('interim-')
              const translation = translations[line.id]
              const isTranslating = isFinal && !translation

              return (
                <div key={line.id} class={styles.subtitleLine}>
                  <span>{line.text}</span>
                  {isTranslating && (
                    <span class={styles.subtitleTranslating}>
                      <span class={styles.dot}>·</span>
                      <span class={styles.dot}>·</span>
                      <span class={styles.dot}>·</span>
                    </span>
                  )}
                  {translation && (
                    <span class={styles.subtitleTranslation}>{translation}</span>
                  )}
                </div>
              )
            })}
          </div>
        </>
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
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" />
              <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
              <line x1="12" y1="19" x2="12" y2="22" />
            </svg>
            Try Again
          </button>
        </div>
      )}

      <Settings
        open={settingsOpen}
        onClose={() => setSettingsOpen(false)}
        engine={engine}
        onEngineChange={(e) => setEngine(e)}
        language={language ?? ''}
        onLanguageChange={handleLanguageChange}
        targetLang={targetLang}
        onTargetLangChange={setTargetLang}
        subtitlePosition={subtitlePosition}
        onSubtitlePositionChange={setSubtitlePosition}
      />
    </div>
  )
}
