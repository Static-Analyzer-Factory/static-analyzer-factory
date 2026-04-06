import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  base: process.env.GITHUB_PAGES ? '/static-analyzer-factory/' : './',
  build: { outDir: 'dist', target: 'esnext' },
});
