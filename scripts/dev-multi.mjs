import { spawn } from 'node:child_process';
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.join(__dirname, '..');
const pakeJsonPath = path.join(rootDir, 'src-tauri/pake.json');
const defaultCapabilityPath = path.join(
  rootDir,
  'src-tauri/capabilities/default.json',
);
const generatedCapabilityPath = path.join(
  rootDir,
  'src-tauri/capabilities/generated.json',
);

const MAIN_WINDOW_LABEL = 'pake';

function parseWindowSpec(spec) {
  const eqIndex = spec.indexOf('=');
  if (eqIndex <= 0) {
    throw new Error(`Invalid window spec "${spec}". Expected label=/path`);
  }
  return {
    label: spec.slice(0, eqIndex).trim(),
    path: spec.slice(eqIndex + 1).trim(),
  };
}

function resolveWindowUrl(baseUrl, routePath) {
  if (
    routePath.startsWith('/') ||
    routePath.startsWith('./') ||
    routePath.startsWith('../')
  ) {
    return new URL(routePath, baseUrl).href;
  }
  try {
    return new URL(routePath).href;
  } catch {
    return new URL(routePath, baseUrl).href;
  }
}

async function applyMultiWindowConfig() {
  const raw = process.env.PAKE_WINDOWS?.trim();
  if (!raw) {
    console.error(
      'PAKE_WINDOWS is required, e.g. PAKE_WINDOWS="camera=/camera,monitor=/monitor"',
    );
    process.exit(1);
  }

  const specs = raw
    .split(',')
    .map((item) => item.trim())
    .filter(Boolean)
    .map(parseWindowSpec);

  const pakeConfig = JSON.parse(await fs.readFile(pakeJsonPath, 'utf8'));
  const mainWindow = { ...pakeConfig.windows[0] };
  const baseUrl = mainWindow.url;
  if (!baseUrl || mainWindow.url_type !== 'web') {
    console.error(
      'dev:multi expects src-tauri/pake.json main window url_type "web" with a base URL.',
    );
    process.exit(1);
  }

  mainWindow.label = MAIN_WINDOW_LABEL;
  const extraWindows = specs.map((spec) => ({
    ...mainWindow,
    label: spec.label,
    url: resolveWindowUrl(baseUrl, spec.path),
    url_type: 'web',
  }));

  pakeConfig.windows = [mainWindow, ...extraWindows];
  pakeConfig.multi_window = true;
  await fs.writeFile(pakeJsonPath, `${JSON.stringify(pakeConfig, null, 2)}\n`);

  const defaultCapability = JSON.parse(
    await fs.readFile(defaultCapabilityPath, 'utf8'),
  );
  const labels = [MAIN_WINDOW_LABEL, ...specs.map((spec) => spec.label)];
  const generated = {
    $schema: defaultCapability.$schema,
    identifier: 'generated',
    description: 'Generated capability for multi-window Pake dev builds.',
    webviews: labels,
    remote: defaultCapability.remote,
    permissions: [...defaultCapability.permissions],
  };
  await fs.writeFile(
    generatedCapabilityPath,
    `${JSON.stringify(generated, null, 2)}\n`,
  );

  console.log(`Applied ${specs.length} route window(s) to src-tauri/pake.json`);
}

await applyMultiWindowConfig();

const child = spawn('pnpm', ['run', 'dev'], {
  cwd: rootDir,
  stdio: 'inherit',
  shell: true,
});

child.on('exit', (code) => {
  process.exit(code ?? 0);
});
