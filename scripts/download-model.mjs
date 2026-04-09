#!/usr/bin/env node
/**
 * Download Whisper model files to public/models/ for local serving.
 * Usage: node scripts/download-model.mjs
 */

const MODEL_ID = 'onnx-community/whisper-base';
const BASE_URL = `https://huggingface.co/${MODEL_ID}/resolve/main`;
const OUTPUT_DIR = 'public/models/onnx-community/whisper-base';

const FILES = [
  'config.json',
  'generation_config.json',
  'preprocessor_config.json',
  'tokenizer.json',
  'tokenizer_config.json',
  'onnx/encoder_model.onnx',
  'onnx/decoder_model_merged.onnx',
];

import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';

async function downloadFile(file) {
  const url = `${BASE_URL}/${file}`;
  const outPath = join(OUTPUT_DIR, file);
  await mkdir(dirname(outPath), { recursive: true });

  console.log(`Downloading ${file}...`);
  const res = await fetch(url);
  if (!res.ok) throw new Error(`Failed to download ${url}: ${res.status}`);

  const buffer = Buffer.from(await res.arrayBuffer());
  await writeFile(outPath, buffer);
  const sizeMB = (buffer.length / 1024 / 1024).toFixed(1);
  console.log(`  ✓ ${file} (${sizeMB} MB)`);
}

async function main() {
  console.log(`Downloading model: ${MODEL_ID}`);
  console.log(`Output: ${OUTPUT_DIR}\n`);

  for (const file of FILES) {
    await downloadFile(file);
  }

  console.log('\n✅ All model files downloaded successfully!');
  console.log('Model will be served from /models/onnx-community/whisper-base/');
}

main().catch((e) => {
  console.error('❌ Download failed:', e.message);
  process.exit(1);
});
