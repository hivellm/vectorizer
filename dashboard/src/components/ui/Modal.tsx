/**
 * Modal component - delegates to the console design Modal primitive.
 *
 * The legacy `<Modal isOpen=...>` API is preserved so existing
 * consumers (modals/*.tsx) keep working. The console primitive owns
 * the styling (`.cmd-overlay` + `.cmd-panel`).
 */

import { Modal as ConsoleModal } from '@/components/console';
import type { ReactNode } from 'react';

export interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title?: ReactNode;
  children: ReactNode;
  size?: 'sm' | 'md' | 'lg' | 'xl';
  footer?: ReactNode;
}

const SIZE_TO_PX: Record<NonNullable<ModalProps['size']>, number> = {
  sm: 380,
  md: 520,
  lg: 720,
  xl: 960,
};

function Modal({ isOpen, onClose, title, children, size = 'md', footer }: ModalProps) {
  return (
    <ConsoleModal
      open={isOpen}
      onClose={onClose}
      title={title}
      width={SIZE_TO_PX[size]}
      footer={footer}
    >
      {children}
    </ConsoleModal>
  );
}

export default Modal;
