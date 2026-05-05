/**
 * Utility to merge class names.
 *
 * After dropping Tailwind, the previous `twMerge` step is no longer
 * needed. We delegate to `clsx`, which already handles conditional
 * classes, arrays, and object shorthand.
 */

import { clsx, type ClassValue } from 'clsx';

export function cn(...inputs: ClassValue[]) {
  return clsx(inputs);
}
