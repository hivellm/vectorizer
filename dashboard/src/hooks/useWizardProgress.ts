/**
 * useWizardProgress — persists interrupted Setup-Wizard state to
 * localStorage so reloading the page (or crashing the tab) doesn't
 * force the user to restart from the welcome step.
 *
 * The stored snapshot is intentionally small: the wizard step, the
 * selected template id, the folder path, and the list of analyzed
 * projects with their current toggle/rename state. The API-key step
 * is NOT persisted (credentials must stay in-memory).
 *
 * Snapshots older than TTL_MS are discarded on read — the user has
 * likely moved on from that workspace by then.
 */

import { useCallback, useEffect, useState } from 'react';

const STORAGE_KEY = 'vectorizer.setup-wizard.progress.v1';
const TTL_MS = 7 * 24 * 60 * 60 * 1000;

export interface WizardSnapshot<TStep, TTemplate, TProject> {
  step: TStep;
  template: TTemplate | null;
  folderPath: string;
  projects: TProject[];
  savedAt: string;
}

function readSnapshot<TStep, TTemplate, TProject>(): WizardSnapshot<TStep, TTemplate, TProject> | null {
  if (typeof window === 'undefined') return null;
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as WizardSnapshot<TStep, TTemplate, TProject>;
    if (!parsed?.savedAt) return null;
    const ageMs = Date.now() - new Date(parsed.savedAt).getTime();
    if (Number.isNaN(ageMs) || ageMs > TTL_MS) {
      window.localStorage.removeItem(STORAGE_KEY);
      return null;
    }
    return parsed;
  } catch {
    return null;
  }
}

function writeSnapshot<TStep, TTemplate, TProject>(
  snapshot: WizardSnapshot<TStep, TTemplate, TProject>
): void {
  if (typeof window === 'undefined') return;
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(snapshot));
  } catch {
    // Best-effort — localStorage quota is not worth surfacing to the UI.
  }
}

function clearSnapshot(): void {
  if (typeof window === 'undefined') return;
  try {
    window.localStorage.removeItem(STORAGE_KEY);
  } catch {
    // Ignore.
  }
}

export interface WizardProgressApi<TStep, TTemplate, TProject> {
  /** Snapshot loaded once on mount; null if none exists or it expired. */
  snapshot: WizardSnapshot<TStep, TTemplate, TProject> | null;
  /** Serialise the current wizard state to localStorage. */
  save: (snapshot: Omit<WizardSnapshot<TStep, TTemplate, TProject>, 'savedAt'>) => void;
  /** Drop any saved progress — called after apply-config succeeds or on explicit reset. */
  clear: () => void;
}

export function useWizardProgress<TStep, TTemplate, TProject>(): WizardProgressApi<TStep, TTemplate, TProject> {
  const [snapshot] = useState<WizardSnapshot<TStep, TTemplate, TProject> | null>(() =>
    readSnapshot<TStep, TTemplate, TProject>()
  );

  const save = useCallback<WizardProgressApi<TStep, TTemplate, TProject>['save']>((data) => {
    writeSnapshot({ ...data, savedAt: new Date().toISOString() });
  }, []);

  const clear = useCallback<WizardProgressApi<TStep, TTemplate, TProject>['clear']>(() => {
    clearSnapshot();
  }, []);

  useEffect(() => {
    // Snapshot is a one-shot load on mount — no subscription needed.
  }, []);

  return { snapshot, save, clear };
}

export const __WIZARD_PROGRESS_INTERNALS = {
  STORAGE_KEY,
  TTL_MS,
  readSnapshot,
  writeSnapshot,
  clearSnapshot,
};
