# STT + RMS Flow (Browser, Gemini, Vosk, OpenAI-Compatible)

เอกสารนี้อธิบายการทำงานของการจับเสียง, RMS gating, การตัด chunk และการส่ง STT สำหรับแต่ละ engine ในโปรเจกต์

## ภาพรวมแบบเร็ว

- จุดเริ่มต้นของ Vosk/Gemini/OpenAI-compatible คือ audio capture จากไมค์ผ่าน Rust
- Browser engine ใช้ Web Speech API โดยตรง (ไม่เริ่ม Rust audio capture)
- สำหรับ Vosk/Gemini/OpenAI-compatible เสียงจะถูก normalize เป็น 16kHz mono
- Browser และ Vosk มี behavior ต่างจาก Gemini/OpenAI-compatible ตรงที่การแบ่งคำพูดและการปล่อยผลลัพธ์
- Gemini/OpenAI-compatible ใช้ buffer + RMS/VAD gate ก่อนส่งขึ้น API

## Mermaid: End-to-End Flow

```mermaid
flowchart TD
    Start[Mic Input] --> Pick{Engine}

    Pick -->|Browser| B1[Web Speech API]
    B1 --> B2[Browser VAD and segmentation]
    B2 --> B3[Interim and final text]
    B3 --> UI[Subtitle UI]

    Pick -->|Vosk| V0[Rust capture]
    V0 --> V1[Resample 16k mono]
    V1 --> V2[Vosk recognition]
    V2 --> V3[Emit subtitle events]
    V3 --> UI

    Pick -->|Gemini or OpenAI| R0[Rust capture]
    R0 --> R1[Resample 16k mono]
    R1 --> R2[Buffer audio]
    R2 --> R3{Flush now}
    R3 -->|No| R2
    R3 -->|Yes| R4{RMS pass}
    R4 -->|No| D1[Drop chunk]
    R4 -->|Yes| W1[Encode WAV]

    W1 --> P1{Provider}
    P1 -->|Gemini| G1[POST generateContent]
    G1 --> G2[Parse text]
    G2 --> G3[Emit subtitle]
    G3 --> UI

    P1 -->|OpenAI compatible| O1[POST audio transcriptions]
    O1 --> O2[Parse text]
    O2 --> O3[Emit subtitle]
    O3 --> UI
```

## พฤติกรรมแต่ละ Engine

### Browser

- Recognition รันใน webview ผ่าน Web Speech API
- Browser จัดการ segmentation/interim/final เอง
- เหมาะกับการเริ่มใช้งานเร็ว แต่ควบคุม low-level audio pipeline ได้น้อย

### Vosk

- On-device recognition ใน Rust
- ส่งผลผ่าน event backend://subtitle/update และ backend://subtitle/final
- เหมาะกับงานออฟไลน์และ latency คงที่

### Gemini (Batch)

- ใช้ Rust buffer/chunk แล้วส่ง WAV ไปที่ generateContent
- parse ข้อความจาก candidates.content.parts[].text
- ปัจจุบันเน้น final subtitle ต่อ chunk

### OpenAI-Compatible

- ใช้ Rust buffer/chunk แล้วส่ง multipart ไป /v1/audio/transcriptions
- อ่านผลจาก text/transcript ตาม response schema

## RMS + Chunking ที่ใช้อยู่ตอนนี้

- max chunk: 3 วินาที
- silence duration: 300ms
- Gemini default min_speech_rms: 0.006 (ไวขึ้นสำหรับไมค์ที่สัญญาณเบา)
- OpenAI-compatible default min_speech_rms: 0.04
- Gemini min silence flush samples: 48000 (ประมาณ 3 วินาที)

หมายเหตุ: สามารถ override ค่า min_speech_rms ผ่าน Settings ใน UI ได้แล้ว โดยค่าจะถูก persist ลง settings store

## การตั้งค่า RMS ผ่าน UI (Live Tuning)

- เมนู: Settings -> Speech To Text
- ฟิลด์: Speech RMS Threshold
- ช่วงค่าที่แนะนำ: 0.006 ถึง 0.020
- ค่านี้ถูกบันทึกใน settings key: remote_min_speech_rms
- เมื่อกด Start รอบใหม่ ระบบจะอ่านค่าแล้วใช้แทนค่า default ของ provider

ข้อควรระวัง

- ค่าต่ำเกินไปอาจติดเสียงรบกวนและ false positives
- ค่าสูงเกินไปอาจทำให้ไม่ส่ง chunk ไป STT (เห็น log ว่า skipping below threshold)

## แนวทางจูนค่า

- ถ้าข้อความไม่ขึ้นเลย: ลด min_speech_rms
- ถ้ามีคำขยะหรือติดเสียงรบกวน: เพิ่ม min_speech_rms
- ถ้าต้องการจับประโยคยาวขึ้น: รักษา chunk ที่ ~3 วินาที และหลีกเลี่ยง silence flush ที่ถี่เกินไป

ช่วงค่าที่แนะนำสำหรับ Gemini

- ห้องเงียบ/เสียงเบา: 0.006 ถึง 0.010
- ทั่วไป: 0.010 ถึง 0.020
- มี noise มาก: 0.020 ขึ้นไป

## จุดสังเกตจาก Log สำหรับ Debug

- Remote ASR: max chunk reached ... rms=...
- Remote ASR: skipping ... below threshold ...
- send_gemini_batch_transcription: response status=...
- send_gemini_batch_transcription: extracted text ...

ถ้าเห็น max chunk แต่ตามด้วย skipping ตลอด แปลว่าต้องลด threshold หรือเพิ่ม gain ไมค์

## โค้ดอ้างอิงสำคัญ

- Browser ไม่ใช้ Rust capture: src/lib/speech.svelte.ts
- Rust audio capture entrypoint: src-tauri/src/audio.rs
- Remote/Gemini chunking + RMS gate: src-tauri/src/remote_asr.rs
- Settings schema/persistence: src-tauri/src/commands.rs
- Settings UI panel: src/components/setting/panels/SttSettingsPanel.svelte
