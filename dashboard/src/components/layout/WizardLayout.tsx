/**
 * Wizard Layout - Full screen layout with glassmorphism effect
 * Creates a focused, immersive experience for the setup wizard
 */

import { useTheme } from '@/providers/ThemeProvider';
import { ToastProvider } from '@/providers/ToastProvider';
import { Moon01, Sun } from '@untitledui/icons';

interface WizardLayoutProps {
  children: React.ReactNode;
}

function WizardLayout({ children }: WizardLayoutProps) {
  const { theme, toggleTheme } = useTheme();

  return (
    <ToastProvider>
      <div className="min-h-screen relative overflow-hidden">
        {/* Animated Background */}
        <div className="fixed inset-0 bg-gradient-to-br from-neutral-900 via-neutral-950 to-black">
          {/* Animated gradient orbs */}
          <div className="absolute top-[-20%] left-[-10%] w-[600px] h-[600px] bg-gradient-to-br from-primary-600/30 to-indigo-600/20 rounded-full blur-[120px] animate-pulse-slow" />
          <div className="absolute bottom-[-20%] right-[-10%] w-[500px] h-[500px] bg-gradient-to-br from-indigo-600/25 to-purple-600/15 rounded-full blur-[100px] animate-pulse-slow" style={{ animationDelay: '1s' }} />
          <div className="absolute top-[40%] right-[20%] w-[300px] h-[300px] bg-gradient-to-br from-cyan-500/15 to-blue-500/10 rounded-full blur-[80px] animate-pulse-slow" style={{ animationDelay: '2s' }} />
          
          {/* Grid overlay for depth */}
          <div 
            className="absolute inset-0 opacity-[0.03]"
            style={{
              backgroundImage: `
                linear-gradient(rgba(255,255,255,0.1) 1px, transparent 1px),
                linear-gradient(90deg, rgba(255,255,255,0.1) 1px, transparent 1px)
              `,
              backgroundSize: '50px 50px'
            }}
          />
          
          {/* Radial gradient overlay */}
          <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,transparent_0%,rgba(0,0,0,0.5)_100%)]" />
        </div>

        {/* Top Navigation Bar - Glassmorphism */}
        <header className="fixed top-0 left-0 right-0 z-50">
          <div className="mx-4 mt-4 sm:mx-6 lg:mx-8 max-w-4xl lg:mx-auto">
            <div className="bg-white/10 dark:bg-white/5 backdrop-blur-xl border border-white/10 rounded-2xl shadow-2xl shadow-black/20">
              <div className="px-4 sm:px-6">
                <div className="flex items-center justify-between h-14">
                  {/* Logo */}
                  <div className="flex items-center gap-3">
                    <div className="w-9 h-9 bg-gradient-to-br from-primary-500 to-indigo-600 rounded-xl flex items-center justify-center shadow-lg shadow-primary-500/25">
                      <svg className="w-5 h-5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
                      </svg>
                    </div>
                    <div>
                      <h1 className="text-lg font-semibold text-white leading-none">
                        Vectorizer
                      </h1>
                      <p className="text-xs text-white/50 mt-0.5 leading-none">
                        Setup Wizard
                      </p>
                    </div>
                  </div>

                  {/* Theme Toggle */}
                  <button
                    onClick={toggleTheme}
                    className="p-2.5 rounded-xl text-white/60 hover:text-white hover:bg-white/10 transition-all duration-200"
                    aria-label="Toggle theme"
                  >
                    {theme === 'dark' ? (
                      <Moon01 className="w-5 h-5" />
                    ) : (
                      <Sun className="w-5 h-5" />
                    )}
                  </button>
                </div>
              </div>
            </div>
          </div>
        </header>

        {/* Main Content */}
        <main className="relative pt-24 pb-20 min-h-screen">
          <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
            {children}
          </div>
        </main>

        {/* Footer - Glassmorphism */}
        <footer className="fixed bottom-0 left-0 right-0 z-50">
          <div className="mx-4 mb-4 sm:mx-6 lg:mx-8 max-w-4xl lg:mx-auto">
            <div className="bg-white/5 backdrop-blur-xl border border-white/10 rounded-2xl">
              <div className="flex items-center justify-center h-12 text-xs text-white/40">
                <span>Vectorizer v{import.meta.env.VITE_APP_VERSION || '2.3.0'}</span>
                <span className="mx-2">•</span>
                <a 
                  href="https://github.com/hivellm/vectorizer" 
                  target="_blank" 
                  rel="noopener noreferrer"
                  className="hover:text-white/70 transition-colors"
                >
                  Documentation
                </a>
              </div>
            </div>
          </div>
        </footer>

        {/* CSS Animations */}
        <style>{`
          @keyframes pulse-slow {
            0%, 100% { opacity: 1; transform: scale(1); }
            50% { opacity: 0.7; transform: scale(1.05); }
          }
          .animate-pulse-slow {
            animation: pulse-slow 8s ease-in-out infinite;
          }
        `}</style>
      </div>
    </ToastProvider>
  );
}

export default WizardLayout;
