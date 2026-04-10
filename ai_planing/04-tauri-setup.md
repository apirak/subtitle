# Tauri v2 Setup

## 1. Initialization

Create the Tauri v2 project on top of the Svelte frontend:

```bash
# From the subtitle/ directory, after setting up Svelte
pnpm add -D @tauri-apps/cli@latest
pnpm tauri init
```

During `tauri init`:
- App name: `Real-time Subtitles`
- Window title: `Real-time Subtitles`
- Frontend dev URL: `http://localhost:5173`
- Frontend dist dir: `../dist`
- Dev command: `pnpm dev`
- Build command: `pnpm build`

## 2. Configuration (`src-tauri/tauri.conf.json`)

Key settings:

```json
{
  "app": {
    "windows": [
      {
        "title": "Real-time Subtitles",
        "width": 800,
        "height": 600,
        "resizable": true,
        "fullscreen": false,
        "decorations": true
      }
    ],
    "security": {
      "csp": "default-src 'self'; connect-src https://api.deepinfra.com"
    }
  }
}
```

**CSP note:** Only allow connections to `api.deepinfra.com` for translation.

## 3. Permissions (`src-tauri/capabilities/default.json`)

Minimal permissions — this app only needs microphone access (handled by the browser/webview):

```json
{
  "identifier": "default",
  "description": "Default permissions for the app",
  "windows": ["main"],
  "permissions": [
    "core:default"
  ]
}
```

No special Tauri commands needed yet. All logic runs in the frontend.

## 4. WebSpeech API in Tauri

Tauri uses the system webview (WebKit on macOS, WebView2 on Windows).

**macOS (WebKit):** WebSpeech API works natively.
**Windows (WebView2/Edge):** WebSpeech API works via Chromium.
**Linux (WebKitGTK):** May have limited support — test needed.

If WebSpeech is unavailable, the app shows the existing error message:
> "Web Speech API is not supported in this browser."

## 5. Environment Variables

In Tauri, Vite env variables work the same way via `import.meta.env.VITE_*`.

The `.env` file at project root:
```
VITE_DEEPINFRA_API_KEY=your_key_here
```

For production builds, consider using Tauri's build-time env injection for security.

## 6. Build Commands

```json
{
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "tauri": "tauri",
    "tauri:dev": "tauri dev",
    "tauri:build": "tauri build"
  }
}
```

## 7. Future Tauri Features (not in scope)

These could be added later using Tauri's Rust backend:
- Export subtitles to .srt/.vtt file
- System tray for quick start/stop
- Global hotkey to toggle listening
- Always-on-top floating subtitle window
