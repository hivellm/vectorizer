/**
 * Main App component
 */

import { BrowserRouter } from 'react-router-dom';
import AppRouter from './router/AppRouter';
import { ThemeProvider } from './providers/ThemeProvider';
import { AuthProvider } from './contexts/AuthContext';

function App() {
  // Use Vite's BASE_URL which matches the base config
  // In production: '/dashboard/' (with trailing slash from vite.config.ts)
  // In dev: '/' (no base path)
  const basename = import.meta.env.BASE_URL.replace(/\/$/, '') || '/';

  return (
    <ThemeProvider>
      <AuthProvider>
        <BrowserRouter basename={basename}>
          <AppRouter />
        </BrowserRouter>
      </AuthProvider>
    </ThemeProvider>
  );
}

export default App;
