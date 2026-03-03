import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import { getRuntimeConfig } from "@shared/runtimeConfig";
import en from "../../../i18n/en.json";
import zh from "../../../i18n/zh.json";

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

const runtimeConfig = getRuntimeConfig();
const resources = {
  en: { translation: transformParams(en) },
  zh: { translation: transformParams(zh) },
};

i18n.use(initReactI18next).init({
  lng: runtimeConfig.i18n.defaultLocale,
  fallbackLng: runtimeConfig.i18n.defaultLocale,
  supportedLngs: runtimeConfig.i18n.supportedLocales,
  keySeparator: false,
  nsSeparator: false,
  interpolation: { escapeValue: false },
  resources,
});

export default i18n;
