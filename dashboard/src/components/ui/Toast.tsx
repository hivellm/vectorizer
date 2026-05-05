/**
 * Toast / Notification — console design.
 *
 * Uses the console palette (`--bg-1`, `--border`, `--green|red|amber|teal`)
 * via inline style, replacing the previous Tailwind colour classes.
 * The public API (`Toast` type, `ToastContainer`, `ToastType`) is
 * preserved for downstream consumers (`useToast`, `ToastProvider`).
 */

import { useEffect, useState, type CSSProperties } from 'react';

const XMarkIcon = ({ size = 14 }: { size?: number }) => (
  <svg
    width={size}
    height={size}
    fill="none"
    stroke="currentColor"
    viewBox="0 0 24 24"
    aria-hidden
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      d="M6 18L18 6M6 6l12 12"
    />
  </svg>
);

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface Toast {
  id: string;
  message: string;
  type: ToastType;
  duration?: number;
}

interface ToastProps {
  toast: Toast;
  onClose: (id: string) => void;
}

const TYPE_TO_TONE: Record<
  ToastType,
  { accent: string; tint: string }
> = {
  success: { accent: 'var(--green)', tint: 'rgba(76, 195, 138, 0.12)' },
  error: { accent: 'var(--red)', tint: 'rgba(229, 72, 77, 0.12)' },
  warning: { accent: 'var(--amber)', tint: 'var(--amber-dim)' },
  info: { accent: 'var(--teal)', tint: 'var(--teal-dim)' },
};

function ToastIcon({ type, size = 18 }: { type: ToastType; size?: number }) {
  const common = {
    width: size,
    height: size,
    fill: 'none',
    stroke: 'currentColor',
    viewBox: '0 0 24 24',
    'aria-hidden': true,
  } as const;
  const stroke = { strokeLinecap: 'round', strokeLinejoin: 'round', strokeWidth: 2 } as const;
  if (type === 'success') {
    return (
      <svg {...common}>
        <path {...stroke} d="M5 13l4 4L19 7" />
      </svg>
    );
  }
  if (type === 'error') {
    return (
      <svg {...common}>
        <path {...stroke} d="M6 18L18 6M6 6l12 12" />
      </svg>
    );
  }
  if (type === 'warning') {
    return (
      <svg {...common}>
        <path
          {...stroke}
          d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
        />
      </svg>
    );
  }
  return (
    <svg {...common}>
      <path {...stroke} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
    </svg>
  );
}

function ToastItem({ toast, onClose }: ToastProps) {
  const [isVisible, setIsVisible] = useState(false);

  useEffect(() => {
    setIsVisible(true);
    const timer = setTimeout(() => {
      setIsVisible(false);
      setTimeout(() => onClose(toast.id), 300);
    }, toast.duration || 5000);

    return () => clearTimeout(timer);
  }, [toast.id, toast.duration, onClose]);

  const tone = TYPE_TO_TONE[toast.type];

  const itemStyle: CSSProperties = {
    display: 'flex',
    alignItems: 'flex-start',
    gap: 12,
    padding: 14,
    borderRadius: 8,
    background: 'var(--bg-1)',
    border: '1px solid var(--border)',
    borderLeft: `3px solid ${tone.accent}`,
    boxShadow: '0 8px 24px rgba(0, 0, 0, 0.32)',
    transition: 'opacity 200ms ease, transform 200ms ease',
    opacity: isVisible ? 1 : 0,
    transform: isVisible ? 'translateY(0)' : 'translateY(-6px)',
    color: 'var(--text)',
    minWidth: 0,
  };

  return (
    <div style={itemStyle} role="status">
      <div style={{ flexShrink: 0, marginTop: 2, color: tone.accent }}>
        <ToastIcon type={toast.type} />
      </div>
      <div style={{ flex: 1, fontSize: 13, lineHeight: 1.45, fontWeight: 500, minWidth: 0 }}>
        {toast.message}
      </div>
      <button
        type="button"
        onClick={() => {
          setIsVisible(false);
          setTimeout(() => onClose(toast.id), 300);
        }}
        className="icon-btn"
        aria-label="Dismiss notification"
        style={{
          flexShrink: 0,
          background: 'transparent',
          color: 'var(--text-2)',
        }}
      >
        <XMarkIcon />
      </button>
    </div>
  );
}

interface ToastContainerProps {
  toasts: Toast[];
  onClose: (id: string) => void;
}

export function ToastContainer({ toasts, onClose }: ToastContainerProps) {
  if (toasts.length === 0) return null;

  return (
    <div
      style={{
        position: 'fixed',
        top: 16,
        right: 16,
        zIndex: 100,
        display: 'flex',
        flexDirection: 'column',
        gap: 8,
        maxWidth: 420,
        width: 'calc(100vw - 32px)',
      }}
    >
      {toasts.map((toast) => (
        <ToastItem key={toast.id} toast={toast} onClose={onClose} />
      ))}
    </div>
  );
}
