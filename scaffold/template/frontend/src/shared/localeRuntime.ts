import i18n from "@shared/i18n";
import { getRuntimeConfig } from "@shared/runtimeConfig";
import type { LocaleCode } from "@shared/types/platform";

const runtime = getRuntimeConfig();

export function availableLocales(): LocaleCode[] {
  return runtime.i18n.supportedLocales;
}

export function defaultLocale(): LocaleCode {
  return runtime.i18n.defaultLocale;
}

export function normalizeSupportedLocale(
  raw: string | null | undefined,
  locales: LocaleCode[] = availableLocales(),
): LocaleCode | null {
  if (!raw) return null;
  const trimmed = raw.trim();
  if (!trimmed) return null;

  const lower = trimmed.toLowerCase();
  const direct = locales.find((locale) => locale.toLowerCase() === lower);
  if (direct) return direct;

  const base = lower.split("-")[0];
  if (!base) return null;
  return locales.find((locale) => locale.toLowerCase() === base) ?? null;
}

export function resolveBrowserLocale(locales: LocaleCode[] = availableLocales()): LocaleCode | null {
  if (typeof navigator === "undefined") return null;
  return normalizeSupportedLocale(navigator.language, locales);
}

export function resolveLocaleHeader(): LocaleCode {
  const active = normalizeSupportedLocale(i18n.resolvedLanguage ?? i18n.language);
  return active ?? defaultLocale();
}
