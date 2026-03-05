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

export function defaultCountryIso2(
  countries: CountryRuntime[] = availableCountries(),
): string | null {
  const preferred = findCountryByIso2("MY", countries);
  if (preferred) return preferred.iso2;
  return countries[0]?.iso2 ?? null;
}
