import { app, BrowserWindow, ipcMain, dialog } from 'electron';
import { join } from 'path';
import Store from 'electron-store';
import { VectorizerManager } from './vectorizer-manager';
import { randomBytes } from 'crypto';

// Generate or load encryption key
function getEncryptionKey(): string {
  const tempStore = new Store({ name: 'vectorizer-secure' });
  let encryptionKey = tempStore.get('encryptionKey') as string | undefined;
  
  if (!encryptionKey) {
    // Generate a new encryption key on first run
    encryptionKey = randomBytes(32).toString('hex');
    tempStore.set('encryptionKey', encryptionKey);
  }
  
  return encryptionKey;
}

// Initialize encrypted store for sensitive data
const encryptionKey = getEncryptionKey();
const store = new Store({
  name: 'vectorizer-config',
  encryptionKey,
  // Store connections and sensitive data securely
  schema: {
    connections: {
      type: 'array',
      default: []
    },
    activeConnectionId: {
      type: ['string', 'null'],
      default: null
    },
    workspaces: {
      type: 'array',
      default: []
    },
    settings: {
      type: 'object',
      default: {}
    }
  }
});

const vectorizerManager = new VectorizerManager();

let mainWindow: BrowserWindow | null = null;

function createWindow(): void {
  mainWindow = new BrowserWindow({
    width: 1400,
    height: 900,
    minWidth: 1200,
    minHeight: 700,
    frame: false,
    titleBarStyle: 'hidden',
    icon: join(__dirname, '../../assets/icons/icon.png'),
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: join(__dirname, 'preload.js'),
      webSecurity: true,
      allowRunningInsecureContent: false
    }
  });

  // Load the app
  if (process.env.NODE_ENV === 'development') {
    mainWindow.loadURL('http://localhost:5173');
    mainWindow.webContents.openDevTools();
  } else {
    mainWindow.loadFile(join(__dirname, '../../dist/index.html'));
    // Open DevTools in production for debugging
    mainWindow.webContents.openDevTools();
  }
  
  // Log any console messages from renderer
  mainWindow.webContents.on('console-message', (event, level, message, line, sourceId) => {
    console.log(`[Renderer] ${message}`);
  });

  mainWindow.on('closed', () => {
    mainWindow = null;
  });
}

app.whenReady().then(() => {
  createWindow();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

// IPC Handlers
ipcMain.handle('select-directory', async (): Promise<string | null> => {
  if (!mainWindow) return null;
  
  const result = await dialog.showOpenDialog(mainWindow, {
    properties: ['openDirectory']
  });
  return result.filePaths[0] || null;
});

ipcMain.handle('select-files', async (): Promise<string[]> => {
  if (!mainWindow) return [];
  
  const result = await dialog.showOpenDialog(mainWindow, {
    properties: ['openFile', 'multiSelections']
  });
  return result.filePaths || [];
});

ipcMain.handle('get-store-value', (_event, key: string): unknown => {
  return store.get(key);
});

ipcMain.handle('set-store-value', (_event, key: string, value: unknown): boolean => {
  try {
    store.set(key, value);
    return true;
  } catch (error) {
    console.error('Failed to set store value:', error);
    return false;
  }
});

ipcMain.handle('delete-store-value', (_event, key: string): boolean => {
  try {
    store.delete(key);
    return true;
  } catch (error) {
    console.error('Failed to delete store value:', error);
    return false;
  }
});

ipcMain.handle('clear-store', (): boolean => {
  try {
    store.clear();
    return true;
  } catch (error) {
    console.error('Failed to clear store:', error);
    return false;
  }
});

ipcMain.handle('get-store-path', (): string => {
  return store.path;
});

ipcMain.handle('vectorizer:start', async () => {
  return await vectorizerManager.start();
});

ipcMain.handle('vectorizer:stop', async () => {
  return await vectorizerManager.stop();
});

ipcMain.handle('vectorizer:restart', async () => {
  return await vectorizerManager.restart();
});

ipcMain.handle('vectorizer:status', async () => {
  return await vectorizerManager.getStatus();
});

ipcMain.handle('vectorizer:logs', async () => {
  return vectorizerManager.getLogs();
});

// Window control handlers
ipcMain.on('window-minimize', () => {
  if (mainWindow) mainWindow.minimize();
});

ipcMain.on('window-maximize', () => {
  if (mainWindow) {
    if (mainWindow.isMaximized()) {
      mainWindow.unmaximize();
    } else {
      mainWindow.maximize();
    }
  }
});

ipcMain.on('window-close', () => {
  if (mainWindow) mainWindow.close();
});

// Cleanup on quit
app.on('before-quit', async () => {
  // Optionally stop vectorizer when GUI closes
  // await vectorizerManager.stop()
});

