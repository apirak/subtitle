# Vertical Slide Settings UI Plan

## Slice 1: Foundation

เพิ่ม Settings button + split-layout modal (menu + panel)

✅ Add button in IdleScreen / header to open Settings
✅ Create SettingsModal.svelte with sidebar menu (4 placeholders: Display, Language, STT, Translate)
✅ Create SettingsPanel.svelte (empty panel for content)
✅ Wire settingsOpen state in app.svelte
✅ Keep old Settings.svelte as SettingsLegacy.svelte (reference backup)
🧪 Test: pnpm typecheck:native pass, modal opens/closes

## Slice 2: Menu Navigation

ทำให้ menu click ได้ (routing state)

✅ Add activeSection state in app.svelte (type: "display" | "language" | "stt" | "translate")
✅ Highlight active menu item in SettingsModal
✅ Pass activeSection to SettingsPanel (still empty)
🧪 Test: pnpm typecheck:native pass, menu click toggle activeSection

## Slice 3: Theme Section

Display section (theme selector)

✅ Create theme state: $state let theme = "night"
✅ Render 3 theme swatches in SettingsPanel (when activeSection === "display")
✅ Handle click → update theme state
✅ Add theme CSS variables to app.css
✅ Scope note: this slice only covers theme selection + persistence from Settings
🧪 Test: pnpm typecheck:native pass, theme selector shows + updates

## Slice 4: Language Section

Language dropdowns (source + translate-to)

✅ Add targetLang state (already exists, but clean it up)
✅ Render dropdowns in SettingsPanel (when activeSection === "language")
✅ Handle language change
🧪 Test: pnpm typecheck:native pass, languages dropdown works

## Slice 5: STT Section

Speech-to-Text engine + endpoint config

✅ Render engine dropdown + conditional endpoint/key inputs
✅ Handle engine selection
🧪 Test: pnpm typecheck:native pass, STT section loads + switches engine type

## Slice 6: Translate Section

Translation provider + model + endpoint

✅ Render provider dropdown + model/endpoint/key inputs
✅ Connect to speech.translate() HTTP logic (already exists)
🧪 Test: pnpm typecheck:native pass, translation config saves

## Slice 7: Apply Theme To Screens

Apply selected theme to IdleScreen and ListeningScreen

✅ Use the saved theme state to drive visual styling in IdleScreen
✅ Use the saved theme state to drive visual styling in ListeningScreen
✅ Keep Settings modal as a control surface, not the main theme preview target
✅ Ensure each theme has readable contrast for text, badges, controls, and background layers
✅ Keep theme tokens centralized in app.css and consume them from screen components
🧪 Test: pnpm typecheck:native pass, switching theme updates IdleScreen and ListeningScreen correctly

🎯 Best Practices Applied Throughout
✅ One state change per slice
✅ Props interface explicit (no magic)
✅ Component responsibility single (SettingsModal = nav, SettingsPanel = content)
✅ TypeScript strict: no any, explicit types
✅ No unused imports/variables
✅ Each slice is "shippable" (works independently)
