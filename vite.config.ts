import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "node:path";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: { "@": path.resolve(__dirname, "src") },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    proxy: {
      // Forward /api to the Axum server during development
      "/api": { target: "http://localhost:3000", changeOrigin: true },
    },
  },
  build: {
    target: "es2022",
    outDir: "dist",
    sourcemap: false,
  },
});
