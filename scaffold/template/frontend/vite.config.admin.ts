import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

const STRICT_ROLLUP_WARNING_ALLOWLIST = new Set<string>();

function failOnRollupWarning(warning: { code?: string; message: string }) {
  const code = warning.code ?? "UNKNOWN";
  if (STRICT_ROLLUP_WARNING_ALLOWLIST.has(code)) {
    return;
  }
  throw new Error(`[Rollup warning:${code}] ${warning.message}`);
}

function manualVendorChunk(id: string): string | undefined {
  if (!id.includes("/node_modules/")) return undefined;
  if (
    id.includes("/@tiptap/") ||
    id.includes("/prosemirror-") ||
    id.includes("/markdown-it/") ||
    id.includes("/highlight.js/")
  ) {
    return "vendor-editor";
  }
  if (id.includes("/react/") || id.includes("/react-dom/") || id.includes("/scheduler/")) {
    return "vendor-react";
  }
  if (id.includes("/react-datepicker/") || id.includes("/date-fns/")) {
    return "vendor-datetime";
  }
  if (id.includes("/i18next/") || id.includes("/react-i18next/")) {
    return "vendor-i18n";
  }
  if (id.includes("/zustand/")) {
    return "vendor-state";
  }
  if (id.includes("/axios/")) {
    return "vendor-http";
  }
  if (id.includes("/lucide-react/")) {
    return "vendor-icons";
  }
  return undefined;
}

export default defineConfig({
  plugins: [react()],
  root: ".",
  envDir: path.resolve(__dirname, ".."),
  envPrefix: "VITE_",
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
      output: {
        manualChunks(id) {
          return manualVendorChunk(id);
        },
      },
      onwarn(warning, _warn) {
        failOnRollupWarning(warning);
      },
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
