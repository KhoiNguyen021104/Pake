import { describe, expect, it, vi, beforeEach } from 'vitest';
import logger from '../../bin/options/logger';
import { validateWindowSpecs, MAIN_WINDOW_LABEL } from '../../bin/utils/window';

describe('validateWindowSpecs', () => {
  beforeEach(() => {
    vi.spyOn(logger, 'warn').mockImplementation(() => undefined);
    vi.spyOn(logger, 'error').mockImplementation(() => undefined);
  });

  it('rejects reserved main label', () => {
    expect(() =>
      validateWindowSpecs(
        [`${MAIN_WINDOW_LABEL}=/dashboard`],
        'https://a.com',
        {
          exitOnError: false,
        },
      ),
    ).toThrow(/reserved for the main window/);
  });

  it('rejects duplicate labels', () => {
    expect(() =>
      validateWindowSpecs(
        ['camera=/camera', 'camera=/monitor'],
        'https://my-web.com/dashboard',
        { exitOnError: false },
      ),
    ).toThrow(/Duplicate window label/);
  });

  it('rejects invalid label characters', () => {
    expect(() =>
      validateWindowSpecs(['cam_era=/camera'], 'https://my-web.com', {
        exitOnError: false,
      }),
    ).toThrow(/Invalid window label/);
  });

  it('warns on cross-origin full URL', () => {
    const specs = validateWindowSpecs(
      ['camera=https://other.com/camera'],
      'https://my-web.com/dashboard',
      { exitOnError: false },
    );
    expect(specs).toHaveLength(1);
    expect(logger.warn).toHaveBeenCalledWith(
      expect.stringContaining('differs from main URL origin'),
    );
  });
});
