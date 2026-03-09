import { getRuntimeConfig } from "@shared/runtimeConfig";
import type { CountryRuntime } from "@shared/types/platform";

const runtime = getRuntimeConfig();

export function availableCountries(): CountryRuntime[] {
  return runtime.countries;
}

export function findCountryByIso2(
  iso2: string | null | undefined,
  countries: CountryRuntime[] = availableCountries(),
): CountryRuntime | null {
  const normalized = iso2?.trim().toUpperCase();
  if (!normalized) return null;
  return countries.find((country) => country.iso2.toUpperCase() === normalized) ?? null;
}

/**
 * Build a full international phone number from country_iso2 + local number.
 * Returns e.g. "+60166885555" or the local number as-is if country not found.
 */
export function formatFullPhoneNumber(
  iso2: string | null | undefined,
  localNumber: string | null | undefined,
): string | null {
  const phone = localNumber?.trim();
  if (!phone) return null;

  const country = findCountryByIso2(iso2);
  if (!country?.calling_code) return phone;

  const code = country.calling_code.trim();
  // Avoid double-prefixing if number already starts with the calling code
  if (phone.startsWith(code) || phone.startsWith(`+${code.replace("+", "")}`)) {
    return phone.startsWith("+") ? phone : `+${phone}`;
  }

  const prefix = code.startsWith("+") ? code : `+${code}`;
  return `${prefix}${phone}`;
}

/**
 * Build a display string with flag + full number, e.g. "🇲🇾 +60166885555"
 */
export function formatPhoneDisplay(
  iso2: string | null | undefined,
  localNumber: string | null | undefined,
): string | null {
  const full = formatFullPhoneNumber(iso2, localNumber);
  if (!full) return null;

  const country = findCountryByIso2(iso2);
  const flag = country?.flag_emoji?.trim();
  return flag ? `${flag} ${full}` : full;
}

export function defaultCountryIso2(
  countries: CountryRuntime[] = availableCountries(),
): string | null {
  const defaultCountry = countries.find((c) => c.is_default);
  if (defaultCountry) return defaultCountry.iso2;
  return countries[0]?.iso2 ?? null;
}
