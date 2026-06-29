import { describe, expect, it } from 'vitest';
import {
  parseWindowSpec,
  resolveWindowUrl,
  isValidWindowLabel,
  normalizeMsysMangledPath,
  MAIN_WINDOW_LABEL,
} from '../../bin/utils/window';

describe('parseWindowSpec', () => {
  it('parses label and path', () => {
    expect(parseWindowSpec('camera=/camera')).toEqual({
      label: 'camera',
      path: '/camera',
    });
  });

  it('rejects missing path', () => {
    expect(() => parseWindowSpec('camera=')).toThrow(/Path must not be empty/);
  });

  it('rejects missing label', () => {
    expect(() => parseWindowSpec('=/camera')).toThrow(
      /Invalid --window format/,
    );
  });
});

describe('resolveWindowUrl', () => {
  it('resolves relative path against base URL', () => {
    expect(resolveWindowUrl('https://my-web.com/dashboard', '/camera')).toBe(
      'https://my-web.com/camera',
    );
  });

  it('passes through full URL with same origin', () => {
    expect(
      resolveWindowUrl(
        'https://my-web.com/dashboard',
        'https://my-web.com/camera',
      ),
    ).toBe('https://my-web.com/camera');
  });
});

describe('normalizeMsysMangledPath', () => {
  it('recovers /live from Git Bash path conversion', () => {
    expect(normalizeMsysMangledPath('c:/Program%20Files/Git/live')).toBe(
      '/live',
    );
    expect(normalizeMsysMangledPath('C:/Program Files/Git/live')).toBe('/live');
  });

  it('leaves normal paths unchanged', () => {
    expect(normalizeMsysMangledPath('/live')).toBe('/live');
    expect(normalizeMsysMangledPath('https://cloud-camera.beex.vn/live')).toBe(
      'https://cloud-camera.beex.vn/live',
    );
  });
});

describe('isValidWindowLabel', () => {
  it('accepts simple nouns', () => {
    expect(isValidWindowLabel('camera')).toBe(true);
    expect(isValidWindowLabel('my-monitor')).toBe(true);
  });

  it('rejects reserved main label pattern issues', () => {
    expect(isValidWindowLabel(MAIN_WINDOW_LABEL)).toBe(true);
    expect(isValidWindowLabel('-camera')).toBe(false);
    expect(isValidWindowLabel('camera-')).toBe(false);
  });
});
