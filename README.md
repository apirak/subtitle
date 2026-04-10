# Subtitle — Real-time Captioning for Live Events

Open-source real-time caption and translation software for live events on stage.

Displays live subtitles with instant translation — built for conferences, talks, performances, and any live event where the audience needs to read what's being said.

An open source alternative to [stagecaptions.io](https://stagecaptions.io).

## Features

- **Live Speech-to-Text** — Continuous recognition using the Web Speech API with interim results
- **Real-time Translation** — Translates captions on the fly using DeepInfra (Qwen3-14B), with context from previous lines for better accuracy
- **14 Languages** — Source and target support for Thai, English, Chinese, Japanese, Korean, Spanish, French, German, Portuguese, Russian, Arabic, Hindi, Vietnamese, and Indonesian
- **Stage-ready Display** — Dark background, large white text, adjustable subtitle position
- **Desktop App** — Runs as a native window via Tauri, with always-on-top support

## Tech Stack

| Layer | Technology |
|---|---|
| UI | Svelte 5 (Runes) |
| Desktop | Tauri v2 (Rust) |
| Build | Vite 8 |
| Speech Recognition | Web Speech API |
| Translation | DeepInfra API (Qwen3-14B) |
| Language | TypeScript |

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/tools/install) (for Tauri desktop build)

### Install

```bash
git clone https://github.com/apirak/subtitle.git
cd subtitle
pnpm install
```

### Configure

Copy the environment file and add your DeepInfra API key (required for translation):

```bash
cp .env.example .env
```

Edit `.env`:

```
VITE_DEEPINFRA_API_KEY=your_deepinfra_api_key
```

Get a free API key at [deepinfra.com](https://deepinfra.com).

### Run

```bash
# Development (web)
pnpm dev

# Development (desktop app)
pnpm tauri:dev

# Production build
pnpm tauri:build
```

## How to Use

1. **Launch the app** — run `pnpm tauri:dev` or open the built app
2. **Configure settings** — choose your source language (the speaker's language) and target translation language
3. **Start listening** — click the start button to begin capturing speech
4. **Position the window** — drag the app window to a secondary display or projector. Use always-on-top to keep it visible
5. **Adjust subtitle position** — use the position slider to move subtitles up or down on screen

## Contributing

Contributions are welcome! Here's how to get started:

1. **Fork** the repository
2. **Create a branch** — `git checkout -b feature/your-feature`
3. **Make your changes** — follow the existing code style
4. **Test locally** — `pnpm tauri:dev` to verify everything works
5. **Submit a pull request** — describe what you changed and why

### Project Structure

```
src/
├── components/          # Svelte UI components
│   ├── IdleScreen       # Start screen
│   ├── ListeningScreen  # Active caption display
│   ├── SubtitleLine     # Single subtitle line with translation
│   ├── Settings         # Language & position settings
│   ├── Dropdown         # Reusable dropdown selector
│   └── ErrorScreen      # Error handling UI
├── lib/
│   ├── speech.svelte.ts # Web Speech API logic
│   ├── languages.ts     # Language definitions
│   └── types.ts         # TypeScript interfaces
├── app.svelte           # Root component
├── app.css              # Global styles
└── main.ts              # Entry point

src-tauri/               # Tauri v2 (Rust) desktop shell
```

### Ideas for Contribution

- Additional speech engines (e.g., Whisper, Google Cloud STT)
- More translation providers
- OBS integration / NDI output
- Export subtitles as SRT
- Multi-display layout options
- Theme customization

## License

MIT
