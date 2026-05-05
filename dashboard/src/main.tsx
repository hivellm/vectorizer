/**
 * Main entry point for the dashboard application
 */

import { createRoot } from 'react-dom/client';
import App from './App';
import './styles/console.css';

const rootElement = document.getElementById('app');

if (!rootElement) {
  throw new Error('Root element not found');
}

const root = createRoot(rootElement);
root.render(<App />);

