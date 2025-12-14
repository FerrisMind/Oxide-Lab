import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [tailwindcss(), sveltekit()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1411,
    strictPort: false,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri` and documentation files
      // @ts-ignore - chokidar supports function in ignored
      ignored: (path) => {
        // Ignore src-tauri
        if (path.includes('src-tauri')) return true;
        // Ignore example directory
        if (path.includes('example')) return true;
        // Ignore documentation files in root (but allow markdown in src/ for content)
        const rootDocFiles = ['ROADMAP', 'README', 'LICENSE', 'THIRD_PARTY_LICENSES'];
        const isRootDocFile = rootDocFiles.some(
          (name) => path.includes(name) && !path.includes('src/'),
        );
        if (isRootDocFile) return true;
        // Standard ignores
        if (path.includes('.git') || path.includes('node_modules')) return true;
        return false;
      },
    },
  },
}));
