import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [react()],
  root: ".",
  base: "/admin/",
  resolve: {
    alias: {
      "@shared": path.resolve(__dirname, "src/shared"),
      "@admin": path.resolve(__dirname, "src/admin"),
      "@user": path.resolve(__dirname, "src/user"),
    },
  },
  build: {
    outDir: "../public/admin",
    emptyOutDir: true,
    rollupOptions: {
      input: "admin.html",
    },
  },
  experimental: {
    renderBuiltUrl(filename, { hostType }) {
      if (hostType === "html") return filename;
      return "/admin/" + filename;
    },
  },
  server: {
    port: 5174,
    proxy: {
      "/api": "http://localhost:3000",
    },
  },
});
