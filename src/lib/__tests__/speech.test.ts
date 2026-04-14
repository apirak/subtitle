import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createSpeechForTest } from '../speech.svelte';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
  type: { UnlistenFn: 'UnlistenFn' as any },
}));

describe('Speech state management', () => {
  let speech: ReturnType<typeof createSpeechForTest>;

  beforeEach(() => {
    vi.clearAllMocks();
    speech = createSpeechForTest();
  });

  afterEach(() => {
    speech.destroy();
  });

  describe('initial state', () => {
    it('should have idle status initially', () => {
      expect(speech.status).toBe('idle');
    });

    it('should have empty subtitles initially', () => {
      expect(speech.subtitles).toEqual([]);
    });

    it('should have default language as en-US', () => {
      expect(speech.language).toBe('en-US');
    });

    it('should have browser engine by default', () => {
      expect(speech.engine).toBe('browser');
    });

    it('should have no error message initially', () => {
      expect(speech.errorMessage).toBe('');
    });
  });

  describe('addInterimSubtitle', () => {
    it('should add interim subtitle with interim- prefix', () => {
      speech.addInterimSubtitle('abc', 'Hello world');
      expect(speech.subtitles).toHaveLength(1);
      expect(speech.subtitles[0].id).toBe('interim-abc');
      expect(speech.subtitles[0].text).toBe('Hello world');
    });

    it('should replace existing interim with same base id', () => {
      speech.addInterimSubtitle('abc', 'Hello');
      speech.addInterimSubtitle('abc', 'Hello world');
      const interimSubtitles = speech.subtitles.filter((s) => s.id === 'interim-abc');
      expect(interimSubtitles).toHaveLength(1);
      expect(speech.subtitles[0].text).toBe('Hello world');
    });
  });

  describe('addFinalSubtitle', () => {
    it('should add final subtitle with original id', () => {
      speech.addFinalSubtitle('final-123', 'Final text');
      expect(speech.subtitles).toHaveLength(1);
      expect(speech.subtitles[0].id).toBe('final-123');
      expect(speech.subtitles[0].text).toBe('Final text');
    });

    it('should remove interim subtitle with same base id', () => {
      speech.addInterimSubtitle('abc', 'Interim text');
      expect(speech.subtitles).toHaveLength(1);
      speech.addFinalSubtitle('abc', 'Final text');
      expect(speech.subtitles).toHaveLength(1);
      expect(speech.subtitles[0].id).toBe('abc');
      expect(speech.subtitles[0].text).toBe('Final text');
    });
  });

  describe('MAX_SUBTITLES limit', () => {
    it('should limit subtitles to 12 lines', () => {
      for (let i = 0; i < 15; i++) {
        speech.addFinalSubtitle(`id-${i}`, `Text ${i}`);
      }
      expect(speech.subtitles).toHaveLength(12);
      expect(speech.subtitles[0].id).toBe('id-3');
    });
  });

  describe('setError', () => {
    it('should set error message', () => {
      speech.setError('Test error');
      expect(speech.errorMessage).toBe('Test error');
    });

    it('should set status to error', () => {
      speech.setError('Test error');
      expect(speech.status).toBe('error');
    });
  });

  describe('engine state', () => {
    it('should update engine', () => {
      speech.engine = 'remote';
      expect(speech.engine).toBe('remote');
    });

    it('should update remote endpoint', () => {
      speech.remoteEndpoint = 'https://api.example.com';
      expect(speech.remoteEndpoint).toBe('https://api.example.com');
    });

    it('should update api key', () => {
      speech.apiKey = 'sk-test-key';
      expect(speech.apiKey).toBe('sk-test-key');
    });
  });

  describe('language setting', () => {
    it('should update language', () => {
      speech.language = 'th-TH';
      expect(speech.language).toBe('th-TH');
    });
  });

  describe('appendSubtitle via public API', () => {
    it('should remove interim subtitle when final with same base id is added', () => {
      speech.addInterimSubtitle('abc', 'Interim text');
      expect(speech.subtitles.find((s) => s.id === 'interim-abc')?.text).toBe('Interim text');

      speech.addFinalSubtitle('abc', 'Final text');
      expect(speech.subtitles).toHaveLength(1);
      expect(speech.subtitles[0].id).toBe('abc');
      expect(speech.subtitles[0].text).toBe('Final text');
      expect(speech.subtitles.find((s) => s.id === 'interim-abc')).toBeUndefined();
    });

    it('should keep other subtitles when removing one', () => {
      speech.addFinalSubtitle('id1', 'First');
      speech.addFinalSubtitle('id2', 'Second');
      speech.addInterimSubtitle('id1', 'Updated interim');

      expect(speech.subtitles).toHaveLength(3);
      expect(speech.subtitles.find((s) => s.id === 'interim-id1')?.text).toBe('Updated interim');
      expect(speech.subtitles.find((s) => s.id === 'id1')?.text).toBe('First');
      expect(speech.subtitles.find((s) => s.id === 'id2')?.text).toBe('Second');
    });
  });
});
