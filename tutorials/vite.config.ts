import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  base: process.env.GITHUB_PAGES ? '/static-analyzer-factory/tutorials/' : (process.env.DEV_BASE ?? './'),
  build: { outDir: 'dist', target: 'esnext' },
  optimizeDeps: { exclude: ['web-tree-sitter'] },
});
