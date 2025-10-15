import { app, BrowserWindow, ipcMain, dialog } from 'electron';
import { join } from 'path';
import Store from 'electron-store';
import { VectorizerManager } from './vectorizer-manager';

const store = new Store();
const vectorizerManager = new VectorizerManager();

let mainWindow: BrowserWindow | null = null;

function createWindow(): void {
  mainWindow = new BrowserWindow({
    width: 1400,
    height: 900,
    minWidth: 1200,
    minHeight: 700,
    icon: join(__dirname, '../../assets/icons/icon.png'),
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: join(__dirname, 'preload.js')
    }
  });

  // Load the app
  if (process.env.NODE_ENV === 'development') {
    mainWindow.loadURL('http://localhost:5173');
    mainWindow.webContents.openDevTools();
  } else {
    mainWindow.loadFile(join(__dirname, '../../dist/index.html'));
  }

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
  store.set(key, value);
  return true;
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

// Cleanup on quit
app.on('before-quit', async () => {
  // Optionally stop vectorizer when GUI closes
  // await vectorizerManager.stop()
});

