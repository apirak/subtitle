# Settings Redesign — macOS-style Modal with Theme / Language / STT / Translate

## Goal

แทนที่ bottom-sheet settings ด้วย modal กลางหน้าจอแบบ macOS System Settings:

- Left sidebar: เลือก section (Display, Language, Speech to Text, Translate)
- Right panel: config ของ section ที่เลือก
- รองรับ OpenAI-compatible provider แยกกันสำหรับ STT และ Translation

## Decisions Made

| #   | ประเด็น                            | การตัดสินใจ                                                 |
| --- | ---------------------------------- | ----------------------------------------------------------- |
| D1  | Translation HTTP: frontend vs Rust | **Frontend fetch**                                          |
| D2  | CSP connect-src                    | **`https:` (อนุญาตทุก HTTPS)** — แก้ใน tauri.conf.json แล้ว |
| D3  | AI menu: รวม vs แยก                | **แยก "Speech to Text" + "Translate"** เป็น 2 menu item     |

---

## UX Review

### โครงสร้าง Menu ที่เสนอ (Theme / Language / AI)

**ข้อดี:**

- แบ่ง concern ชัดเจน — appearance, content language, AI backend แยกกัน
- ผู้ใช้ที่ไม่ต้องการ configure AI ไม่ต้องเห็น API key
- ตามรูปแบบที่ผู้ใช้คุ้นเคยจาก macOS / Android Settings

**ข้อสังเกตและข้อเสนอแนะ:**

1. **"Theme"** ชื่อนี้ครอบคลุมพอ แต่ภายในมีทั้ง visual theme และ behavior (highlight toggle) — อาจเรียกว่า **"Display"** เพื่อให้กว้างกว่า

2. **Color: Day / Night / Toy**
   - Night = dark theme ปัจจุบัน (black bg, white text)
   - Day = light theme (ยังไม่มีใน codebase, ต้องสร้างใหม่)
   - Toy = theme สีสดใส (เช่น neon/pastel) สำหรับความสนุก
   - ⚠️ ปัจจุบัน STACK.md ระบุ "Dark theme only" ต้องเพิ่ม CSS variable system

3. **"Highlight first language"**
   - หมายถึง: แสดง source language subtitle (ต้นฉบับ) ให้เด่นกว่า translation
   - ใช้งาน: SubtitleLine.svelte ต้องรู้ว่า line ไหนคือ source, ไหนคือ translation แล้ว apply style ต่างกัน
   - ชื่อ toggle ควรเป็น **"Show source text prominently"** หรือ **"Emphasize original text"** เพื่อความชัดเจน

4. **Language section**
   - "Base Language" = source language (ภาษาที่พูด)
   - "First Translate" = translation หลัก (เดิมคือ Target Language)
   - "Second Translate" = translation ที่สอง (ใหม่ ยังไม่มีใน codebase)
   - ⚠️ Second translate เพิ่ม complexity ให้ subtitle display และ translation pipeline
   - แนะนำ: "None" เป็น default และ disable ได้ง่าย
   - SubtitleLine จะต้องแสดง 3 rows: source + translation1 + translation2

5. **AI section — Speech to Text**
   - "online/offline" toggle ไม่จำเป็น เพราะ model choice บอก mode อยู่แล้ว
   - แนะนำ: ลบ online/offline toggle แล้วให้ model picker แสดงผลตรงๆ
   - Model options ที่ควรมี:
     - `browser` → Web Speech API (ฟรี, ต้อง internet, ไม่มี URL/key)
     - `vosk` → Vosk on-device (ฟรี, offline, ไม่มี URL/key)
     - `openai-compatible` → Custom endpoint (URL + key จำเป็น)
     - `gemini` → Google Gemini (API key จำเป็น, URL fixed)
   - URL และ API Key ควร **show/hide ตาม model ที่เลือก** (conditional rendering)

6. **AI section — Translate**
   - Provider options:
     - `none` → ปิด translation
     - `openai-compatible` → Custom OpenAI-compatible endpoint (URL + key)
     - `deepinfra` → DeepInfra preset (URL autofill, key จำเป็น)
     - `ollama` → Local Ollama (URL default localhost, no key)
   - Model field: text input free-form หรือ dropdown ของ common models
   - ⚠️ Model ต้องส่งไปใน API request body ด้วย (ปัจจุบัน translation engine ยัง stub)

