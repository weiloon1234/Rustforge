import { create } from "zustand";
import i18n from "@shared/i18n";
import { type LocaleCode } from "@shared/types/platform";
import { getRuntimeConfig } from "@shared/runtimeConfig";

interface LocaleState {
  locale: LocaleCode;
  defaultLocale: LocaleCode;
  availableLocales: LocaleCode[];
  defaultTimezone: string;
  setLocale: (locale: LocaleCode) => Promise<void>;
}

const runtimeConfig = getRuntimeConfig();

function resolveInitialLocale(): LocaleCode {
  const current = i18n.resolvedLanguage ?? i18n.language;
  if (current && runtimeConfig.i18n.supportedLocales.includes(current as LocaleCode)) {
    return current as LocaleCode;
  }
  return runtimeConfig.i18n.defaultLocale;
}

export const useLocaleStore = create<LocaleState>((set) => ({
  locale: resolveInitialLocale(),
  defaultLocale: runtimeConfig.i18n.defaultLocale,
  availableLocales: runtimeConfig.i18n.supportedLocales,
  defaultTimezone: runtimeConfig.i18n.defaultTimezone,
  setLocale: async (locale) => {
    if (!runtimeConfig.i18n.supportedLocales.includes(locale)) return;
    await i18n.changeLanguage(locale);
    set({ locale });
  },
}));
