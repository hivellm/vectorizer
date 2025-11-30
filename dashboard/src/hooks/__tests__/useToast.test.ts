/**
 * Unit tests for useToast hook
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useToast } from '../useToast';

describe('useToast', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('should initialize with empty toasts', () => {
    const { result } = renderHook(() => useToast());
    expect(result.current.toasts).toEqual([]);
  });

  it('should add success toast', () => {
    const { result } = renderHook(() => useToast());

    act(() => {
      result.current.success('Success message');
    });

    expect(result.current.toasts.length).toBe(1);
    expect(result.current.toasts[0].message).toBe('Success message');
    expect(result.current.toasts[0].type).toBe('success');
  });

  it('should add error toast', () => {
    const { result } = renderHook(() => useToast());

    act(() => {
      result.current.error('Error message');
    });

    expect(result.current.toasts.length).toBe(1);
    expect(result.current.toasts[0].message).toBe('Error message');
    expect(result.current.toasts[0].type).toBe('error');
  });

  it('should add warning toast', () => {
    const { result } = renderHook(() => useToast());

    act(() => {
      result.current.warning('Warning message');
    });

    expect(result.current.toasts.length).toBe(1);
    expect(result.current.toasts[0].type).toBe('warning');
  });

  it('should add info toast', () => {
    const { result } = renderHook(() => useToast());

    act(() => {
      result.current.info('Info message');
    });

    expect(result.current.toasts.length).toBe(1);
    expect(result.current.toasts[0].type).toBe('info');
  });

  it('should remove toast', () => {
    const { result } = renderHook(() => useToast());

    act(() => {
      result.current.success('Test message');
    });

    const toastId = result.current.toasts[0].id;

    act(() => {
      result.current.removeToast(toastId);
    });

    expect(result.current.toasts.length).toBe(0);
  });

  it('should return toast id when showing toast', () => {
    const { result } = renderHook(() => useToast());

    let toastId: string | undefined;
    act(() => {
      toastId = result.current.success('Test message');
    });

    expect(toastId).toBeDefined();
    expect(typeof toastId).toBe('string');
    expect(result.current.toasts[0].id).toBe(toastId);
  });
});