---

## Data Flow Analysis

### Settings ที่เพิ่มใหม่ → ไปอยู่ที่ไหน

| Setting                | Storage Key                                           | ใช้ใน                                            |
| ---------------------- | ----------------------------------------------------- | ------------------------------------------------ |
| `theme`                | `settings.json: theme`                                | app.svelte → CSS class on `<body>` หรือ root div |
| `highlight_source`     | `settings.json: highlight_source`                     | SubtitleLine.svelte props                        |
| `target_lang_2`        | `settings.json: target_lang_2`                        | app.svelte → second translateLine pipeline       |
| `stt_model`            | `settings.json: engine` (rename/extend)               | speech.svelte.ts `engine` field                  |
| `stt_endpoint`         | `settings.json: remote_endpoint` (existing)           | remote_asr.rs, speech.svelte.ts                  |
| `stt_api_key`          | Stronghold vault key: `"stt"` (rename จาก `"remote"`) | speech.svelte.ts `apiKey`                        |
| `translation_provider` | `settings.json: translation_engine` (existing)        | app.svelte translateLine()                       |
| `translation_model`    | `settings.json: translation_model` (new)              | HTTP request body ใน translate call              |
| `translation_endpoint` | `settings.json: translation_endpoint` (new)           | translate HTTP client                            |
| `translation_api_key`  | Stronghold vault key: `"translation"` (new)           | translate HTTP client                            |

### Translation Pipeline ปัจจุบัน vs ใหม่

**ปัจจุบัน:** `speech.translate()` ใน speech.svelte.ts → stub ใน commands.rs (ยังไม่ implement จริง)

**ใหม่ที่ต้องทำ:** เมื่อ translation_engine = "openai-compatible" หรือ "deepinfra":

- Frontend call `invoke('translate_remote', { text, sourceLang, targetLang, endpoint, model, apiKey })`
- หรือ: เรียก fetch โดยตรงจาก frontend (ง่ายกว่า ไม่ต้องผ่าน Rust สำหรับ HTTP)
- ⚠️ CSP ปัจจุบันอนุญาต `https://api.deepinfra.com` เท่านั้น ต้องเปลี่ยนเป็น custom URL ได้

---

## Component Architecture ใหม่

```
Settings.svelte (modal, centered)
├── SettingsSidebar.svelte
│   ├── MenuItem: Theme
│   ├── MenuItem: Language
│   └── MenuItem: AI
└── SettingsPanel (right side, switches by activeSection)
    ├── ThemePanel.svelte
    │   ├── Color picker (Day/Night/Toy)
    │   └── Highlight source toggle
    ├── LanguagePanel.svelte
    │   ├── Base Language dropdown
    │   ├── First Translate dropdown
    │   └── Second Translate dropdown
    └── AiPanel.svelte
        ├── STT subsection
        │   ├── Model dropdown
        │   ├── URL input (conditional)
        │   └── API Key input (conditional)
        └── Translate subsection
            ├── Provider dropdown
            ├── Model input
            ├── URL input (conditional)
            └── API Key input (conditional)
```

---

## Implementation Plan

### Phase 1 — Modal Shell + Navigation (ไม่เปลี่ยน logic)

**Scope:** แค่เปลี่ยน UI shell ของ Settings จาก bottom-sheet → centered modal พร้อม left sidebar

**Frontend tasks:**

1. `Settings.svelte` — เปลี่ยน layout:
   - `fixed inset-0` → centered modal (`flex items-center justify-center`)
   - เพิ่ม left sidebar `w-40` พร้อม menu items
   - เพิ่ม state `activeSection: 'theme' | 'language' | 'ai'`
   - แยก content เป็น 3 panel sections
   - ขนาด modal: `w-[720px] h-[480px]` (ปรับได้)
