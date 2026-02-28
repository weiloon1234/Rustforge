import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [react()],
  root: ".",
  base: "/",
  resolve: {
    alias: {
      "@shared": path.resolve(__dirname, "src/shared"),
      "@admin": path.resolve(__dirname, "src/admin"),
      "@user": path.resolve(__dirname, "src/user"),
    },
  },
  build: {
    outDir: "../public",
    emptyOutDir: false,
    rollupOptions: {
      input: "user.html",
    },
  },
  // Rename user.html → index.html in the output so the Rust SPA
  // fallback (which looks for public/index.html) works unchanged.
  experimental: {
    renderBuiltUrl(filename, { hostType }) {
      if (hostType === "html") return filename;
      return "/" + filename;
    },
  },
  server: {
    port: 5173,
    proxy: {
      "/api": "http://localhost:3000",
    },
  },
});
