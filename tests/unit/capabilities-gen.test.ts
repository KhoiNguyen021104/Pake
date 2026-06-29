import path from 'path';
import fsExtra from 'fs-extra';
import { afterEach, describe, expect, it, vi } from 'vitest';

vi.mock('../../bin/utils/dir', () => ({
  npmDirectory: process.cwd(),
  tauriConfigDirectory: path.join(process.cwd(), 'src-tauri/.pake'),
}));

import {
  generateCapabilitiesFile,
  removeGeneratedCapabilitiesFile,
  buildMultiWindowConfig,
} from '../../bin/helpers/merge';
import { validateWindowSpecs } from '../../bin/utils/window';
import type { WindowConfig } from '../../bin/types';

const generatedPath = path.join(
  process.cwd(),
  'src-tauri/capabilities/generated.json',
);

const baseWindow: WindowConfig = {
  url: 'https://my-web.com/dashboard',
  url_type: 'web',
  hide_title_bar: false,
  fullscreen: false,
  maximize: false,
  width: 1200,
  height: 780,
  resizable: true,
  always_on_top: false,
  dark_mode: false,
  disabled_web_shortcuts: false,
  activation_shortcut: '',
  hide_on_close: false,
  incognito: false,
  enable_wasm: false,
  enable_drag_drop: false,
  start_to_tray: false,
  force_internal_navigation: false,
  internal_url_regex: '',
  enable_find: false,
  zoom: 100,
  min_width: 0,
  min_height: 0,
  ignore_certificate_errors: false,
  new_window: false,
};

describe('capabilities generation', () => {
  afterEach(async () => {
    await removeGeneratedCapabilitiesFile();
  });

  it('writes generated.json without webview label restrictions', async () => {
    await generateCapabilitiesFile(['pake', 'camera', 'monitor']);
    const generated = await fsExtra.readJSON(generatedPath);
    expect(generated.identifier).toBe('generated');
    expect(generated.webviews).toBeUndefined();
    expect(generated.permissions.length).toBeGreaterThan(0);
  });

  it('removes generated.json when cleaning up', async () => {
    await generateCapabilitiesFile(['pake', 'camera']);
    await removeGeneratedCapabilitiesFile();
    expect(await fsExtra.pathExists(generatedPath)).toBe(false);
  });
});

describe('buildMultiWindowConfig', () => {
  it('builds main and route windows with stable labels', () => {
    const specs = validateWindowSpecs(
      ['camera=/camera', 'monitor=/monitor'],
      'https://my-web.com/dashboard',
      { exitOnError: false },
    );
    const windows = buildMultiWindowConfig(
      'https://my-web.com/dashboard',
      specs,
      baseWindow,
      'https://my-web.com/dashboard',
    );

    expect(windows).toHaveLength(3);
    expect(windows[0].label).toBe('pake');
    expect(windows[1].url).toBe('https://my-web.com/camera');
    expect(windows[2].url).toBe('https://my-web.com/monitor');
  });
});
