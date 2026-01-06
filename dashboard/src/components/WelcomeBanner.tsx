/**
 * Welcome Banner Component
 * Shows a welcome message for first-time users who need to complete setup
 */

import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useSetupStatus } from '@/hooks/useSetupRedirect';
import { Settings02, XClose, Rocket01 } from '@untitledui/icons';

interface WelcomeBannerProps {
  /** Whether the banner can be dismissed */
  dismissible?: boolean;
  /** Custom class name */
  className?: string;
}

function WelcomeBanner({ dismissible = true, className = '' }: WelcomeBannerProps) {
  const navigate = useNavigate();
  const { needsSetup, loading, status } = useSetupStatus();
  const [dismissed, setDismissed] = useState(false);

  // Don't show if loading, setup not needed, or dismissed
  if (loading || !needsSetup || dismissed) {
    return null;
  }

  return (
    <div className={`relative bg-gradient-to-r from-primary-600 to-indigo-600 rounded-xl p-6 text-white shadow-lg ${className}`}>
      {/* Dismiss button */}
      {dismissible && (
        <button
          onClick={() => setDismissed(true)}
          className="absolute top-3 right-3 p-1 rounded-full hover:bg-white/20 transition-colors"
          aria-label="Dismiss banner"
        >
          <XClose className="w-5 h-5" />
        </button>
      )}

      <div className="flex flex-col md:flex-row items-start md:items-center gap-4">
        {/* Icon */}
        <div className="flex-shrink-0">
          <div className="w-14 h-14 bg-white/20 rounded-xl flex items-center justify-center">
            <Rocket01 className="w-8 h-8" />
          </div>
        </div>

        {/* Content */}
        <div className="flex-1">
          <h3 className="text-xl font-semibold mb-1">
            Welcome to Vectorizer! 🎉
          </h3>
          <p className="text-white/90 text-sm mb-3">
            Get started by configuring your workspace. The Setup Wizard will help you
            detect your projects and create optimized collections automatically.
          </p>

          {/* Stats */}
          {status && (
            <div className="flex flex-wrap gap-4 text-sm text-white/80 mb-4">
              <span>Version: <strong className="text-white">{status.version}</strong></span>
              <span>•</span>
              <span>Collections: <strong className="text-white">{status.collection_count}</strong></span>
              <span>•</span>
              <span>Deployment: <strong className="text-white capitalize">{status.deployment_type}</strong></span>
            </div>
          )}
        </div>

        {/* CTA Button */}
        <div className="flex-shrink-0">
          <button
            onClick={() => navigate('/setup')}
            className="flex items-center gap-2 px-5 py-2.5 bg-white text-primary-700 font-semibold rounded-lg hover:bg-white/90 transition-colors shadow-md"
          >
            <Settings02 className="w-5 h-5" />
            Open Setup Wizard
          </button>
        </div>
      </div>

      {/* Quick tips */}
      <div className="mt-4 pt-4 border-t border-white/20">
        <p className="text-sm text-white/80">
          <strong>Quick tip:</strong> You can also run <code className="bg-white/20 px-1.5 py-0.5 rounded text-xs">vectorizer-cli setup /path/to/project</code> from the terminal.
        </p>
      </div>
    </div>
  );
}

export default WelcomeBanner;
