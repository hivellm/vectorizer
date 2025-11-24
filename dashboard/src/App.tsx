/**
 * Main App component
 */

import { BrowserRouter } from 'react-router';
import ErrorBoundary from './components/ErrorBoundary';
import { ThemeProvider } from './providers/ThemeProvider';
import AppRouter from './router/AppRouter';

function App() {
  return (
    <ErrorBoundary>
      <ThemeProvider defaultTheme="dark">
        <BrowserRouter>
          <AppRouter />
        </BrowserRouter>
      </ThemeProvider>
    </ErrorBoundary>
  );
}

export default App;
