import { DEFAULT_LOCALE, type LocaleCode } from "@shared/types/platform";

interface RuntimeBootstrapWire {
  i18n?: {
    default_locale?: unknown;
    supported_locales?: unknown;
    default_timezone?: unknown;
  };
}

export interface RuntimeI18nConfig {
  defaultLocale: LocaleCode;
  supportedLocales: LocaleCode[];
  defaultTimezone: string;
}

export interface RuntimeConfig {
  i18n: RuntimeI18nConfig;
}

const FALLBACK_RUNTIME_CONFIG: RuntimeConfig = {
  i18n: {
    defaultLocale: DEFAULT_LOCALE,
    supportedLocales: [DEFAULT_LOCALE],
    defaultTimezone: "+00:00",
  },
};

declare global {
  interface Window {
    __RUSTFORGE_BOOTSTRAP__?: RuntimeBootstrapWire;
  }
}

let cachedRuntimeConfig: RuntimeConfig | null = null;

function asLocaleCode(value: unknown): LocaleCode | null {
  if (typeof value !== "string") return null;
  const trimmed = value.trim();
  if (!trimmed) return null;
  return trimmed as LocaleCode;
}

function asLocaleCodes(value: unknown): LocaleCode[] {
  if (!Array.isArray(value)) return [];
  const out: LocaleCode[] = [];
  for (const item of value) {
    const locale = asLocaleCode(item);
    if (!locale || out.includes(locale)) continue;
    out.push(locale);
  }
  return out;
}

function fallbackConfig(): RuntimeConfig {
  return {
    i18n: {
      defaultLocale: FALLBACK_RUNTIME_CONFIG.i18n.defaultLocale,
      supportedLocales: [...FALLBACK_RUNTIME_CONFIG.i18n.supportedLocales],
      defaultTimezone: FALLBACK_RUNTIME_CONFIG.i18n.defaultTimezone,
    },
  };
}

export function getRuntimeConfig(): RuntimeConfig {
  if (cachedRuntimeConfig) return cachedRuntimeConfig;

  if (typeof window === "undefined") {
    cachedRuntimeConfig = fallbackConfig();
    return cachedRuntimeConfig;
  }

  const wire = window.__RUSTFORGE_BOOTSTRAP__;
  if (!wire || typeof wire !== "object") {
    cachedRuntimeConfig = fallbackConfig();
    return cachedRuntimeConfig;
  }

  const wireI18n = wire.i18n ?? {};
  const defaultLocale =
    asLocaleCode(wireI18n.default_locale) ?? FALLBACK_RUNTIME_CONFIG.i18n.defaultLocale;
  const supportedLocales = asLocaleCodes(wireI18n.supported_locales);
  if (!supportedLocales.includes(defaultLocale)) {
    supportedLocales.unshift(defaultLocale);
  }
  const defaultTimezone =
    typeof wireI18n.default_timezone === "string" &&
    wireI18n.default_timezone.trim()
      ? wireI18n.default_timezone.trim()
      : FALLBACK_RUNTIME_CONFIG.i18n.defaultTimezone;

  cachedRuntimeConfig = {
    i18n: {
      defaultLocale,
      supportedLocales:
        supportedLocales.length > 0 ? supportedLocales : [defaultLocale],
      defaultTimezone,
    },
  };
  return cachedRuntimeConfig;
}
