/**
 * Unit tests for useWizardProgress.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useWizardProgress, __WIZARD_PROGRESS_INTERNALS } from '../useWizardProgress';

describe('useWizardProgress', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('returns null snapshot on a fresh install', () => {
    const { result } = renderHook(() => useWizardProgress<'folder', null, never>());
    expect(result.current.snapshot).toBeNull();
  });

  it('save persists a snapshot that a subsequent mount reads', () => {
    const first = renderHook(() =>
      useWizardProgress<'analysis', { id: string }, { name: string }>()
    );

    act(() => {
      first.result.current.save({
        step: 'analysis',
        template: { id: 'rag' },
        folderPath: '/workspace/demo',
        projects: [{ name: 'demo' }],
      });
    });

    const second = renderHook(() =>
      useWizardProgress<'analysis', { id: string }, { name: string }>()
    );

    expect(second.result.current.snapshot).not.toBeNull();
    expect(second.result.current.snapshot?.step).toBe('analysis');
    expect(second.result.current.snapshot?.template).toEqual({ id: 'rag' });
    expect(second.result.current.snapshot?.folderPath).toBe('/workspace/demo');
    expect(second.result.current.snapshot?.projects).toEqual([{ name: 'demo' }]);
    expect(new Date(second.result.current.snapshot!.savedAt).toString()).not.toBe('Invalid Date');
  });

  it('clear wipes the saved snapshot', () => {
    const first = renderHook(() => useWizardProgress<'folder', null, never>());
    act(() => {
      first.result.current.save({
        step: 'folder',
        template: null,
        folderPath: '/tmp',
        projects: [],
      });
    });

    expect(localStorage.getItem(__WIZARD_PROGRESS_INTERNALS.STORAGE_KEY)).not.toBeNull();

    act(() => first.result.current.clear());

    expect(localStorage.getItem(__WIZARD_PROGRESS_INTERNALS.STORAGE_KEY)).toBeNull();
  });

  it('discards snapshots older than the TTL', () => {
    const oldTimestamp = new Date(
      Date.now() - __WIZARD_PROGRESS_INTERNALS.TTL_MS - 60_000
    ).toISOString();

    localStorage.setItem(
      __WIZARD_PROGRESS_INTERNALS.STORAGE_KEY,
      JSON.stringify({
        step: 'review',
        template: null,
        folderPath: '/old',
        projects: [],
        savedAt: oldTimestamp,
      })
    );

    const { result } = renderHook(() => useWizardProgress<'review', null, never>());
    expect(result.current.snapshot).toBeNull();
    // The stale payload was pruned by readSnapshot.
    expect(localStorage.getItem(__WIZARD_PROGRESS_INTERNALS.STORAGE_KEY)).toBeNull();
  });

  it('ignores corrupt JSON without throwing', () => {
    localStorage.setItem(__WIZARD_PROGRESS_INTERNALS.STORAGE_KEY, 'not-json');
    const { result } = renderHook(() => useWizardProgress<'welcome', null, never>());
    expect(result.current.snapshot).toBeNull();
  });
});
