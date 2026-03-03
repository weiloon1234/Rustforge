import { create } from "zustand";
import i18n from "@shared/i18n";
import { type LocaleCode } from "@shared/types/platform";
import { getRuntimeConfig } from "@shared/runtimeConfig";
import { normalizeSupportedLocale, resolveBrowserLocale } from "@shared/localeRuntime";

interface LocaleState {
  locale: LocaleCode;
  defaultLocale: LocaleCode;
  availableLocales: LocaleCode[];
  defaultTimezone: string;
  setLocale: (locale: LocaleCode) => Promise<void>;
  normalizeLocale: (raw: string | null | undefined) => LocaleCode | null;
  syncFromAccountLocale: (raw: string | null | undefined) => Promise<LocaleCode>;
}

const runtimeConfig = getRuntimeConfig();

function resolveInitialLocale(): LocaleCode {
  const current = normalizeSupportedLocale(i18n.resolvedLanguage ?? i18n.language);
  if (current) return current;
  return runtimeConfig.i18n.defaultLocale;
}

export const useLocaleStore = create<LocaleState>((set, get) => ({
  locale: resolveInitialLocale(),
  defaultLocale: runtimeConfig.i18n.defaultLocale,
  availableLocales: runtimeConfig.i18n.supportedLocales,
  defaultTimezone: runtimeConfig.i18n.defaultTimezone,
  setLocale: async (locale) => {
    if (!runtimeConfig.i18n.supportedLocales.includes(locale)) return;
    await i18n.changeLanguage(locale);
    set({ locale });
  },
  normalizeLocale: (raw) => normalizeSupportedLocale(raw, runtimeConfig.i18n.supportedLocales),
  syncFromAccountLocale: async (raw) => {
    const normalizedAccount = get().normalizeLocale(raw);
    const browserLocale = resolveBrowserLocale(runtimeConfig.i18n.supportedLocales);
    const target = normalizedAccount ?? browserLocale ?? runtimeConfig.i18n.defaultLocale;
    if (target !== get().locale) {
      await i18n.changeLanguage(target);
      set({ locale: target });
    }
    return target;
  },
}));
