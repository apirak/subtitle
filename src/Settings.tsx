import { Dropdown } from './Dropdown'
import { LANGUAGES as WHISPER_LANGUAGES } from './useSpeechToText'
import { LANGUAGES as WEBSPEECH_LANGUAGES } from './useWebSpeechApi'
import { TARGET_LANGUAGES } from './useTranslation'
import type { EngineType } from './types'
import styles from './Settings.module.css'

interface SettingsProps {
  open: boolean
  onClose: () => void
  engine: EngineType
  onEngineChange: (e: EngineType) => void
  language: string
  onLanguageChange: (lang: string) => void
  targetLang: string
  onTargetLangChange: (lang: string) => void
  subtitlePosition: number
  onSubtitlePositionChange: (pos: number) => void
}

const ENGINE_OPTIONS = [
  { value: 'webspeech', label: 'Built-in' },
  { value: 'whisper', label: 'Whisper AI' },
]

export function Settings({ open, onClose, engine, onEngineChange, language, onLanguageChange, targetLang, onTargetLangChange, subtitlePosition, onSubtitlePositionChange }: SettingsProps) {
  const languages = (engine === 'whisper' ? WHISPER_LANGUAGES : WEBSPEECH_LANGUAGES).map((l) => ({
    value: l.code ?? '',
    label: l.label,
  }))

  return (
    <div class={`${styles.overlay} ${open ? styles.overlayOpen : ''}`} onClick={onClose}>
      <div class={`${styles.panel} ${open ? styles.panelOpen : ''}`} onClick={(e) => e.stopPropagation()}>
        <div class={styles.panelHeader}>
          <span class={styles.panelTitle}>Settings</span>
          <button class={styles.closeButton} onClick={onClose}>
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>

        <div class={styles.panelBody}>
          <div class={styles.field}>
            <label class={styles.label}>Speech Engine</label>
            <Dropdown value={engine} options={ENGINE_OPTIONS} onChange={(v) => onEngineChange(v as EngineType)} />
            <span class={styles.hint}>{engine === 'whisper' ? 'Better accuracy, downloads model (~40 MB)' : 'Instant start, uses browser speech recognition'}</span>
          </div>

          <div class={styles.field}>
            <label class={styles.label}>Source Language</label>
            <Dropdown value={language} options={languages} onChange={onLanguageChange} />
          </div>

          <div class={styles.field}>
            <label class={styles.label}>Translate To</label>
            <Dropdown value={targetLang} options={TARGET_LANGUAGES.map((l) => ({ value: l.value, label: l.label }))} onChange={onTargetLangChange} />
          </div>

          <div class={styles.field}>
            <label class={styles.label}>Subtitle Position — {subtitlePosition}%</label>
            <input
              type="range"
              min={5}
              max={90}
              value={subtitlePosition}
              onInput={(e) => onSubtitlePositionChange(Number((e.target as HTMLInputElement).value))}
              class={styles.slider}
            />
            <div class={styles.sliderLabels}>
              <span>Bottom</span>
              <span>Top</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
