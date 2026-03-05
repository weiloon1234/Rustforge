import type { ChangeEvent } from "react";
import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { availableCountries, defaultCountryIso2 } from "@shared/countryRuntime";
import { Select, type SelectOption } from "@shared/components/Select";
import { TextInput } from "@shared/components/TextInput";
import type { CountryRuntime } from "@shared/types/platform";

export interface ContactInputValue {
  country_iso2: string;
  phone_number: string;
}

export interface ContactInputProps {
  value: ContactInputValue;
  onChange: (value: ContactInputValue) => void;
  countries?: CountryRuntime[];
  label?: string;
  countryLabel?: string;
  phoneLabel?: string;
  countryName?: string;
  phoneName?: string;
  disabled?: boolean;
  required?: boolean;
  countryRequired?: boolean;
  phoneRequired?: boolean;
  countryError?: string;
  countryErrors?: string[];
  phoneError?: string;
  phoneErrors?: string[];
  containerClassName?: string;
}

function toOptionLabel(country: CountryRuntime): string {
  const flag = country.flag_emoji?.trim() ?? "";
  const callingCode = country.calling_code?.trim() ?? "";
  const prefix = [flag, callingCode].filter((part) => part.length > 0).join(" ");
  const base = `${country.name} (${country.iso2})`;
  return prefix ? `${prefix} ${base}` : base;
}

export function ContactInput({
  value,
  onChange,
  countries,
  label,
  countryLabel,
  phoneLabel,
  countryName = "country_iso2",
  phoneName = "phone_number",
  disabled = false,
  required = false,
  countryRequired,
  phoneRequired,
  countryError,
  countryErrors,
  phoneError,
  phoneErrors,
  containerClassName,
}: ContactInputProps) {
  const { t } = useTranslation();

  const resolvedCountries = useMemo(
    () => countries ?? availableCountries(),
    [countries],
  );

  const options = useMemo<SelectOption[]>(
    () =>
      resolvedCountries.map((country) => ({
        value: country.iso2,
        label: toOptionLabel(country),
      })),
    [resolvedCountries],
  );

  const fallbackIso2 = defaultCountryIso2(resolvedCountries) ?? "";
  const selectedIso2 = (value.country_iso2 || fallbackIso2).toUpperCase();
  const hasCountryOptions = options.length > 0;

  const handleCountryChange = (event: ChangeEvent<HTMLSelectElement>) => {
    onChange({
      country_iso2: event.target.value.toUpperCase(),
      phone_number: value.phone_number,
    });
  };

  const handlePhoneChange = (event: ChangeEvent<HTMLInputElement>) => {
    onChange({
      country_iso2: selectedIso2,
      phone_number: event.target.value,
    });
  };

  return (
    <div className={containerClassName}>
      {label && (
        <p className="mb-2 text-sm font-medium">{label}</p>
      )}
      <div className="grid gap-3 md:grid-cols-[minmax(12rem,16rem)_minmax(0,1fr)]">
        <Select
          name={countryName}
          label={countryLabel ?? t("Country")}
          value={selectedIso2}
          onChange={handleCountryChange}
          options={options}
          placeholder={
            hasCountryOptions ? t("Select country") : t("No countries available")
          }
          error={countryError}
          errors={countryErrors}
          required={countryRequired ?? required}
          disabled={disabled || !hasCountryOptions}
          containerClassName="mb-0"
        />
        <TextInput
          name={phoneName}
          type="tel"
          label={phoneLabel ?? t("Phone Number")}
          value={value.phone_number}
          onChange={handlePhoneChange}
          placeholder={t("Enter phone number")}
          error={phoneError}
          errors={phoneErrors}
          required={phoneRequired ?? required}
          disabled={disabled}
          containerClassName="mb-0"
        />
      </div>
    </div>
  );
}
