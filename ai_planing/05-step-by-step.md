# Step-by-Step Migration Guide

## Phase 1: Scaffold New Project

### 1.1 Create Svelte project
```bash
cd /Users/apirak/workspace/subtitle
# Create a temporary Svelte project to copy config from
pnpm create @tauri-apps/app subtitle-temp --template svelte-ts
```

Or manually:
```bash
pnpm add -D svelte @sveltejs/vite-plugin-svelte typescript
pnpm add -D @tauri-apps/cli@latest @tauri-apps/api@latest
```

### 1.2 Key config files to create

**vite.config.ts**
```typescript
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  envPrefix: ['VITE_', 'TAURI_'],
});
```

**svelte.config.js**
```javascript
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
export default { preprocess: vitePreprocess() };
```

**tsconfig.json** — standard Svelte + TypeScript config

### 1.3 Create directory structure
```bash
mkdir -p src/lib src/components static
```

---

## Phase 2: Implement Core Logic

### 2.1 Types (`src/lib/types.ts`)
- Simple: `AppStatus`, `SubtitleLine`
- No Worker types, no Engine type

### 2.2 Languages (`src/lib/languages.ts`)
- Copy language lists from current code
- Merge source + target into one file
- Remove Whisper-specific language list

### 2.3 Speech module (`src/lib/speech.svelte.ts`)
- Port `useWebSpeechApi.ts` logic to Svelte 5 class with runes
- Remove all `console.log` debug lines (clean start)
- Keep all error handling behavior

---

## Phase 3: Build UI Components

Build bottom-up (smallest first):

### 3.1 `Dropdown.svelte`
- Props: `value`, `options: {value: string, label: string}[]`
- Emits `onchange` event
- Scoped styles (dark theme, custom arrow)

### 3.2 `SubtitleLine.svelte`
- Props: `text`, `translation?`, `isTranslating`, `isLast`
- Handles "···" animation
- Handles fade for older lines

### 3.3 `IdleScreen.svelte`
- Title, config display, start button, settings button
- Props: `sourceLabel`, `targetLabel`, `onStart`, `onSettings`

### 3.4 `ListeningScreen.svelte`
- Status bar + subtitle container
- Uses `SubtitleLine`

### 3.5 `ErrorScreen.svelte`
- Error message + retry button

### 3.6 `Settings.svelte`
- Bottom sheet overlay
- Uses `Dropdown` × 2 (source lang, target lang)
- Slider for position
- **No engine dropdown** (WebSpeech only)

---

## Phase 4: Wire It All Together

### 4.1 `app.svelte`
- Import speech module and all components
- State management with runes
- Translation logic as plain function
- Screen switching based on `speech.status`

### 4.2 `app.css`
- Global reset
- CSS custom properties
- Animation keyframes

### 4.3 `main.ts`
- Mount App to `document.body`

---

## Phase 5: Tauri Integration

### 5.1 Initialize Tauri
```bash
pnpm tauri init
```

### 5.2 Configure
- Set up `tauri.conf.json`
- Set up capabilities/permissions
- Add app icons

### 5.3 Test
```bash
pnpm tauri dev
```

### 5.4 Build
```bash
pnpm tauri build
```

---

## Phase 6: Cleanup

- [ ] Delete `src-tauri/` from old Preact files
- [ ] Delete `public/models/` (Whisper models ~276MB)
- [ ] Delete `scripts/download-model.mjs`
- [ ] Delete old `src/` Preact files
- [ ] Remove `@huggingface/transformers` from dependencies
- [ ] Remove `preact` and `@preact/preset-vite` from dependencies
- [ ] Update `.gitignore`
- [ ] Test all features work in Tauri window
- [ ] Test WebSpeech works in Tauri webview
- [ ] Test translation still works

---

## Migration Order (What to Build First)

```
1. types.ts + languages.ts          (no dependencies)
2. speech.svelte.ts                 (depends on types)
3. app.css                          (global styles)
4. Dropdown.svelte                  (standalone component)
5. SubtitleLine.svelte              (standalone component)
6. IdleScreen.svelte                (uses Dropdown concept)
7. ListeningScreen.svelte           (uses SubtitleLine)
8. ErrorScreen.svelte               (simple)
9. Settings.svelte                  (uses Dropdown)
10. app.svelte                      (ties everything together)
11. main.ts                         (entry point)
12. Tauri setup + config            (desktop wrapper)
```
