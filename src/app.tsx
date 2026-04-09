import { useSpeechToText } from './useSpeechToText'
import styles from './app.module.css'

export function App() {
  const { status, subtitles, loadProgress, startSession, stopSession } = useSpeechToText()

  return (
    <div class={styles.container}>
      {status === 'idle' && (
        <div class={styles.idleScreen}>
          <span class={styles.title}>Real-time Subtitles</span>
          <button class={styles.startButton} onClick={startSession}>
            <span class={styles.micIcon}>🎙</span>
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
        </div>
      )}

      {(status === 'listening' || status === 'processing') && (
        <div class={styles.listeningScreen}>
          <div class={styles.statusBar}>
            <div class={styles.statusIndicator}>
              <span class={styles.statusDot} />
              {status === 'processing' ? 'Processing…' : 'Listening…'}
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
          <button class={styles.startButton} onClick={startSession}>
            <span class={styles.micIcon}>🎙</span>
            Try Again
          </button>
        </div>
      )}
    </div>
  )
}
