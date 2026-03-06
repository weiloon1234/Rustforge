import {
  DEFAULT_LOCALE,
  type CountryCurrency,
  type CountryRuntime,
  type CountryStatus,
  type LocaleCode,
} from "@shared/types/platform";

interface RuntimeBootstrapWire {
  i18n?: {
    default_locale?: unknown;
    supported_locales?: unknown;
    default_timezone?: unknown;
  };
  countries?: unknown;
}

export interface RuntimeI18nConfig {
  defaultLocale: LocaleCode;
  supportedLocales: LocaleCode[];
  defaultTimezone: string;
}

export interface RuntimeConfig {
  i18n: RuntimeI18nConfig;
  countries: CountryRuntime[];
}

const FALLBACK_RUNTIME_CONFIG: RuntimeConfig = {
  i18n: {
    defaultLocale: DEFAULT_LOCALE,
    supportedLocales: [DEFAULT_LOCALE],
    defaultTimezone: "+00:00",
  },
  countries: [],
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

function asStatus(value: unknown): CountryStatus | null {
  if (typeof value !== "string") return null;
  const normalized = value.trim().toLowerCase();
  if (normalized === "enabled" || normalized === "disabled") {
    return normalized as CountryStatus;
  }
  return null;
}

function asStringOrNull(value: unknown): string | null {
  if (value === null || value === undefined) return null;
  if (typeof value !== "string") return null;
  const trimmed = value.trim();
  return trimmed ? trimmed : null;
}

function asNumberOrNull(value: unknown): number | null {
  if (value === null || value === undefined) return null;
  if (typeof value !== "number" || Number.isNaN(value)) return null;
  return value;
}

function asBooleanOrNull(value: unknown): boolean | null {
  if (value === null || value === undefined) return null;
  if (typeof value !== "boolean") return null;
  return value;
}

function asStringArray(value: unknown): string[] {
  if (!Array.isArray(value)) return [];
  const out: string[] = [];
  for (const item of value) {
    const text = asStringOrNull(item);
    if (!text) continue;
    out.push(text);
  }
  return out;
}

function asCountryRuntime(value: unknown): CountryRuntime | null {
  if (!value || typeof value !== "object" || Array.isArray(value)) return null;
  const row = value as Record<string, unknown>;

  const iso2 = asStringOrNull(row.iso2)?.toUpperCase();
  const iso3 = asStringOrNull(row.iso3)?.toUpperCase();
  const name = asStringOrNull(row.name);
  const status = asStatus(row.status);

  if (!iso2 || !iso3 || !name || !status) return null;

  const currencies = Array.isArray(row.currencies)
    ? row.currencies
        .map((item): CountryCurrency | null => {
          if (!item || typeof item !== "object" || Array.isArray(item)) return null;
          const currency = item as Record<string, unknown>;
          const code = asStringOrNull(currency.code)?.toUpperCase();
          if (!code) return null;
          return {
            code,
            name: asStringOrNull(currency.name) ?? undefined,
            symbol: asStringOrNull(currency.symbol) ?? undefined,
            minor_units: asNumberOrNull(currency.minor_units) ?? undefined,
          };
        })
        .filter((item): item is CountryCurrency => item !== null)
    : [];

  return {
    iso2,
    iso3,
    iso_numeric: asStringOrNull(row.iso_numeric),
    name,
    official_name: asStringOrNull(row.official_name),
    capital: asStringOrNull(row.capital),
    capitals: asStringArray(row.capitals),
    region: asStringOrNull(row.region),
    subregion: asStringOrNull(row.subregion),
    currencies,
    primary_currency_code: asStringOrNull(row.primary_currency_code),
    calling_code: asStringOrNull(row.calling_code),
    calling_root: asStringOrNull(row.calling_root),
    calling_suffixes: asStringArray(row.calling_suffixes),
    tlds: asStringArray(row.tlds),
    timezones: asStringArray(row.timezones),
    latitude: asNumberOrNull(row.latitude),
    longitude: asNumberOrNull(row.longitude),
    independent: asBooleanOrNull(row.independent),
    status,
    assignment_status: asStringOrNull(row.assignment_status),
    un_member: asBooleanOrNull(row.un_member),
    flag_emoji: asStringOrNull(row.flag_emoji),
    created_at: asStringOrNull(row.created_at) ?? "",
    updated_at: asStringOrNull(row.updated_at) ?? "",
  };
}

function asCountries(value: unknown): CountryRuntime[] {
  if (!Array.isArray(value)) return [];
  const out: CountryRuntime[] = [];
  const seen = new Set<string>();
  for (const item of value) {
    const country = asCountryRuntime(item);
    if (!country) continue;
    if (country.status !== "enabled") continue;
    if (seen.has(country.iso2)) continue;
    out.push(country);
    seen.add(country.iso2);
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
    countries: [],
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
    countries: asCountries(wire.countries),
  };
  return cachedRuntimeConfig;
}
