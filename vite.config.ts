import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri 需要固定端口，避免与 dev server 冲突
const port = 1420;

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port,
    strictPort: true,
    host: "0.0.0.0",
    hmr: {
      protocol: "ws",
      host: "localhost",
      port: 1421,
    },
  },
  build: {
    target: "es2021",
    rollupOptions: {
      input: {
        main: "index.html",
        notes: "notes.html",
      },
    },
  },
});
