import { contextBridge, ipcRenderer } from 'electron';

export interface ElectronAPI {
  selectDirectory: () => Promise<string | null>;
  selectFiles: () => Promise<string[]>;
  getStoreValue: (key: string) => Promise<unknown>;
  setStoreValue: (key: string, value: unknown) => Promise<boolean>;
  deleteStoreValue: (key: string) => Promise<boolean>;
  clearStore: () => Promise<boolean>;
  getStorePath: () => Promise<string>;
  writeFile: (filePath: string, content: string) => Promise<boolean>;
  windowMinimize: () => void;
  windowMaximize: () => void;
  windowClose: () => void;
  vectorizer: {
    start: () => Promise<{ success: boolean; message: string }>;
    stop: () => Promise<{ success: boolean; message: string }>;
    restart: () => Promise<{ success: boolean; message: string }>;
    getStatus: () => Promise<{
      online: boolean;
      version?: string;
      uptime?: number;
      error?: string;
    }>;
    getLogs: () => Promise<ReadonlyArray<{
      timestamp: string;
      level: string;
      message: string;
    }>>;
  };
}

const electronAPI: ElectronAPI = {
  selectDirectory: () => ipcRenderer.invoke('select-directory'),
  selectFiles: () => ipcRenderer.invoke('select-files'),
  getStoreValue: (key: string) => ipcRenderer.invoke('get-store-value', key),
  setStoreValue: (key: string, value: unknown) => ipcRenderer.invoke('set-store-value', key, value),
  deleteStoreValue: (key: string) => ipcRenderer.invoke('delete-store-value', key),
  clearStore: () => ipcRenderer.invoke('clear-store'),
  getStorePath: () => ipcRenderer.invoke('get-store-path'),
  writeFile: (filePath: string, content: string) => ipcRenderer.invoke('write-file', filePath, content),
  windowMinimize: () => ipcRenderer.send('window-minimize'),
  windowMaximize: () => ipcRenderer.send('window-maximize'),
  windowClose: () => ipcRenderer.send('window-close'),
  vectorizer: {
    start: () => ipcRenderer.invoke('vectorizer:start'),
    stop: () => ipcRenderer.invoke('vectorizer:stop'),
    restart: () => ipcRenderer.invoke('vectorizer:restart'),
    getStatus: () => ipcRenderer.invoke('vectorizer:status'),
    getLogs: () => ipcRenderer.invoke('vectorizer:logs')
  }
};

contextBridge.exposeInMainWorld('electron', electronAPI);

