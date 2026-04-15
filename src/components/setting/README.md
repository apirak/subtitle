# SettingsSplitModal Refactor Plan

เป้าหมายคือแตก `src/components/SettingsSplitModal.svelte` ออกเป็น component ย่อยตาม section และตามโครงสร้าง modal shell เพื่อลดความซับซ้อนของไฟล์หลัก.

## Proposed Structure

```text
src/components/setting/
  README.md
  types.ts
  SettingsSplitModalShell.svelte
  SettingsSidebar.svelte
  SettingsPanelHeader.svelte
  panels/
    ThemeSettingsPanel.svelte
    LanguageSettingsPanel.svelte
    SttSettingsPanel.svelte
    TranslationSettingsPanel.svelte
  fields/
    SettingsField.svelte
    SettingsSelectField.svelte
    SettingsInputField.svelte
    SettingsRadioGroup.svelte
```

## Refactor Order

1. แยก `SettingsSidebar.svelte` และ `SettingsPanelHeader.svelte` ออกจาก shell ก่อน
2. แยกแต่ละ section เป็น panel ย่อย โดยคง callbacks และ prop names เดิมไว้
3. ประเมิน markup ที่ซ้ำกัน แล้วค่อยแยก shared field components
4. ค่อยย่อ `SettingsSplitModal.svelte` ให้เหลือ orchestration layer

## Suggested Responsibility Split

- `SettingsSplitModalShell.svelte`: overlay, modal layout, close behavior
- `SettingsSidebar.svelte`: เมนู section และ selected state
- `SettingsPanelHeader.svelte`: title และ close button
- `ThemeSettingsPanel.svelte`: theme cards
- `LanguageSettingsPanel.svelte`: source/target dropdowns
- `SttSettingsPanel.svelte`: engine selector, endpoint, api key
- `TranslationSettingsPanel.svelte`: translation provider, model, endpoint, api key

## Guardrails

- refactor UI ก่อน ไม่ย้าย business logic พร้อมกัน
- คง `$state` mirror pattern เดิมไว้ชั่วคราวเพื่อลดความเสี่ยง
- ย้าย style ที่ใช้ร่วมกันเมื่อเห็น duplication จริง ไม่แตก component เล็กเกินจำเป็น
