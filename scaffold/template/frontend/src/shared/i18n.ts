import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import { getRuntimeConfig } from "@shared/runtimeConfig";
import en from "../../../i18n/en.json";
import zh from "../../../i18n/zh.json";

const BOOTSTRAP_SCRIPT_ID = "rustforge-bootstrap-runtime";
let initPromise: Promise<typeof i18n> | null = null;

function loadBootstrapScript(): Promise<void> {
  if (typeof window === "undefined" || typeof document === "undefined") {
    return Promise.resolve();
  }
  if (window.__RUSTFORGE_BOOTSTRAP__) return Promise.resolve();

  const existing = document.getElementById(BOOTSTRAP_SCRIPT_ID) as HTMLScriptElement | null;
  if (existing) {
    if (existing.dataset.loaded === "1") return Promise.resolve();
    return new Promise((resolve) => {
      existing.addEventListener("load", () => resolve(), { once: true });
      existing.addEventListener("error", () => resolve(), { once: true });
    });
  }

  return new Promise((resolve) => {
    const script = document.createElement("script");
    script.id = BOOTSTRAP_SCRIPT_ID;
    script.src = "/api/bootstrap.js";
    script.async = true;
    script.addEventListener(
      "load",
      () => {
        script.dataset.loaded = "1";
        resolve();
      },
      { once: true },
    );
    script.addEventListener("error", () => resolve(), { once: true });
    document.head.appendChild(script);
  });
}

/**
 * Transform Rust-style `:param` placeholders to i18next `{{param}}` syntax.
 * This lets both Rust and React share the same i18n JSON files.
 */
function transformParams(
  obj: Record<string, string>,
): Record<string, string> {
  const result: Record<string, string> = {};
  for (const [key, value] of Object.entries(obj)) {
    result[key] = value.replace(/:([a-zA-Z_]+)/g, "{{$1}}");
  }
  return result;
}

/**
 * i18next postProcessor that handles Rust-style `:param` placeholders
 * in fallback values (when a key is missing from the JSON and the key
 * itself is returned as the translation).
 */
const rustParamPostProcessor = {
  type: "postProcessor" as const,
  name: "rustParams",
  process(value: string, _key: string | string[], options: Record<string, unknown>) {
    return value.replace(/:([a-zA-Z_]+)/g, (match, param) => {
      return options[param] !== undefined ? String(options[param]) : match;
    });
  },
};

export async function initI18n(): Promise<typeof i18n> {
  if (i18n.isInitialized) return i18n;
  if (initPromise) return initPromise;

  initPromise = (async () => {
    await loadBootstrapScript();

    const runtimeConfig = getRuntimeConfig();
    const resources = {
      en: { translation: transformParams(en) },
      zh: { translation: transformParams(zh) },
    };

    await i18n
      .use(rustParamPostProcessor)
      .use(initReactI18next)
      .init({
        lng: runtimeConfig.i18n.defaultLocale,
        fallbackLng: runtimeConfig.i18n.defaultLocale,
        supportedLngs: runtimeConfig.i18n.supportedLocales,
        keySeparator: false,
        nsSeparator: false,
        interpolation: { escapeValue: false },
        postProcess: ["rustParams"],
        resources,
      });

    return i18n;
  })();

  try {
    return await initPromise;
  } catch (error) {
    initPromise = null;
    throw error;
  }
}

export default i18n;
