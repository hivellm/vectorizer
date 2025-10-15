/**
 * Check if the app is running in Electron
 */
export function isElectron(): boolean {
  return typeof window !== 'undefined' && window.electron !== undefined;
}

/**
 * Get Electron API or throw error if not available
 */
export function requireElectron() {
  if (!isElectron()) {
    throw new Error('This feature requires Electron. Please run the app in Electron mode.');
  }
  return window.electron!;
}

/**
 * Alert wrapper that works in both web and Electron
 */
export function showAlert(message: string): void {
  alert(message);
}

/**
 * Confirm dialog wrapper
 */
export function showConfirm(message: string): boolean {
  return confirm(message);
}

