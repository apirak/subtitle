import { useCallback, useRef } from 'preact/hooks';

const API_URL = 'https://api.deepinfra.com/v1/openai/chat/completions';
const MODEL = 'Qwen/Qwen3-14B'

const TARGET_LANGUAGES = [
  { value: 'en', label: 'English' },
  { value: 'th', label: 'ไทย' },
  { value: 'zh', label: '中文' },
  { value: 'ja', label: '日本語' },
  { value: 'ko', label: '한국어' },
  { value: 'es', label: 'Español' },
  { value: 'fr', label: 'Français' },
  { value: 'de', label: 'Deutsch' },
  { value: 'pt', label: 'Português' },
  { value: 'ru', label: 'Русский' },
  { value: 'ar', label: 'العربية' },
  { value: 'hi', label: 'हिन्दी' },
  { value: 'vi', label: 'Tiếng Việt' },
  { value: 'id', label: 'Bahasa Indonesia' },
] as const;

const LANG_NAMES: Record<string, string> = {
  en: 'English', th: 'Thai', zh: 'Chinese', ja: 'Japanese', ko: 'Korean',
  es: 'Spanish', fr: 'French', de: 'German', pt: 'Portuguese', ru: 'Russian',
  ar: 'Arabic', hi: 'Hindi', vi: 'Vietnamese', id: 'Indonesian',
};

export { TARGET_LANGUAGES };

export function useTranslation(targetLang: string) {
  const inFlightRef = useRef<Set<string>>(new Set());
  const recentLines = useRef<string[]>([]);

  const translate = useCallback(async (id: string, text: string, onUpdate: (id: string, translation: string) => void) => {
    console.log('[translate] called — id:', id, 'targetLang:', targetLang, 'inFlight:', inFlightRef.current.has(id));
    if (!targetLang || inFlightRef.current.has(id)) return;

    const apiKey = import.meta.env.VITE_DEEPINFRA_API_KEY;
    console.log('[translate] apiKey:', apiKey ? `${apiKey.slice(0, 8)}...` : 'MISSING');
    if (!apiKey) {
      console.warn('[translate] No API key found');
      return;
    }

    const langName = LANG_NAMES[targetLang] ?? targetLang;

    // Build context from previous 2 lines
    const context = recentLines.current.slice(-2);
    let prompt: string;
    if (context.length > 0) {
      prompt = `/no_think
Translate the last sentence to ${langName}. The previous sentences are for context only — do not translate them.

Context:
${context.map((c, i) => `${i + 1}. ${c}`).join('\n')}

Translate this:
${text}

Return only the translation.`;
    } else {
      prompt = `/no_think\nTranslate to ${langName}. Return only the translation.\n\n${text}`;
    }

    console.log('[translate] translating to', langName, ':', text);

    // Store this line for future context
    recentLines.current.push(text);
    if (recentLines.current.length > 3) recentLines.current.shift();

    inFlightRef.current.add(id);

    try {
      const res = await fetch(API_URL, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${apiKey}`,
        },
        body: JSON.stringify({
          model: MODEL,
          messages: [
            { role: 'user', content: prompt },
          ],
        }),
      });

      if (!res.ok) {
        console.error('[translate] API error:', res.status, await res.text());
        return;
      }

      const data = await res.json();
      console.log('[translate] full response:', JSON.stringify(data.choices?.[0], null, 2));
      const msg = data.choices?.[0]?.message as Record<string, unknown> | undefined;
      const translation = String(msg?.content || msg?.reasoning_content || '').trim();
      const translationWithoutThinkTag = translation.replace(/<think>[\s\S]*?<\/think>/g, '').trim();
      console.log('[translate] result:', translationWithoutThinkTag);

      if (translationWithoutThinkTag) {
        onUpdate(id, translationWithoutThinkTag);
      }
    } catch (e) {
      console.error('[translate] failed:', e);
    } finally {
      inFlightRef.current.delete(id);
    }
  }, [targetLang]);

  return { translate };
}
