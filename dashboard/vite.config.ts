import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';
import { resolve } from 'path';

export default defineConfig(({ mode }) => {
  const isProduction = mode === 'production';
  
  return {
    plugins: [
      react(),
      tailwindcss(),
    ],
    resolve: {
      alias: {
        '@': resolve(__dirname, './src'),
        // Ensure single React instance
        'react': resolve(__dirname, './node_modules/react'),
        'react-dom': resolve(__dirname, './node_modules/react-dom'),
        // Resolve vis-data peer dependency (vis-network expects .js but vis-data provides .mjs)
        'vis-data/peer/esm/vis-data.js': resolve(__dirname, './node_modules/vis-data/peer/esm/vis-data.mjs'),
      },
      dedupe: ['react', 'react-dom'],
    },
    build: {
      outDir: 'dist',
      emptyOutDir: true,
      // Production optimizations
      minify: 'esbuild', // Faster than terser
      sourcemap: false, // Disable sourcemaps in production for smaller bundle
      // Code splitting configuration
      rollupOptions: {
        output: {
          // Manual chunk splitting for better caching
          manualChunks: (id) => {
            // Vendor chunks
            if (id.includes('node_modules')) {
              // React and React DOM in separate chunk
              if (id.includes('react') || id.includes('react-dom')) {
                return 'react-vendor';
              }
              // Monaco Editor is large - separate chunk
              if (id.includes('monaco-editor')) {
                return 'monaco-editor';
              }
              // Visx is large - separate chunk
              if (id.includes('@visx')) {
                return 'visx-vendor';
              }
              // Other node_modules
              return 'vendor';
            }
          },
          // Better chunk file names
          chunkFileNames: 'assets/js/[name]-[hash].js',
          entryFileNames: 'assets/js/[name]-[hash].js',
          assetFileNames: 'assets/[ext]/[name]-[hash].[ext]',
        },
      },
      // Chunk size warnings threshold (500KB)
      chunkSizeWarningLimit: 500,
      // Target modern browsers for smaller bundles
      target: 'esnext',
    },
    // Base path: '/dashboard/' for production (served by Rust server)
    // In dev mode, Vite dev server uses '/'
    base: isProduction ? '/dashboard/' : '/',
    optimizeDeps: {
      include: ['react', 'react-dom', 'vis-network', 'vis-data'],
      // Exclude large dependencies from pre-bundling
      exclude: ['@visx/visx'],
    },
  };
});