2. ย้าย settings เดิมเข้า panel ที่ถูกต้อง:
   - Source Language, Target Language → Language panel
   - Overlay appearance → Theme panel (transparency/font size)
   - ASR Engine, Remote Endpoint, API Key → AI panel

**ไม่ต้องเปลี่ยน:** commands.rs, speech.svelte.ts, app.svelte logic

---

### Phase 2 — Theme System

**Scope:** Color themes (Day/Night/Toy) + Highlight source toggle

**Frontend tasks:**

1. เพิ่ม CSS variables ใน `app.css`:
   ```css
   :root {
   	--bg: black;
   	--text: white;
   	--text-dim: rgba(255, 255, 255, 0.4);
   }
   [data-theme="day"] {
   	--bg: #f5f5f5;
   	--text: #111;
   	--text-dim: rgba(0, 0, 0, 0.4);
   }
   [data-theme="toy"] {
   	--bg: #1a0533;
   	--text: #ff79c6;
   	--text-dim: rgba(255, 121, 198, 0.4);
   }
   ```
2. ใน `app.svelte`: เพิ่ม `theme = $state('night')`, apply `data-theme={theme}` บน root div
3. `ThemePanel.svelte`: 3 color swatch buttons (Day/Night/Toy) + toggle switch component
4. `SubtitleLine.svelte`: รับ `highlightSource` prop, apply font-weight/opacity ต่างกัน

**Backend tasks:**

1. `commands.rs`: เพิ่ม `theme` และ `highlight_source` field ใน `Settings` struct
2. `settings_get` / `settings_set`: handle new keys

---

### Phase 3 — Language: Second Translation

**Scope:** เพิ่ม second translation language

**Frontend tasks:**

1. `app.svelte`: เพิ่ม `targetLang2 = $state('none')`, เพิ่ม `translations2` state
2. `$effect` ใน app.svelte: เมื่อ `targetLang2 !== 'none'` → เรียก translateLine สำหรับ lang 2 ด้วย
3. `SubtitleLine.svelte`: รับ `translation2` prop, แสดง row ที่ 3
4. `LanguagePanel.svelte`: Second Translate dropdown (รวม "None" option)

**Backend tasks:**

1. `commands.rs`: เพิ่ม `target_lang_2` ใน Settings struct + settings_get/set

---

### Phase 4 — AI Panel: STT Providers

**Scope:** เปลี่ยน ASR Engine selector ให้รองรับ Gemini + OpenAI-compatible custom URL

**Frontend tasks:**

1. `speech.svelte.ts`:
   - เพิ่ม engine types: `'openai-compatible'` (alias สำหรับ remote แต่ชื่อชัดกว่า), `'gemini'`
   - เพิ่ม `sttEndpoint` และ `sttApiKey` state (rename จาก `remoteEndpoint` / `apiKey`)
2. `AiPanel.svelte`: Model dropdown + conditional URL/API key fields
3. `app.svelte`: update props ที่ส่งไป Settings

**Backend tasks:**

1. `remote_asr.rs`: รองรับ Gemini API endpoint format ถ้าต่างจาก OpenAI-compatible
2. `commands.rs`: rename `remote_api_key_name` → `stt_api_key_name` (backward-compatible default)

---

### Phase 5 — AI Panel: Translation Providers + Real Implementation

**Scope:** Translation ที่ทำงานจริงผ่าน OpenAI-compatible API / DeepInfra / Ollama

**Frontend tasks:**

1. `speech.svelte.ts` → `translate()`: ปัจจุบัน stub, implement:
   ```ts
   // fetch to translation endpoint
   const res = await fetch(endpoint + "/chat/completions", {
   	method: "POST",
   	headers: { Authorization: `Bearer ${key}`, "Content-Type": "application/json" },
   	body: JSON.stringify({
   		model,
   		messages: [{ role: "user", content: `Translate to ${targetLang}: ${text}` }],
   	}),
   });
   ```
2. `stronghold.ts`: เพิ่ม `setTranslationApiKey` / `getTranslationApiKey` (Stronghold key: `"translation"`)

**Backend tasks (optional — ถ้า CORS เป็นปัญหา):**

1. `commands.rs`: เพิ่ม `translate_remote` command ที่ใช้ reqwest POSTงาน

