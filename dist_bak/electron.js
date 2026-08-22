const { app, BrowserWindow, globalShortcut, ipcMain } = require('electron');
const path = require('path');
const log = require('electron-log');

log.transports.file.resolvePath = () =>
  path.join(app.getPath('userData'), 'logs', 'main.log');
log.transports.file.maxSize = 5 * 1024 * 1024;
log.transports.file.archiveLog = true;
log.transports.file.format = '{y}-{m}-{d} {h}:{i}:{s}.{ms} [{level}] {text}';
log.transports.console.level = false;

log.info('--- App starting ---');
log.info('Electron version:', process.versions.electron);
log.info('Chrome version:', process.versions.chrome);

let mainWindow;
const isDev = !app.isPackaged;

app.commandLine.appendSwitch('ignore-gpu-blocklist');
app.commandLine.appendSwitch('enable-gpu-rasterization');
app.commandLine.appendSwitch('enable-zero-copy');

app.commandLine.appendSwitch('enable-accelerated-video-decode');
app.commandLine.appendSwitch('enable-accelerated-video-encode');
app.commandLine.appendSwitch('enable-accelerated-mjpeg-decode');

app.commandLine.appendSwitch('use-angle', 'd3d11');
app.commandLine.appendSwitch('use-gl', 'angle');

app.commandLine.appendSwitch('enable-native-gpu-memory-buffers');
app.commandLine.appendSwitch('num-raster-threads', '4');
app.commandLine.appendSwitch('enable-gpu-memory-buffer-video-frames');

app.commandLine.appendSwitch(
  'enable-features',
  'VaapiVideoDecoder,' +
    'CanvasOopRasterization,' +
    'AcceleratedVideoDecode,' +
    'UseSkiaRenderer,' +
    'VideoPlaybackQuality,' +
    'DefaultANGLEVulkan',
);

app.commandLine.appendSwitch('disable-software-rasterizer');

app.commandLine.appendSwitch('enable-webgl');
app.commandLine.appendSwitch('enable-webgl2-compute-context');

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1280,
    height: 800,
    webPreferences: {
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: false,
      enablePreferredSizeMode: false,
      spellcheck: false,
      backgroundThrottling: false,
    },
  });

  globalShortcut.register('CommandOrControl+Shift+G', () => {
    const gpuWin = new BrowserWindow({ width: 1000, height: 700 });
    gpuWin.loadURL('chrome://gpu');
  });

  if (isDev) {
    mainWindow.loadURL(
      process.env.ELECTRON_START_URL || 'http://localhost:4200',
    );
  } else {
    mainWindow.loadFile(path.join(__dirname, '../build/index.html'));
    process.stdout.write = () => {};
    process.stderr.write = () => {};
    console.log = () => {};
    console.error = () => {};
  }

  mainWindow.on('closed', () => {
    mainWindow = null;
  });

  mainWindow.webContents.on('did-finish-load', async () => {
    log.info('Main window loaded');
    try {
      const gpuInfo = await app.getGPUInfo('complete');
      log.info(
        'GPU Info:',
        JSON.stringify({
          gpuDevice: gpuInfo.gpuDevice?.[0]?.vendorId,
          auxAttributes: gpuInfo.auxAttributes,
          featureStatus: gpuInfo.featureStatus,
        }),
      );

      if (gpuInfo.auxAttributes?.softwareRendering) {
        log.warn('WARNING: Software rendering is enabled!');
      } else {
        log.info('Hardware acceleration is active');
      }
    } catch (err) {
      log.error('Failed to get GPU info:', err);
    }
  });

  mainWindow.webContents.on('render-process-gone', (event, details) => {
    log.error('Renderer process gone:', details);
  });

  mainWindow.webContents.on('before-input-event', (event, input) => {
    if (!isDev) {
      const isReload =
        (input.key === 'r' && (input.control || input.meta)) ||
        (input.key === 'r' && (input.control || input.meta) && input.shift) ||
        input.key === 'F5';
      if (isReload) {
        event.preventDefault();
        mainWindow.loadFile(path.join(__dirname, '../build/index.html'));
      }
    }
  });
}

setInterval(() => {
  const mem = process.getSystemMemoryInfo();
  const cpu = process.getCPUUsage();
  log.info(
    `CPU User: ${cpu.user.toFixed(2)}%, System: ${cpu.system.toFixed(2)}%, Free Memory: ${Math.round(mem.free / 1024)} MB`,
  );
}, 30000);

app.on('render-process-gone', (event, webContents, details) => {
  log.error('Renderer process gone:', details);
});

app.on('child-process-gone', (event, details) => {
  log.error('Child process gone:', details);
});

app.on('gpu-process-crashed', (event, killed) => {
  log.error('GPU process crashed. Killed:', killed);
  if (mainWindow && !mainWindow.isDestroyed()) {
    mainWindow.webContents.reload();
  }
});

process.on('uncaughtException', (error) => {
  log.error('Uncaught Exception:', error);
});

process.on('unhandledRejection', (reason, promise) => {
  log.error('Unhandled Rejection at:', promise, 'reason:', reason);
});

ipcMain.on('log:info', (_, msg) => log.info('[Renderer]', msg));
ipcMain.on('log:error', (_, msg) => log.error('[Renderer]', msg));

app
  .whenReady()
  .then(() => {
    createWindow();
    log.info('App ready. Hardware acceleration enabled.');
  })
  .catch((err) => {
    log.error('App init error:', err);
  });

app.on('window-all-closed', () => {
  globalShortcut.unregisterAll();
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('activate', () => {
  if (mainWindow === null) {
    createWindow();
  }
});

app.on('will-quit', () => {
  globalShortcut.unregisterAll();
  log.info('App quitting...');
});
