/**
 * Loading state component with message - Dark mode support
 */

import LoadingSpinner from './LoadingSpinner';

interface LoadingStateProps {
  message?: string;
  size?: 'sm' | 'md' | 'lg';
}

function LoadingState({ message = 'Loading...', size = 'md' }: LoadingStateProps) {
  return (
    <div className="flex flex-col items-center justify-center p-8">
      <LoadingSpinner size={size} />
      {message && (
        <p className="mt-4 text-sm text-neutral-600 dark:text-neutral-400">{message}</p>
      )}
    </div>
  );
}

export default LoadingState;
