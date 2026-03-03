import i18n from "@shared/i18n";
import { getRuntimeConfig } from "@shared/runtimeConfig";
import type { LocaleCode } from "@shared/types/platform";

const runtime = getRuntimeConfig();

function normalizeLocale(raw: string | null | undefined): LocaleCode | null {
  if (!raw) return null;
  const trimmed = raw.trim();
  if (!trimmed) return null;

  const lower = trimmed.toLowerCase();
  const direct = runtime.i18n.supportedLocales.find((locale) => locale.toLowerCase() === lower);
  if (direct) return direct;

  const base = lower.split("-")[0];
  if (!base) return null;
  return runtime.i18n.supportedLocales.find((locale) => locale.toLowerCase() === base) ?? null;
}

export function resolveLocaleHeader(): LocaleCode {
  const active = normalizeLocale(i18n.resolvedLanguage ?? i18n.language);
  if (active) return active;
  return runtime.i18n.defaultLocale;
}
