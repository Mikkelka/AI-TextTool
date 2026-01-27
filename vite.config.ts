import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { resolve } from "path";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [vue()],

  // Build configuration for multiple entry points
  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'windows/index.html'),
        popup: resolve(__dirname, 'windows/popup.html'),
        chat: resolve(__dirname, 'windows/chat.html'),
        settings: resolve(__dirname, 'windows/settings.html'),
        history: resolve(__dirname, 'windows/history.html'),
        onboarding: resolve(__dirname, 'windows/onboarding.html'),
        'operation-edit': resolve(__dirname, 'windows/operation-edit.html')
      }
    }
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