**CSP tasks:**

1. `tauri.conf.json`: เปลี่ยน CSP `connect-src` ให้รองรับ custom domain
   - ปัจจุบัน: `connect-src https://api.deepinfra.com`
   - ใหม่: ต้องอนุญาต `https:` หรือ user-provided URL → ต้องระวัง security

---

## ลำดับการพัฒนาแนะนำ

```
Phase 1 (UI Shell)     → Phase 3 (Second Lang)  → Phase 2 (Theme)
     ↓
Phase 4 (STT AI)       → Phase 5 (Translation AI)
```

Phase 1 ก่อนเพราะ: ทุก phase ต้องใช้ Settings modal ใหม่ และ Phase 1 ไม่ break อะไรเลย

---

## ประเด็นที่ต้องตัดสินใจก่อน (Design Decisions Needed)

| #   | ประเด็น                                                                        | ตัวเลือก                                   | default แนะนำ                           |
| --- | ------------------------------------------------------------------------------ | ------------------------------------------ | --------------------------------------- |
| D1  | "online/offline" toggle: เก็บหรือลบ?                                           | เก็บ (explicit mode) / ลบ (model implied)  | ลบ                                      |
| D2  | Gemini STT: ใช้ได้จริงหรือ future?                                             | implement ใน Phase 4 / placeholder         | placeholder ก่อน                        |
| D3  | Translation ทำใน frontend (fetch) หรือ Rust (reqwest)?                         | Frontend fetch / Rust command              | Frontend fetch (ง่ายกว่า)               |
| D4  | CSP policy สำหรับ custom endpoint: allow all https หรือ user สามารถ whitelist? | allow https: / specific domains            | allow https: (Tauri v2 sandbox ช่วยได้) |
| D5  | Overlay transparency และ font size อยู่ใน Theme หรือปล่อยใน AI panel?          | Theme (Display) / แยก "Display" menu เพิ่ม | Theme/Display panel                     |
| D6  | Modal size บน screen เล็ก (800×600)?                                           | 720×480 / 680×460 / 90%×80%                | `min(720px, 90vw)` × `min(480px, 85vh)` |

---

## Files ที่ต้องแก้ (สรุป)

### Frontend

- `src/components/Settings.svelte` — redesign ทั้งหมด (หรือ split เป็น sub-components)
- `src/app.svelte` — เพิ่ม theme, targetLang2, translations2, STT endpoint/key rename
- `src/app.css` — CSS variables สำหรับ theme system
- `src/components/SubtitleLine.svelte` — รองรับ translation2, highlightSource prop
- `src/lib/speech.svelte.ts` — engine types, sttEndpoint/apiKey rename, translate() implement
- `src/lib/stronghold.ts` — key สำหรับ translation API
- `src/lib/types.ts` — เพิ่ม theme types

### Backend (Rust)

- `src-tauri/src/commands.rs` — Settings struct fields ใหม่ (theme, highlight_source, target_lang_2, translation_model, translation_endpoint, stt_api_key_name)
- `src-tauri/tauri.conf.json` — CSP เปลี่ยน connect-src

---

## Mock/Preview UI Layout

```
┌─────────────────────────────────────────────────────┐
│  Settings                                    [✕]    │
├──────────────┬──────────────────────────────────────┤
│              │                                      │
│  ● Theme     │  AI                                  │
│  ○ Language  │  ─────────────────────────────────   │
│  ○ AI        │  Speech to Text                      │
│              │                                      │
│              │  Model  [OpenAI Compatible      ▾]   │
│              │  URL    [https://api.example... ]   │
│              │  API Key [sk-••••••••••••••••  ]    │
│              │                                      │
│              │  ─────────────────────────────────   │
│              │  Translation                         │
│              │                                      │
│              │  Provider [DeepInfra            ▾]   │
│              │  Model    [Qwen/Qwen3-235B-A22B ]   │
│              │  URL      [https://api.deepin...  ] │
│              │  API Key  [sk-••••••••••••••••  ]   │
│              │                                      │
└──────────────┴──────────────────────────────────────┘
```

---

_Created: 2026-04-14_
