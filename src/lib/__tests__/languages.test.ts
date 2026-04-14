import { describe, expect, it } from 'vitest';
import { getLangName, SOURCE_LANGUAGES, TARGET_LANGUAGES } from '../languages';

describe('SOURCE_LANGUAGES', () => {
  it('should contain expected languages', () => {
    const codes = SOURCE_LANGUAGES.map((l) => l.code);
    expect(codes).toContain('en-US');
    expect(codes).toContain('th-TH');
    expect(codes).toContain('zh-CN');
    expect(codes).toContain('ja-JP');
    expect(codes).toContain('ko-KR');
  });

  it('should have valid locale format codes', () => {
    SOURCE_LANGUAGES.forEach((lang) => {
      expect(lang.code).toMatch(/^[a-z]{2}-[A-Z]{2}$/);
      expect(lang.label.length).toBeGreaterThan(0);
    });
  });

  it('should have unique codes', () => {
    const codes = SOURCE_LANGUAGES.map((l) => l.code);
    const uniqueCodes = new Set(codes);
    expect(uniqueCodes.size).toBe(codes.length);
  });
});

describe('TARGET_LANGUAGES', () => {
  it('should contain common target languages', () => {
    const values = TARGET_LANGUAGES.map((l) => l.value);
    expect(values).toContain('en');
    expect(values).toContain('th');
    expect(values).toContain('zh');
    expect(values).toContain('ja');
  });

  it('should have short code format (2 chars)', () => {
    TARGET_LANGUAGES.forEach((lang) => {
      expect(lang.value).toMatch(/^[a-z]{2}$/);
      expect(lang.label.length).toBeGreaterThan(0);
    });
  });

  it('should have unique values', () => {
    const values = TARGET_LANGUAGES.map((l) => l.value);
    const uniqueValues = new Set(values);
    expect(uniqueValues.size).toBe(values.length);
  });
});

describe('getLangName', () => {
  it('should return language name for valid codes', () => {
    expect(getLangName('en')).toBe('English');
    expect(getLangName('th')).toBe('Thai');
    expect(getLangName('zh')).toBe('Chinese');
    expect(getLangName('ja')).toBe('Japanese');
  });

  it('should return the code itself for unknown codes', () => {
    expect(getLangName('xyz')).toBe('xyz');
    expect(getLangName('invalid')).toBe('invalid');
  });

  it('should handle mixed case input', () => {
    expect(getLangName('EN')).toBe('EN');
    expect(getLangName('Th')).toBe('Th');
  });
});

describe('language code consistency', () => {
  it('should have source language codes that start with target language equivalents', () => {
    const targetCodes = new Set(TARGET_LANGUAGES.map((l) => l.value));
    SOURCE_LANGUAGES.forEach((src) => {
      const base = src.code.split('-')[0];
      expect(targetCodes.has(base)).toBe(true);
    });
  });
});
